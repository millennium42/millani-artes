use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::{FixedRecurrence, RecurrenceObligation};
use std::fmt;
use std::time::UNIX_EPOCH;

pub trait FixedObligationStore {
    type Error;

    fn persist_fixed_obligation_with_audit(
        &mut self,
        recurrence: &FixedRecurrence,
        obligation: &RecurrenceObligation,
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<(), Self::Error>;
}

#[derive(PartialEq, Eq)]
pub enum GenerateFixedObligationError {
    InvalidInput,
    IdGenerationFailed,
    ClockFailed,
    StoreFailed,
}

impl fmt::Debug for GenerateFixedObligationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidInput => "InvalidInput",
            Self::IdGenerationFailed => "IdGenerationFailed",
            Self::ClockFailed => "ClockFailed",
            Self::StoreFailed => "StoreFailed",
        })
    }
}

impl fmt::Display for GenerateFixedObligationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidInput => "invalid input",
            Self::IdGenerationFailed => "fixed obligation identifiers could not be generated",
            Self::ClockFailed => "fixed obligation timestamp is invalid",
            Self::StoreFailed => "fixed obligation could not be stored",
        })
    }
}

impl std::error::Error for GenerateFixedObligationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub fn generate_fixed_obligation<S: FixedObligationStore, I: UuidGenerator, C: Clock>(
    recurrence: &FixedRecurrence,
    period: (u16, u8),
    ids: &mut I,
    clock: &C,
    store: &mut S,
) -> Result<RecurrenceObligation, GenerateFixedObligationError> {
    let mut obligation = recurrence
        .generate_obligation([0; 16], period)
        .map_err(|_| GenerateFixedObligationError::InvalidInput)?;

    obligation.id = ids
        .generate()
        .map_err(|_| GenerateFixedObligationError::IdGenerationFailed)?;
    let event_id = ids
        .generate()
        .map_err(|_| GenerateFixedObligationError::IdGenerationFailed)?;
    let occurred_at = clock
        .now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| GenerateFixedObligationError::ClockFailed)?;
    let occurred_at_ms = duration_to_epoch_millis(occurred_at)?;

    store
        .persist_fixed_obligation_with_audit(recurrence, &obligation, event_id, occurred_at_ms)
        .map_err(|_| GenerateFixedObligationError::StoreFailed)?;

    Ok(obligation)
}

fn duration_to_epoch_millis(
    duration: std::time::Duration,
) -> Result<i64, GenerateFixedObligationError> {
    i64::try_from(duration.as_millis()).map_err(|_| GenerateFixedObligationError::ClockFailed)
}

#[cfg(test)]
#[path = "generate_fixed_obligation_tests.rs"]
mod tests;
