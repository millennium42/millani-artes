use crate::domain::Account;

#[derive(Debug)]
pub struct ListAccountsError;

impl std::fmt::Display for ListAccountsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "accounts could not be read")
    }
}

impl std::error::Error for ListAccountsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub trait AccountReadStore {
    type Error;
    fn read_all_accounts(&self) -> Result<Vec<Account>, Self::Error>;
}

pub fn list_accounts(store: &impl AccountReadStore) -> Result<Vec<Account>, ListAccountsError> {
    store.read_all_accounts().map_err(|_| ListAccountsError)
}

#[cfg(test)]
#[path = "list_accounts_tests.rs"]
mod tests;
