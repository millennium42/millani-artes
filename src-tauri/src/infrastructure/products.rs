use crate::application::create_product::ProductCreationStore;
use crate::application::deactivate_product::ProductDeactivationStore;
use crate::application::edit_product::ProductEditingStore;
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

impl ProductEditingStore for AdapterSqliteProductStore {
    type Error = ProductStoreError;

    fn edit_product_with_audit(
        &mut self,
        id: [u8; 16],
        name: &str,
        description: &str,
        suggested_price_cents: i64,
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<Option<Product>, Self::Error> {
        let temp_product = Product {
            id,
            name: name.to_string(),
            description: description.to_string(),
            suggested_price_cents,
            is_active: true,
        };
        temp_product
            .validate_name()
            .map_err(|_| ProductStoreError)?;
        temp_product
            .validate_suggested_price()
            .map_err(|_| ProductStoreError)?;

        let connection = self.connection.take().ok_or(ProductStoreError)?;
        let result = execute_write_transaction(connection, |transaction| {
            let is_active_result: Result<bool, rusqlite::Error> = transaction.query_row(
                "SELECT is_active FROM main.products WHERE id = ?1",
                rusqlite::params![id.as_slice()],
                |row| row.get(0),
            );
            let is_active = match is_active_result {
                Ok(active) => active,
                Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
                Err(_) => return Err(()),
            };
            let updated = transaction
                .execute(
                    "UPDATE main.products SET name = ?1, description = ?2, suggested_price_cents = ?3 WHERE id = ?4",
                    rusqlite::params![name, description, suggested_price_cents, id.as_slice()],
                )
                .map_err(|_| ())?;
            if updated != 1 {
                return Err(());
            }
            append_audit_event(
                transaction,
                &AuditEvent {
                    event_id,
                    occurred_at_ms,
                    operation: "product_edit",
                    entity: "product",
                    entity_id: id,
                },
            )
            .map_err(|_| ())?;
            Ok(Some(Product {
                id,
                name: name.to_string(),
                description: description.to_string(),
                suggested_price_cents,
                is_active,
            }))
        });
        match result {
            Ok((connection, product)) => {
                self.connection = Some(connection);
                Ok(product)
            }
            Err(_) => Err(ProductStoreError),
        }
    }
}

impl ProductDeactivationStore for AdapterSqliteProductStore {
    type Error = ProductStoreError;
    fn deactivate_product_with_audit(
        &mut self,
        id: [u8; 16],
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<Option<Product>, Self::Error> {
        let connection = self.connection.take().ok_or(ProductStoreError)?;
        let result = execute_write_transaction(connection, |transaction| {
            let product_result: Result<(String, String, i64), rusqlite::Error> = transaction
                .query_row(
                "SELECT name, description, suggested_price_cents FROM main.products WHERE id = ?1",
                rusqlite::params![id.as_slice()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            );

            let (name, description, suggested_price_cents) = match product_result {
                Ok(p) => p,
                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    return Ok(None);
                }
                Err(_) => {
                    return Err(());
                }
            };

            let updated = transaction
                .execute(
                    "UPDATE main.products SET is_active = 0 WHERE id = ?1",
                    rusqlite::params![id.as_slice()],
                )
                .map_err(|_| ())?;

            if updated != 1 {
                return Err(());
            }

            append_audit_event(
                transaction,
                &AuditEvent {
                    event_id,
                    occurred_at_ms,
                    operation: "product_deactivate",
                    entity: "product",
                    entity_id: id,
                },
            )
            .map_err(|_| ())?;

            Ok(Some(Product {
                id,
                name,
                description,
                suggested_price_cents,
                is_active: false,
            }))
        });

        match result {
            Ok((connection, product)) => {
                self.connection = Some(connection);
                Ok(product)
            }
            Err(_) => Err(ProductStoreError),
        }
    }
}
