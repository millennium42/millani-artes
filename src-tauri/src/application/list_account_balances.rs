use crate::application::{
    list_accounts::{list_accounts, AccountReadStore},
    list_movements::{list_movements, MovementReadStore},
};
use crate::domain::Account;

#[derive(Debug)]
pub struct ListAccountBalancesError;

impl std::fmt::Display for ListAccountBalancesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "account balances could not be listed")
    }
}

impl std::error::Error for ListAccountBalancesError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub struct AccountBalance {
    pub account: Account,
    pub balance_cents: i64,
}

pub fn list_account_balances(
    account_store: &impl AccountReadStore,
    movement_store: &impl MovementReadStore,
) -> Result<Vec<AccountBalance>, ListAccountBalancesError> {
    let accounts = list_accounts(account_store).map_err(|_| ListAccountBalancesError)?;
    let movements = list_movements(movement_store).map_err(|_| ListAccountBalancesError)?;

    accounts
        .into_iter()
        .map(|account| {
            let balance_cents = account
                .balance_cents(&movements)
                .map_err(|_| ListAccountBalancesError)?;
            Ok(AccountBalance {
                account,
                balance_cents,
            })
        })
        .collect()
}

#[cfg(test)]
#[path = "list_account_balances_tests.rs"]
mod tests;
