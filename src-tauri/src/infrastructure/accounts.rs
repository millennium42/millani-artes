use crate::application::create_account::AccountCreationStore;
use crate::application::list_accounts::AccountReadStore;
use crate::database::{append_audit_event, execute_write_transaction, AuditEvent};
use crate::domain::{Account, AccountType};
use rusqlite::Connection;
use std::fmt;

pub struct AccountStoreError;

impl fmt::Debug for AccountStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("AccountStoreError")
    }
}

impl fmt::Display for AccountStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("account could not be stored")
    }
}

impl std::error::Error for AccountStoreError {}

pub struct AdapterSqliteAccountStore {
    connection: Option<Connection>,
}

impl AdapterSqliteAccountStore {
    pub fn new(connection: Connection) -> Self {
        Self {
            connection: Some(connection),
        }
    }
}

impl AccountCreationStore for AdapterSqliteAccountStore {
    type Error = AccountStoreError;

    fn persist_account_with_audit(
        &mut self,
        account: &Account,
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<(), Self::Error> {
        let connection = self.connection.take().ok_or(AccountStoreError)?;
        let result = execute_write_transaction(connection, |transaction| {
            let account_type = match &account.account_type {
                AccountType::Cash => "cash",
                AccountType::BankPix => "bank_pix",
                AccountType::Card => "card",
                AccountType::Other => "other",
            };
            let inserted = transaction
                .execute(
                    "INSERT INTO main.accounts (id, name, account_type, initial_balance_cents) VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![
                        account.id.as_slice(),
                        &account.name,
                        account_type,
                        account.initial_balance_cents,
                    ],
                )
                .map_err(|_| ())?;
            if inserted != 1 {
                return Err(());
            }
            append_audit_event(
                transaction,
                &AuditEvent {
                    event_id,
                    occurred_at_ms,
                    operation: "account_create",
                    entity: "account",
                    entity_id: account.id,
                },
            )
            .map_err(|_| ())
        });

        match result {
            Ok((connection, ())) => {
                self.connection = Some(connection);
                Ok(())
            }
            Err(_) => Err(AccountStoreError),
        }
    }
}

impl AccountReadStore for AdapterSqliteAccountStore {
    type Error = AccountStoreError;

    fn read_all_accounts(&self) -> Result<Vec<Account>, Self::Error> {
        let connection = self.connection.as_ref().ok_or(AccountStoreError)?;
        let mut stmt = connection
            .prepare("SELECT id, name, account_type, initial_balance_cents FROM main.accounts ORDER BY id")
            .map_err(|_| AccountStoreError)?;
        let rows = stmt
            .query_map([], |row| {
                let id_vec: Vec<u8> = row.get(0)?;
                let id: [u8; 16] = id_vec
                    .try_into()
                    .map_err(|_| rusqlite::Error::InvalidQuery)?;
                let name: String = row.get(1)?;
                let account_type_str: String = row.get(2)?;
                let account_type = match account_type_str.as_str() {
                    "cash" => AccountType::Cash,
                    "bank_pix" => AccountType::BankPix,
                    "card" => AccountType::Card,
                    "other" => AccountType::Other,
                    _ => return Err(rusqlite::Error::InvalidQuery),
                };
                let initial_balance_cents: i64 = row.get(3)?;
                Ok(Account {
                    id,
                    name,
                    account_type,
                    initial_balance_cents,
                })
            })
            .map_err(|_| AccountStoreError)?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|_| AccountStoreError)
    }
}
