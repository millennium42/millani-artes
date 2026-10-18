use crate::application::create_category::CategoryCreationStore;
use crate::database::{append_audit_event, execute_write_transaction, AuditEvent};
use crate::domain::Category;
use rusqlite::Connection;
use std::fmt;

pub struct CategoryStoreError;

impl fmt::Debug for CategoryStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CategoryStoreError")
    }
}

impl fmt::Display for CategoryStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("category could not be stored")
    }
}

impl std::error::Error for CategoryStoreError {}

pub struct AdapterSqliteCategoryStore {
    connection: Option<Connection>,
}

impl AdapterSqliteCategoryStore {
    pub fn new(connection: Connection) -> Self {
        Self {
            connection: Some(connection),
        }
    }
}

impl CategoryCreationStore for AdapterSqliteCategoryStore {
    type Error = CategoryStoreError;

    fn persist_category_with_audit(
        &mut self,
        category: &Category,
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<(), Self::Error> {
        category.validate_name().map_err(|_| CategoryStoreError)?;
        if !category.is_active {
            return Err(CategoryStoreError);
        }
        let connection = self.connection.take().ok_or(CategoryStoreError)?;
        let result = execute_write_transaction(connection, |transaction| {
            let inserted = transaction
                .execute(
                    "INSERT INTO main.categories (id, name, is_active) VALUES (?1, ?2, 1)",
                    rusqlite::params![category.id.as_slice(), &category.name],
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
                    operation: "category_create",
                    entity: "category",
                    entity_id: category.id,
                },
            )
            .map_err(|_| ())
        });
        match result {
            Ok((connection, ())) => {
                self.connection = Some(connection);
                Ok(())
            }
            Err(_) => Err(CategoryStoreError),
        }
    }
}
