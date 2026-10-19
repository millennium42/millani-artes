use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::RecurrenceObligation;
use std::fmt;
use std::time::UNIX_EPOCH;

pub trait ObligationAmountStore {
    type Error;

    fn persist_obligation_amount_with_audit(
        &mut self,
        obligation: &RecurrenceObligation,
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<(), Self::Error>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum SupplyObligationAmountError {
    InvalidInput,
    IdGenerationFailed,
    ClockFailed,
    StoreFailed,
}

impl fmt::Display for SupplyObligationAmountError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidInput => "obligation is invalid",
            Self::IdGenerationFailed => "obligation audit identifier could not be generated",
            Self::ClockFailed => "obligation timestamp is invalid",
            Self::StoreFailed => "obligation could not be edited",
        })
    }
}

impl std::error::Error for SupplyObligationAmountError {}

pub fn supply_obligation_amount<S: ObligationAmountStore, I: UuidGenerator, C: Clock>(
    mut obligation: RecurrenceObligation,
    amount_cents: i64,
    ids: &mut I,
    clock: &C,
    store: &mut S,
) -> Result<RecurrenceObligation, SupplyObligationAmountError> {
    obligation.state = obligation
        .state
        .supply_amount(amount_cents)
        .map_err(|_| SupplyObligationAmountError::InvalidInput)?;

    let event_id = ids
        .generate()
        .map_err(|_| SupplyObligationAmountError::IdGenerationFailed)?;
    let occurred_at = clock
        .now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| SupplyObligationAmountError::ClockFailed)?;
    let occurred_at_ms = duration_to_epoch_millis(occurred_at)?;

    store
        .persist_obligation_amount_with_audit(&obligation, event_id, occurred_at_ms)
        .map_err(|_| SupplyObligationAmountError::StoreFailed)?;
    Ok(obligation)
}

fn duration_to_epoch_millis(
    duration: std::time::Duration,
) -> Result<i64, SupplyObligationAmountError> {
    i64::try_from(duration.as_millis()).map_err(|_| SupplyObligationAmountError::ClockFailed)
}

#[cfg(test)]
#[path = "supply_obligation_amount_tests.rs"]
mod tests;
