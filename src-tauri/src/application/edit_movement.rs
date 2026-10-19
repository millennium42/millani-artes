use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::Movement;
use std::fmt;
use std::time::UNIX_EPOCH;

pub trait MovementEditingStore {
    type Error;

    fn persist_movement_edit_with_audit(
        &mut self,
        movement: &Movement,
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<(), Self::Error>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum EditMovementError {
    InvalidInput,
    IdGenerationFailed,
    ClockFailed,
    StoreFailed,
}

impl fmt::Display for EditMovementError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidInput => "movement is invalid",
            Self::IdGenerationFailed => "movement audit identifier could not be generated",
            Self::ClockFailed => "movement timestamp is invalid",
            Self::StoreFailed => "movement could not be edited",
        })
    }
}

impl std::error::Error for EditMovementError {}

pub fn edit_movement<S: MovementEditingStore, I: UuidGenerator, C: Clock>(
    movement: Movement,
    ids: &mut I,
    clock: &C,
    store: &mut S,
) -> Result<Movement, EditMovementError> {
    movement
        .result_effect_cents()
        .map_err(|_| EditMovementError::InvalidInput)?;

    let event_id = ids
        .generate()
        .map_err(|_| EditMovementError::IdGenerationFailed)?;
    let occurred_at = clock
        .now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| EditMovementError::ClockFailed)?;
    let occurred_at_ms = duration_to_epoch_millis(occurred_at)?;

    store
        .persist_movement_edit_with_audit(&movement, event_id, occurred_at_ms)
        .map_err(|_| EditMovementError::StoreFailed)?;
    Ok(movement)
}

fn duration_to_epoch_millis(duration: std::time::Duration) -> Result<i64, EditMovementError> {
    i64::try_from(duration.as_millis()).map_err(|_| EditMovementError::ClockFailed)
}

#[cfg(test)]
#[path = "edit_movement_tests.rs"]
mod tests;
