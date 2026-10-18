use crate::application::create_category::CategoryCreationStore;
use crate::application::deactivate_category::CategoryDeactivationStore;
use crate::application::rename_category::CategoryRenamingStore;
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

impl CategoryRenamingStore for AdapterSqliteCategoryStore {
    type Error = CategoryStoreError;

    fn rename_category_with_audit(
        &mut self,
        id: [u8; 16],
        name: &str,
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<Option<Category>, Self::Error> {
        let temp_category = Category {
            id,
            name: name.to_string(),
            is_active: true,
        };
        temp_category
            .validate_name()
            .map_err(|_| CategoryStoreError)?;

        let connection = self.connection.take().ok_or(CategoryStoreError)?;
        let result = execute_write_transaction(connection, |transaction| {
            let is_active_result: Result<bool, rusqlite::Error> = transaction.query_row(
                "SELECT is_active FROM main.categories WHERE id = ?1",
                rusqlite::params![id.as_slice()],
                |row| row.get(0),
            );

            let is_active = match is_active_result {
                Ok(active) => active,
                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    return Ok(None);
                }
                Err(_) => {
                    return Err(());
                }
            };

            let updated = transaction
                .execute(
                    "UPDATE main.categories SET name = ?1 WHERE id = ?2",
                    rusqlite::params![name, id.as_slice()],
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
                    operation: "category_rename",
                    entity: "category",
                    entity_id: id,
                },
            )
            .map_err(|_| ())?;

            Ok(Some(Category {
                id,
                name: name.to_string(),
                is_active,
            }))
        });

        match result {
            Ok((connection, category)) => {
                self.connection = Some(connection);
                Ok(category)
            }
            Err(_) => Err(CategoryStoreError),
        }
    }
}

impl CategoryDeactivationStore for AdapterSqliteCategoryStore {
    type Error = CategoryStoreError;

    fn deactivate_category_with_audit(
        &mut self,
        id: [u8; 16],
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<Option<Category>, Self::Error> {
        let connection = self.connection.take().ok_or(CategoryStoreError)?;
        let result = execute_write_transaction(connection, |transaction| {
            let name_result: Result<String, rusqlite::Error> = transaction.query_row(
                "SELECT name FROM main.categories WHERE id = ?1",
                rusqlite::params![id.as_slice()],
                |row| row.get(0),
            );

            let name = match name_result {
                Ok(n) => n,
                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    return Ok(None);
                }
                Err(_) => {
                    return Err(());
                }
            };

            let updated = transaction
                .execute(
                    "UPDATE main.categories SET is_active = 0 WHERE id = ?1",
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
                    operation: "category_deactivate",
                    entity: "category",
                    entity_id: id,
                },
            )
            .map_err(|_| ())?;

            Ok(Some(Category {
                id,
                name,
                is_active: false,
            }))
        });

        match result {
            Ok((connection, category)) => {
                self.connection = Some(connection);
                Ok(category)
            }
            Err(_) => Err(CategoryStoreError),
        }
    }
}
