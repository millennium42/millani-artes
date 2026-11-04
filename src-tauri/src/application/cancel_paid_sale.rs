use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::Context;
use std::collections::HashMap;
use std::fmt;
use std::time::UNIX_EPOCH;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum SaleCancellationRefundTarget {
    ExistingReturnItem([u8; 16]),
    CancellationReturnForSaleItem([u8; 16]),
}

#[derive(PartialEq, Eq)]
pub struct SaleCancellationReturnPlan {
    pub sale_item_id: [u8; 16],
    pub quantity: i64,
    pub return_amount_cents: i64,
}

#[derive(PartialEq, Eq)]
pub struct SaleCancellationRefundPlan {
    pub target: SaleCancellationRefundTarget,
    pub amount_cents: i64,
}

#[derive(PartialEq, Eq)]
pub struct SaleCancellationPreview {
    pub sale_id: [u8; 16],
    pub returns: Vec<SaleCancellationReturnPlan>,
    pub refunds: Vec<SaleCancellationRefundPlan>,
}

pub struct SaleCancellationRefundRoute {
    pub target: SaleCancellationRefundTarget,
    pub account_id: [u8; 16],
    pub context: Context,
}

pub struct SaleCancellationReturnWrite {
    pub id: [u8; 16],
    pub sale_item_id: [u8; 16],
    pub quantity: i64,
    pub return_amount_cents: i64,
    pub audit_event_id: [u8; 16],
}

pub struct SaleCancellationRefundWrite {
    pub target: SaleCancellationRefundTarget,
    pub return_item_id: [u8; 16],
    pub amount_cents: i64,
    pub account_id: [u8; 16],
    pub context: Context,
    pub movement_id: [u8; 16],
    pub audit_event_id: [u8; 16],
}

pub struct SaleCancellationWrite {
    pub preview: SaleCancellationPreview,
    pub returns: Vec<SaleCancellationReturnWrite>,
    pub refunds: Vec<SaleCancellationRefundWrite>,
    pub cancellation_audit_event_id: [u8; 16],
    pub occurred_at_ms: i64,
}

pub trait SaleCancellationStore {
    fn preview_sale_cancellation(
        &mut self,
        sale_id: [u8; 16],
    ) -> Result<SaleCancellationPreview, CancelPaidSaleError>;

    fn persist_paid_sale_cancellation(
        &mut self,
        write: SaleCancellationWrite,
    ) -> Result<(), CancelPaidSaleError>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum CancelPaidSaleError {
    InvalidInput,
    RefundRoutesInvalid,
    IdGenerationFailed,
    ClockFailed,
    SaleNotFound,
    SaleNotFinalized,
    ReceivableNotSettled,
    AlreadyCancelled,
    AccountNotFound,
    StoreFailed,
}

impl fmt::Display for CancelPaidSaleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidInput => "sale cancellation input is invalid",
            Self::RefundRoutesInvalid => "refund routes do not match the reserved refunds",
            Self::IdGenerationFailed => "sale cancellation identifiers could not be generated",
            Self::ClockFailed => "sale cancellation timestamp is invalid",
            Self::SaleNotFound => "sale could not be found",
            Self::SaleNotFinalized => "sale is not finalized",
            Self::ReceivableNotSettled => "sale receivable is not settled",
            Self::AlreadyCancelled => "sale is already cancelled",
            Self::AccountNotFound => "refund account could not be found",
            Self::StoreFailed => "sale cancellation could not be stored",
        })
    }
}

impl std::error::Error for CancelPaidSaleError {}

pub fn preview_sale_cancellation<S: SaleCancellationStore>(
    sale_id: [u8; 16],
    store: &mut S,
) -> Result<SaleCancellationPreview, CancelPaidSaleError> {
    store.preview_sale_cancellation(sale_id)
}

pub fn cancel_paid_sale<S: SaleCancellationStore, I: UuidGenerator, C: Clock>(
    preview: SaleCancellationPreview,
    routes: Vec<SaleCancellationRefundRoute>,
    ids: &mut I,
    clock: &C,
    store: &mut S,
) -> Result<(), CancelPaidSaleError> {
    let selected_account_id = routes.first().map(|route| route.account_id);
    if routes
        .iter()
        .any(|route| Some(route.account_id) != selected_account_id)
    {
        return Err(CancelPaidSaleError::RefundRoutesInvalid);
    }

    let mut routes_by_target = HashMap::with_capacity(routes.len());
    for route in routes {
        if routes_by_target.insert(route.target, route).is_some() {
            return Err(CancelPaidSaleError::RefundRoutesInvalid);
        }
    }
    if routes_by_target.len() != preview.refunds.len()
        || preview
            .refunds
            .iter()
            .any(|refund| !routes_by_target.contains_key(&refund.target))
    {
        return Err(CancelPaidSaleError::RefundRoutesInvalid);
    }

    let mut return_ids = HashMap::with_capacity(preview.returns.len());
    let mut returns = Vec::with_capacity(preview.returns.len());
    for item in &preview.returns {
        let return_id = generate_id(ids)?;
        if return_ids.insert(item.sale_item_id, return_id).is_some() {
            return Err(CancelPaidSaleError::InvalidInput);
        }
        returns.push(SaleCancellationReturnWrite {
            id: return_id,
            sale_item_id: item.sale_item_id,
            quantity: item.quantity,
            return_amount_cents: item.return_amount_cents,
            audit_event_id: generate_id(ids)?,
        });
    }

    let mut refunds = Vec::with_capacity(preview.refunds.len());
    for refund in &preview.refunds {
        let route = routes_by_target
            .remove(&refund.target)
            .ok_or(CancelPaidSaleError::RefundRoutesInvalid)?;
        let return_item_id = match refund.target {
            SaleCancellationRefundTarget::ExistingReturnItem(id) => id,
            SaleCancellationRefundTarget::CancellationReturnForSaleItem(sale_item_id) => {
                *return_ids
                    .get(&sale_item_id)
                    .ok_or(CancelPaidSaleError::InvalidInput)?
            }
        };
        refunds.push(SaleCancellationRefundWrite {
            target: refund.target,
            return_item_id,
            amount_cents: refund.amount_cents,
            account_id: route.account_id,
            context: route.context,
            movement_id: generate_id(ids)?,
            audit_event_id: generate_id(ids)?,
        });
    }

    let cancellation_audit_event_id = generate_id(ids)?;
    let occurred_at = clock
        .now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| CancelPaidSaleError::ClockFailed)?;
    let occurred_at_ms =
        i64::try_from(occurred_at.as_millis()).map_err(|_| CancelPaidSaleError::ClockFailed)?;

    store.persist_paid_sale_cancellation(SaleCancellationWrite {
        preview,
        returns,
        refunds,
        cancellation_audit_event_id,
        occurred_at_ms,
    })
}

fn generate_id<I: UuidGenerator>(ids: &mut I) -> Result<[u8; 16], CancelPaidSaleError> {
    ids.generate()
        .map_err(|_| CancelPaidSaleError::IdGenerationFailed)
}
