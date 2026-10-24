use crate::application::create_customer::CustomerCreationStore;
use crate::database::{append_audit_event, execute_write_transaction, AuditEvent};
use crate::domain::Customer;
use rusqlite::Connection;
use std::fmt;

pub struct CustomerStoreError;

impl fmt::Debug for CustomerStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CustomerStoreError")
    }
}

impl fmt::Display for CustomerStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("customer could not be stored")
    }
}

impl std::error::Error for CustomerStoreError {}

pub struct AdapterSqliteCustomerStore {
    connection: Option<Connection>,
}

impl AdapterSqliteCustomerStore {
    pub fn new(connection: Connection) -> Self {
        Self {
            connection: Some(connection),
        }
    }
}

impl CustomerCreationStore for AdapterSqliteCustomerStore {
    type Error = CustomerStoreError;

    fn persist_customer_with_audit(
        &mut self,
        customer: &Customer,
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<(), Self::Error> {
        customer.validate_name().map_err(|_| CustomerStoreError)?;
        customer.validate_phone().map_err(|_| CustomerStoreError)?;
        let connection = self.connection.take().ok_or(CustomerStoreError)?;
        let result = execute_write_transaction(connection, |transaction| {
            let inserted = transaction
                .execute(
                    "INSERT INTO main.customers (id, name, phone) VALUES (?1, ?2, ?3)",
                    rusqlite::params![
                        customer.id.as_slice(),
                        &customer.name,
                        customer.phone.as_deref()
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
                    operation: "customer_create",
                    entity: "customer",
                    entity_id: customer.id,
                },
            )
            .map_err(|_| ())
        });
        match result {
            Ok((connection, ())) => {
                self.connection = Some(connection);
                Ok(())
            }
            Err(_) => Err(CustomerStoreError),
        }
    }
}
