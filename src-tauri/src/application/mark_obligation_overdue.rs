use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::{ObligationState, RecurrenceObligation};
use std::fmt;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

pub trait OverdueObligationStore {
    type Error;

    fn persist_overdue_obligation_with_audit(
        &mut self,
        obligation: &RecurrenceObligation,
        event_id: [u8; 16],
        observed_at: SystemTime,
    ) -> Result<(), Self::Error>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum MarkObligationOverdueError {
    InvalidInput,
    ClockFailed,
    IdGenerationFailed,
    StoreFailed,
}

impl fmt::Display for MarkObligationOverdueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidInput => "obligation is invalid",
            Self::ClockFailed => "obligation timestamp is invalid",
            Self::IdGenerationFailed => "obligation audit identifier could not be generated",
            Self::StoreFailed => "obligation could not be edited",
        })
    }
}

impl std::error::Error for MarkObligationOverdueError {}

pub fn mark_obligation_overdue<S: OverdueObligationStore, I: UuidGenerator, C: Clock>(
    mut obligation: RecurrenceObligation,
    ids: &mut I,
    clock: &C,
    store: &mut S,
) -> Result<RecurrenceObligation, MarkObligationOverdueError> {
    obligation
        .state
        .validate()
        .map_err(|_| MarkObligationOverdueError::InvalidInput)?;

    if !matches!(&obligation.state, ObligationState::Payable { .. }) {
        return Err(MarkObligationOverdueError::InvalidInput);
    }

    let observed_at = clock.now();

    obligation.state = obligation
        .state
        .mark_overdue_ms(obligation.due_at_ms, &ObservedClock(observed_at))
        .map_err(|_| MarkObligationOverdueError::InvalidInput)?;

    audit_millis(observed_at).map_err(|_| MarkObligationOverdueError::ClockFailed)?;

    let event_id = ids
        .generate()
        .map_err(|_| MarkObligationOverdueError::IdGenerationFailed)?;

    store
        .persist_overdue_obligation_with_audit(&obligation, event_id, observed_at)
        .map_err(|_| MarkObligationOverdueError::StoreFailed)?;

    Ok(obligation)
}

pub(crate) struct ObservedClock(pub(crate) SystemTime);

impl Clock for ObservedClock {
    fn now(&self) -> SystemTime {
        self.0
    }
}

pub(crate) fn audit_millis(observed_at: SystemTime) -> Result<i64, MarkObligationOverdueError> {
    observed_at
        .duration_since(UNIX_EPOCH)
        .map_err(|_| MarkObligationOverdueError::ClockFailed)
        .and_then(duration_to_epoch_millis)
}

fn duration_to_epoch_millis(
    duration: std::time::Duration,
) -> Result<i64, MarkObligationOverdueError> {
    i64::try_from(duration.as_millis()).map_err(|_| MarkObligationOverdueError::ClockFailed)
}

#[cfg(test)]
#[path = "mark_obligation_overdue_tests.rs"]
mod tests;
