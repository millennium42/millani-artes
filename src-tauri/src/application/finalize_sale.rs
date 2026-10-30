use super::add_product_item::SaleError;
use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::{Movement, MovementKind, SalePayment};
use std::time::UNIX_EPOCH;

pub struct FinalizeSaleInput {
    pub sale_id: [u8; 16],
    pub payments: Vec<SalePayment>,
    pub pending: Option<PendingSaleTerms>,
}

pub struct PendingSaleTerms {
    pub customer_id: [u8; 16],
    pub due_at_ms: i64,
}

pub struct SalePaymentWrite {
    pub movement: Movement,
    pub event_id: [u8; 16],
}

pub struct PendingReceivableWrite {
    pub receivable_id: [u8; 16],
    pub event_id: [u8; 16],
    pub customer_id: [u8; 16],
    pub due_at_ms: i64,
}

pub struct FinalizeSaleWrite {
    pub sale_id: [u8; 16],
    pub payments: Vec<SalePaymentWrite>,
    pub sale_event_id: [u8; 16],
    pub occurred_at_ms: i64,
    pub pending: Option<PendingReceivableWrite>,
}

pub struct FinalizedSale {
    pub sale_id: [u8; 16],
    pub total_cents: i64,
    pub paid_cents: i64,
    pub pending_cents: i64,
}

pub trait SaleFinalizationStore {
    fn finalize_sale_with_audit(
        &mut self,
        write: FinalizeSaleWrite,
    ) -> Result<FinalizedSale, SaleError>;
}

pub fn finalize_sale<S: SaleFinalizationStore, I: UuidGenerator, C: Clock>(
    input: FinalizeSaleInput,
    ids: &mut I,
    clock: &C,
    store: &mut S,
) -> Result<FinalizedSale, SaleError> {
    input.payments.iter().try_fold(0_i64, |total, payment| {
        if payment.amount_cents <= 0 {
            return Err(SaleError::InvalidPayment);
        }
        total
            .checked_add(payment.amount_cents)
            .ok_or(SaleError::InvalidPayment)
    })?;

    let sale_event_id = generate_id(ids)?;
    let mut payment_writes = Vec::with_capacity(input.payments.len());
    for payment in input.payments {
        let movement_id = generate_id(ids)?;
        let event_id = generate_id(ids)?;
        let movement = Movement {
            id: movement_id,
            kind: MovementKind::Entry {
                account_id: payment.account_id,
            },
            amount_cents: payment.amount_cents,
            context: payment.context,
        };
        payment_writes.push(SalePaymentWrite { movement, event_id });
    }

    let pending = input
        .pending
        .map(|terms| {
            Ok(PendingReceivableWrite {
                receivable_id: generate_id(ids)?,
                event_id: generate_id(ids)?,
                customer_id: terms.customer_id,
                due_at_ms: terms.due_at_ms,
            })
        })
        .transpose()?;
    let occurred_at_ms = clock
        .now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| SaleError::ClockFailed)?;
    let occurred_at_ms =
        i64::try_from(occurred_at_ms.as_millis()).map_err(|_| SaleError::ClockFailed)?;

    store.finalize_sale_with_audit(FinalizeSaleWrite {
        sale_id: input.sale_id,
        payments: payment_writes,
        sale_event_id,
        occurred_at_ms,
        pending,
    })
}

fn generate_id<I: UuidGenerator>(ids: &mut I) -> Result<[u8; 16], SaleError> {
    let id = ids.generate().map_err(|_| SaleError::IdGenerationFailed)?;
    if id == [0; 16] {
        return Err(SaleError::IdGenerationFailed);
    }
    Ok(id)
}

#[cfg(test)]
#[path = "finalize_sale_tests.rs"]
mod tests;
