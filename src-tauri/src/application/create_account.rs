use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::{Account, AccountType};
use std::fmt;
use std::time::UNIX_EPOCH;

pub trait AccountCreationStore {
    type Error;

    fn persist_account_with_audit(
        &mut self,
        account: &Account,
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<(), Self::Error>;
}

#[derive(PartialEq, Eq)]
pub enum CreateAccountError {
    InvalidName,
    IdGenerationFailed,
    ClockFailed,
    StoreFailed,
}

impl fmt::Debug for CreateAccountError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidName => "InvalidName",
            Self::IdGenerationFailed => "IdGenerationFailed",
            Self::ClockFailed => "ClockFailed",
            Self::StoreFailed => "StoreFailed",
        })
    }
}

impl fmt::Display for CreateAccountError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidName => "account name is invalid",
            Self::IdGenerationFailed => "account identifiers could not be generated",
            Self::ClockFailed => "account timestamp is invalid",
            Self::StoreFailed => "account could not be stored",
        })
    }
}

impl std::error::Error for CreateAccountError {}

pub fn create_account<S: AccountCreationStore, I: UuidGenerator, C: Clock>(
    name: String,
    account_type: AccountType,
    initial_balance_cents: i64,
    ids: &mut I,
    clock: &C,
    store: &mut S,
) -> Result<Account, CreateAccountError> {
    let mut account = Account {
        id: [0; 16],
        name,
        account_type,
        initial_balance_cents,
    };
    account
        .validate_name()
        .map_err(|_| CreateAccountError::InvalidName)?;

    account.id = ids
        .generate()
        .map_err(|_| CreateAccountError::IdGenerationFailed)?;
    let event_id = ids
        .generate()
        .map_err(|_| CreateAccountError::IdGenerationFailed)?;
    let occurred_at = clock
        .now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| CreateAccountError::ClockFailed)?;
    let occurred_at_ms = duration_to_epoch_millis(occurred_at)?;

    store
        .persist_account_with_audit(&account, event_id, occurred_at_ms)
        .map_err(|_| CreateAccountError::StoreFailed)?;
    Ok(account)
}

fn duration_to_epoch_millis(duration: std::time::Duration) -> Result<i64, CreateAccountError> {
    i64::try_from(duration.as_millis()).map_err(|_| CreateAccountError::ClockFailed)
}

#[cfg(test)]
#[path = "create_account_tests.rs"]
mod tests;
