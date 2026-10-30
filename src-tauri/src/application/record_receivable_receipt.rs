use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::{Context, Movement, MovementKind, ReceivableReceipt};
use std::fmt;
use std::time::UNIX_EPOCH;

pub struct ReceivableReceiptWrite<'a> {
    pub receivable_id: [u8; 16],
    pub receipt_id: [u8; 16],
    pub receipt: &'a ReceivableReceipt,
    pub receipt_event_id: [u8; 16],
    pub movement: &'a Movement,
    pub movement_event_id: [u8; 16],
    pub occurred_at_ms: i64,
}

pub trait ReceivableReceiptRecordingStore {
    fn persist_receipt_and_entry(
        &mut self,
        write: ReceivableReceiptWrite<'_>,
    ) -> Result<(), RecordReceivableReceiptError>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum RecordReceivableReceiptError {
    InvalidInput,
    IdGenerationFailed,
    ClockFailed,
    ReceivableNotFound,
    AmountExceedsDue,
    StoreFailed,
}

impl fmt::Display for RecordReceivableReceiptError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidInput => "receipt amount is invalid",
            Self::IdGenerationFailed => "receipt identifiers could not be generated",
            Self::ClockFailed => "receipt timestamp is invalid",
            Self::ReceivableNotFound => "receivable could not be found",
            Self::AmountExceedsDue => "receipt amount exceeds the due balance",
            Self::StoreFailed => "receipt and entry could not be stored",
        })
    }
}

impl std::error::Error for RecordReceivableReceiptError {}

pub fn record_receivable_receipt<S: ReceivableReceiptRecordingStore, I: UuidGenerator, C: Clock>(
    receivable_id: [u8; 16],
    receipt: ReceivableReceipt,
    ids: &mut I,
    clock: &C,
    store: &mut S,
) -> Result<Movement, RecordReceivableReceiptError> {
    let movement_context = match &receipt.context {
        Context::Home => Context::Home,
        Context::Factory => Context::Factory,
    };
    let mut movement = Movement {
        id: [0; 16],
        kind: MovementKind::Entry {
            account_id: receipt.account_id,
        },
        amount_cents: receipt.amount_cents,
        context: movement_context,
    };
    movement
        .validate_entry()
        .map_err(|_| RecordReceivableReceiptError::InvalidInput)?;

    let receipt_id = generate_id(ids)?;
    let receipt_event_id = generate_id(ids)?;
    movement.id = generate_id(ids)?;
    let movement_event_id = generate_id(ids)?;
    let occurred_at = clock
        .now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| RecordReceivableReceiptError::ClockFailed)?;
    let occurred_at_ms = i64::try_from(occurred_at.as_millis())
        .map_err(|_| RecordReceivableReceiptError::ClockFailed)?;

    store.persist_receipt_and_entry(ReceivableReceiptWrite {
        receivable_id,
        receipt_id,
        receipt: &receipt,
        receipt_event_id,
        movement: &movement,
        movement_event_id,
        occurred_at_ms,
    })?;
    Ok(movement)
}

fn generate_id<I: UuidGenerator>(ids: &mut I) -> Result<[u8; 16], RecordReceivableReceiptError> {
    ids.generate()
        .map_err(|_| RecordReceivableReceiptError::IdGenerationFailed)
}

#[cfg(test)]
#[path = "record_receivable_receipt_tests.rs"]
mod tests;
