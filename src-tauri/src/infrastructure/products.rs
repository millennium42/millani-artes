use crate::application::create_product::ProductCreationStore;
use crate::database::{append_audit_event, execute_write_transaction, AuditEvent};
use crate::domain::Product;
use rusqlite::Connection;
use std::fmt;

pub struct ProductStoreError;

impl fmt::Debug for ProductStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ProductStoreError")
    }
}

impl fmt::Display for ProductStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("product could not be stored")
    }
}

impl std::error::Error for ProductStoreError {}

pub struct AdapterSqliteProductStore {
    connection: Option<Connection>,
}

impl AdapterSqliteProductStore {
    pub fn new(connection: Connection) -> Self {
        Self {
            connection: Some(connection),
        }
    }
}

impl ProductCreationStore for AdapterSqliteProductStore {
    type Error = ProductStoreError;

    fn persist_product_with_audit(
        &mut self,
        product: &Product,
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<(), Self::Error> {
        product.validate_name().map_err(|_| ProductStoreError)?;
        product
            .validate_suggested_price()
            .map_err(|_| ProductStoreError)?;
        if !product.is_active {
            return Err(ProductStoreError);
        }
        let connection = self.connection.take().ok_or(ProductStoreError)?;
        let result = execute_write_transaction(connection, |transaction| {
            let inserted = transaction
                .execute(
                    "INSERT INTO main.products (id, name, description, suggested_price_cents, is_active) VALUES (?1, ?2, ?3, ?4, 1)",
                    rusqlite::params![product.id.as_slice(), &product.name, &product.description, product.suggested_price_cents],
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
                    operation: "product_create",
                    entity: "product",
                    entity_id: product.id,
                },
            )
            .map_err(|_| ())
        });
        match result {
            Ok((connection, ())) => {
                self.connection = Some(connection);
                Ok(())
            }
            Err(_) => Err(ProductStoreError),
        }
    }
}
