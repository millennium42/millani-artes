use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::{Context, Movement, MovementKind, ObligationState, RecurrenceObligation};
use std::fmt;
use std::time::UNIX_EPOCH;

pub trait RecurrencePaymentStore {
    type Error;

    fn persist_recurrence_payment_with_audit(
        &mut self,
        obligation: &RecurrenceObligation,
        movement: &Movement,
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<(), Self::Error>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum PayRecurrenceError {
    InvalidInput,
    IdGenerationFailed,
    ClockFailed,
    StoreFailed,
}

impl fmt::Display for PayRecurrenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidInput => "payment is invalid",
            Self::IdGenerationFailed => "payment identifiers could not be generated",
            Self::ClockFailed => "payment timestamp is invalid",
            Self::StoreFailed => "payment could not be stored",
        })
    }
}

impl std::error::Error for PayRecurrenceError {}

pub fn pay_recurrence<S: RecurrencePaymentStore, I: UuidGenerator, C: Clock>(
    mut obligation: RecurrenceObligation,
    account_id: [u8; 16],
    ids: &mut I,
    clock: &C,
    store: &mut S,
) -> Result<RecurrenceObligation, PayRecurrenceError> {
    obligation
        .state
        .validate()
        .map_err(|_| PayRecurrenceError::InvalidInput)?;

    let amount = match &obligation.state {
        ObligationState::Payable { amount_cents } | ObligationState::Overdue { amount_cents } => {
            *amount_cents
        }
        _ => return Err(PayRecurrenceError::InvalidInput),
    };

    let mut payment = Movement {
        id: [0; 16],
        kind: MovementKind::Exit { account_id },
        amount_cents: amount,
        context: match &obligation.context {
            crate::domain::Context::Home => Context::Home,
            crate::domain::Context::Factory => Context::Factory,
        },
    };

    payment
        .validate_exit()
        .map_err(|_| PayRecurrenceError::InvalidInput)?;

    payment.id = ids
        .generate()
        .map_err(|_| PayRecurrenceError::IdGenerationFailed)?;

    obligation.state = obligation
        .state
        .mark_paid(&payment)
        .map_err(|_| PayRecurrenceError::InvalidInput)?;

    let event_id = ids
        .generate()
        .map_err(|_| PayRecurrenceError::IdGenerationFailed)?;

    let occurred_at = clock
        .now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| PayRecurrenceError::ClockFailed)?;
    let occurred_at_ms = duration_to_epoch_millis(occurred_at)?;

    store
        .persist_recurrence_payment_with_audit(&obligation, &payment, event_id, occurred_at_ms)
        .map_err(|_| PayRecurrenceError::StoreFailed)?;

    Ok(obligation)
}

fn duration_to_epoch_millis(duration: std::time::Duration) -> Result<i64, PayRecurrenceError> {
    i64::try_from(duration.as_millis()).map_err(|_| PayRecurrenceError::ClockFailed)
}

#[cfg(test)]
#[path = "pay_recurrence_tests.rs"]
mod tests;
