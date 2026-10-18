use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::{Context, Movement, MovementKind};
use std::fmt;
use std::time::UNIX_EPOCH;

pub trait EntryCreationStore {
    type Error;

    fn persist_entry_with_audit(
        &mut self,
        movement: &Movement,
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<(), Self::Error>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum CreateEntryError {
    InvalidInput,
    IdGenerationFailed,
    ClockFailed,
    StoreFailed,
}

impl fmt::Display for CreateEntryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidInput => "entry is invalid",
            Self::IdGenerationFailed => "entry identifiers could not be generated",
            Self::ClockFailed => "entry timestamp is invalid",
            Self::StoreFailed => "entry could not be stored",
        })
    }
}

impl std::error::Error for CreateEntryError {}

pub fn create_entry<S: EntryCreationStore, I: UuidGenerator, C: Clock>(
    account_id: [u8; 16],
    amount_cents: i64,
    context: Context,
    ids: &mut I,
    clock: &C,
    store: &mut S,
) -> Result<Movement, CreateEntryError> {
    let mut movement = Movement {
        id: [0; 16],
        kind: MovementKind::Entry { account_id },
        amount_cents,
        context,
    };
    movement
        .validate_entry()
        .map_err(|_| CreateEntryError::InvalidInput)?;

    movement.id = ids
        .generate()
        .map_err(|_| CreateEntryError::IdGenerationFailed)?;
    let event_id = ids
        .generate()
        .map_err(|_| CreateEntryError::IdGenerationFailed)?;
    let occurred_at = clock
        .now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| CreateEntryError::ClockFailed)?;
    let occurred_at_ms = duration_to_epoch_millis(occurred_at)?;

    store
        .persist_entry_with_audit(&movement, event_id, occurred_at_ms)
        .map_err(|_| CreateEntryError::StoreFailed)?;
    Ok(movement)
}

fn duration_to_epoch_millis(duration: std::time::Duration) -> Result<i64, CreateEntryError> {
    i64::try_from(duration.as_millis()).map_err(|_| CreateEntryError::ClockFailed)
}

#[cfg(test)]
#[path = "create_entry_tests.rs"]
mod tests;
