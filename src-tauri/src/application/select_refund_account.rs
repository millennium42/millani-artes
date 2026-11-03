use crate::domain::Account;
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum SelectRefundAccountError {
    AccountUnavailable,
}

impl fmt::Display for SelectRefundAccountError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("selected refund account is unavailable")
    }
}

impl std::error::Error for SelectRefundAccountError {}

pub fn select_refund_account(
    account_id: [u8; 16],
    available_accounts: &[Account],
) -> Result<&Account, SelectRefundAccountError> {
    available_accounts
        .iter()
        .find(|account| account.id == account_id)
        .ok_or(SelectRefundAccountError::AccountUnavailable)
}

#[cfg(test)]
#[path = "select_refund_account_tests.rs"]
mod tests;
