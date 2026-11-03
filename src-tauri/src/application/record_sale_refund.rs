use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::{Context, Movement, MovementKind};
use std::fmt;
use std::time::UNIX_EPOCH;

pub struct SaleRefundWrite<'a> {
    pub return_item_id: [u8; 16],
    pub movement: &'a Movement,
    pub audit_event_id: [u8; 16],
    pub occurred_at_ms: i64,
}

pub trait SaleRefundRecordingStore {
    fn persist_refund_with_audit(
        &mut self,
        write: SaleRefundWrite<'_>,
    ) -> Result<(), RecordSaleRefundError>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum RecordSaleRefundError {
    InvalidInput,
    IdGenerationFailed,
    ClockFailed,
    ReturnItemNotFound,
    AmountExceedsAvailable,
    StoreFailed,
}

impl fmt::Display for RecordSaleRefundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidInput => "refund amount is invalid",
            Self::IdGenerationFailed => "refund identifiers could not be generated",
            Self::ClockFailed => "refund timestamp is invalid",
            Self::ReturnItemNotFound => "return item could not be found",
            Self::AmountExceedsAvailable => "refund amount exceeds the available reserved value",
            Self::StoreFailed => "refund could not be stored",
        })
    }
}

impl std::error::Error for RecordSaleRefundError {}

pub fn record_sale_refund<S: SaleRefundRecordingStore, I: UuidGenerator, C: Clock>(
    return_item_id: [u8; 16],
    account_id: [u8; 16],
    amount_cents: i64,
    context: Context,
    ids: &mut I,
    clock: &C,
    store: &mut S,
) -> Result<Movement, RecordSaleRefundError> {
    if amount_cents <= 0 {
        return Err(RecordSaleRefundError::InvalidInput);
    }
    let mut movement = Movement {
        id: [0; 16],
        kind: MovementKind::Exit { account_id },
        amount_cents,
        context,
    };
    movement
        .validate_exit()
        .map_err(|_| RecordSaleRefundError::InvalidInput)?;
    movement.id = ids
        .generate()
        .map_err(|_| RecordSaleRefundError::IdGenerationFailed)?;
    let audit_event_id = ids
        .generate()
        .map_err(|_| RecordSaleRefundError::IdGenerationFailed)?;
    let occurred_at = clock
        .now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| RecordSaleRefundError::ClockFailed)?;
    let occurred_at_ms =
        i64::try_from(occurred_at.as_millis()).map_err(|_| RecordSaleRefundError::ClockFailed)?;

    store.persist_refund_with_audit(SaleRefundWrite {
        return_item_id,
        movement: &movement,
        audit_event_id,
        occurred_at_ms,
    })?;
    Ok(movement)
}

#[cfg(test)]
#[path = "record_sale_refund_tests.rs"]
mod tests;
