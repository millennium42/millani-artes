use crate::application::create_entry::EntryCreationStore;
use crate::application::create_exit::ExitCreationStore;
use crate::database::{append_audit_event, execute_write_transaction, AuditEvent};
use crate::domain::{Context, Movement, MovementKind};
use rusqlite::Connection;
use std::fmt;

pub struct MovementStoreError;

impl fmt::Debug for MovementStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("MovementStoreError")
    }
}

impl fmt::Display for MovementStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("entry could not be stored")
    }
}

impl std::error::Error for MovementStoreError {}

pub struct ExitStoreError;

impl fmt::Debug for ExitStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ExitStoreError")
    }
}

impl fmt::Display for ExitStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("exit could not be stored")
    }
}

impl std::error::Error for ExitStoreError {}

pub struct AdapterSqliteMovementStore {
    connection: Option<Connection>,
}

impl AdapterSqliteMovementStore {
    pub fn new(connection: Connection) -> Self {
        Self {
            connection: Some(connection),
        }
    }
}

impl EntryCreationStore for AdapterSqliteMovementStore {
    type Error = MovementStoreError;

    fn persist_entry_with_audit(
        &mut self,
        movement: &Movement,
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<(), Self::Error> {
        let account_id = match &movement.kind {
            MovementKind::Entry { account_id } => account_id,
            _ => return Err(MovementStoreError),
        };
        movement.validate_entry().map_err(|_| MovementStoreError)?;
        let connection = self.connection.take().ok_or(MovementStoreError)?;
        let result = execute_write_transaction(connection, |transaction| {
            let context = match &movement.context {
                Context::Home => "home",
                Context::Factory => "factory",
            };
            let inserted = transaction.execute(
                "INSERT INTO main.movements (id, kind, amount_cents, context, account_id) VALUES (?1, 'entry', ?2, ?3, ?4)",
                rusqlite::params![movement.id.as_slice(), movement.amount_cents, context, account_id.as_slice()],
            ).map_err(|_| ())?;
            if inserted != 1 {
                return Err(());
            }
            append_audit_event(
                transaction,
                &AuditEvent {
                    event_id,
                    occurred_at_ms,
                    operation: "movement_entry_create",
                    entity: "movement",
                    entity_id: movement.id,
                },
            )
            .map_err(|_| ())
        });
        match result {
            Ok((connection, ())) => {
                self.connection = Some(connection);
                Ok(())
            }
            Err(_) => Err(MovementStoreError),
        }
    }
}

impl ExitCreationStore for AdapterSqliteMovementStore {
    type Error = ExitStoreError;

    fn persist_exit_with_audit(
        &mut self,
        movement: &Movement,
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<(), Self::Error> {
        let account_id = match &movement.kind {
            MovementKind::Exit { account_id } => account_id,
            _ => return Err(ExitStoreError),
        };
        movement.validate_exit().map_err(|_| ExitStoreError)?;
        let connection = self.connection.take().ok_or(ExitStoreError)?;
        let result = execute_write_transaction(connection, |transaction| {
            let context = match &movement.context {
                Context::Home => "home",
                Context::Factory => "factory",
            };
            let inserted = transaction.execute(
                "INSERT INTO main.movements (id, kind, amount_cents, context, account_id) VALUES (?1, 'exit', ?2, ?3, ?4)",
                rusqlite::params![movement.id.as_slice(), movement.amount_cents, context, account_id.as_slice()],
            ).map_err(|_| ())?;
            if inserted != 1 {
                return Err(());
            }
            append_audit_event(
                transaction,
                &AuditEvent {
                    event_id,
                    occurred_at_ms,
                    operation: "movement_exit_create",
                    entity: "movement",
                    entity_id: movement.id,
                },
            )
            .map_err(|_| ())
        });
        match result {
            Ok((connection, ())) => {
                self.connection = Some(connection);
                Ok(())
            }
            Err(_) => Err(ExitStoreError),
        }
    }
}
