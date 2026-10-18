use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::{Context, Movement, MovementKind};
use std::fmt;
use std::time::UNIX_EPOCH;

pub trait ExitCreationStore {
    type Error;

    fn persist_exit_with_audit(
        &mut self,
        movement: &Movement,
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<(), Self::Error>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum CreateExitError {
    InvalidInput,
    IdGenerationFailed,
    ClockFailed,
    StoreFailed,
}

impl fmt::Display for CreateExitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidInput => "exit is invalid",
            Self::IdGenerationFailed => "exit identifiers could not be generated",
            Self::ClockFailed => "exit timestamp is invalid",
            Self::StoreFailed => "exit could not be stored",
        })
    }
}

impl std::error::Error for CreateExitError {}

pub fn create_exit<S: ExitCreationStore, I: UuidGenerator, C: Clock>(
    account_id: [u8; 16],
    amount_cents: i64,
    context: Context,
    ids: &mut I,
    clock: &C,
    store: &mut S,
) -> Result<Movement, CreateExitError> {
    let mut movement = Movement {
        id: [0; 16],
        kind: MovementKind::Exit { account_id },
        amount_cents,
        context,
    };
    movement
        .validate_exit()
        .map_err(|_| CreateExitError::InvalidInput)?;

    movement.id = ids
        .generate()
        .map_err(|_| CreateExitError::IdGenerationFailed)?;
    let event_id = ids
        .generate()
        .map_err(|_| CreateExitError::IdGenerationFailed)?;
    let occurred_at = clock
        .now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| CreateExitError::ClockFailed)?;
    let occurred_at_ms = duration_to_epoch_millis(occurred_at)?;

    store
        .persist_exit_with_audit(&movement, event_id, occurred_at_ms)
        .map_err(|_| CreateExitError::StoreFailed)?;
    Ok(movement)
}

fn duration_to_epoch_millis(duration: std::time::Duration) -> Result<i64, CreateExitError> {
    i64::try_from(duration.as_millis()).map_err(|_| CreateExitError::ClockFailed)
}

#[cfg(test)]
#[path = "create_exit_tests.rs"]
mod tests;
