use crate::application::create_account::AccountCreationStore;
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
