use super::{ports::Clock, DomainError, ObligationState, Result};
use std::time::UNIX_EPOCH;

impl ObligationState {
    pub fn mark_overdue_ms(&self, due_at_ms: i64, clock: &dyn Clock) -> Result<Self> {
        self.validate()?;

        let amount = match self {
            ObligationState::Payable { amount_cents } => *amount_cents,
            _ => return Err(DomainError::InvalidInput),
        };

        let now_ns = match clock.now().duration_since(UNIX_EPOCH) {
            Ok(d) => d.as_nanos() as i128,
            Err(e) => -(e.duration().as_nanos() as i128),
        };

        let deadline_ns = i128::from(due_at_ms) * 1_000_000;

        if now_ns <= deadline_ns {
            return Err(DomainError::InvalidInput);
        }

        Ok(ObligationState::Overdue {
            amount_cents: amount,
        })
    }
}
