use super::{DomainError, ObligationState, Result};

impl ObligationState {
    pub fn supply_amount(&self, amount_cents: i64) -> Result<Self> {
        self.validate()?;
        if !matches!(self, Self::AwaitingValue) {
            return Err(DomainError::InvalidInput);
        }
        let state = Self::Payable { amount_cents };
        state.validate()?;
        Ok(state)
    }
}
