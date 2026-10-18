use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::{Context, Movement, MovementKind};
use std::fmt;
use std::time::UNIX_EPOCH;

pub trait TransferCreationStore {
    type Error;

    fn persist_transfer_with_audit(
        &mut self,
        movement: &Movement,
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<(), Self::Error>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum CreateTransferError {
    InvalidInput,
    IdGenerationFailed,
    ClockFailed,
    StoreFailed,
}

impl fmt::Display for CreateTransferError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidInput => "transfer is invalid",
            Self::IdGenerationFailed => "transfer identifiers could not be generated",
            Self::ClockFailed => "transfer timestamp is invalid",
            Self::StoreFailed => "transfer could not be stored",
        })
    }
}

impl std::error::Error for CreateTransferError {}

pub fn create_transfer<S: TransferCreationStore, I: UuidGenerator, C: Clock>(
    source_account_id: [u8; 16],
    destination_account_id: [u8; 16],
    amount_cents: i64,
    context: Context,
    ids: &mut I,
    clock: &C,
    store: &mut S,
) -> Result<Movement, CreateTransferError> {
    let mut movement = Movement {
        id: [0; 16],
        kind: MovementKind::Transfer {
            source_account_id,
            destination_account_id,
        },
        amount_cents,
        context,
    };
    movement
        .validate_transfer()
        .map_err(|_| CreateTransferError::InvalidInput)?;

    movement.id = ids
        .generate()
        .map_err(|_| CreateTransferError::IdGenerationFailed)?;
    let event_id = ids
        .generate()
        .map_err(|_| CreateTransferError::IdGenerationFailed)?;
    let occurred_at = clock
        .now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| CreateTransferError::ClockFailed)?;
    let occurred_at_ms = duration_to_epoch_millis(occurred_at)?;

    store
        .persist_transfer_with_audit(&movement, event_id, occurred_at_ms)
        .map_err(|_| CreateTransferError::StoreFailed)?;
    Ok(movement)
}

fn duration_to_epoch_millis(duration: std::time::Duration) -> Result<i64, CreateTransferError> {
    i64::try_from(duration.as_millis()).map_err(|_| CreateTransferError::ClockFailed)
}

#[cfg(test)]
#[path = "create_transfer_tests.rs"]
mod tests;
