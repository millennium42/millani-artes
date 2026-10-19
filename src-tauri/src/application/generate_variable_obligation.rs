use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::{RecurrenceObligation, VariableRecurrence};
use std::fmt;
use std::time::UNIX_EPOCH;

pub trait VariableObligationStore {
    type Error;

    fn persist_variable_obligation_with_audit(
        &mut self,
        recurrence: &VariableRecurrence,
        obligation: &RecurrenceObligation,
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<(), Self::Error>;
}

#[derive(PartialEq, Eq)]
pub enum GenerateVariableObligationError {
    InvalidInput,
    IdGenerationFailed,
    ClockFailed,
    StoreFailed,
}

impl fmt::Debug for GenerateVariableObligationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidInput => "InvalidInput",
            Self::IdGenerationFailed => "IdGenerationFailed",
            Self::ClockFailed => "ClockFailed",
            Self::StoreFailed => "StoreFailed",
        })
    }
}

impl fmt::Display for GenerateVariableObligationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidInput => "invalid input",
            Self::IdGenerationFailed => "variable obligation identifiers could not be generated",
            Self::ClockFailed => "variable obligation timestamp is invalid",
            Self::StoreFailed => "variable obligation could not be stored",
        })
    }
}

impl std::error::Error for GenerateVariableObligationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub fn generate_variable_obligation<S: VariableObligationStore, I: UuidGenerator, C: Clock>(
    recurrence: &VariableRecurrence,
    period: (u16, u8),
    ids: &mut I,
    clock: &C,
    store: &mut S,
) -> Result<RecurrenceObligation, GenerateVariableObligationError> {
    let mut obligation = recurrence
        .generate_obligation([0; 16], period)
        .map_err(|_| GenerateVariableObligationError::InvalidInput)?;

    obligation.id = ids
        .generate()
        .map_err(|_| GenerateVariableObligationError::IdGenerationFailed)?;
    let event_id = ids
        .generate()
        .map_err(|_| GenerateVariableObligationError::IdGenerationFailed)?;
    let occurred_at = clock
        .now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| GenerateVariableObligationError::ClockFailed)?;
    let occurred_at_ms = duration_to_epoch_millis(occurred_at)?;

    store
        .persist_variable_obligation_with_audit(recurrence, &obligation, event_id, occurred_at_ms)
        .map_err(|_| GenerateVariableObligationError::StoreFailed)?;

    Ok(obligation)
}

fn duration_to_epoch_millis(
    duration: std::time::Duration,
) -> Result<i64, GenerateVariableObligationError> {
    i64::try_from(duration.as_millis()).map_err(|_| GenerateVariableObligationError::ClockFailed)
}

#[cfg(test)]
#[path = "generate_variable_obligation_tests.rs"]
mod tests;
