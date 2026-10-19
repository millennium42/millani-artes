use crate::application::create_entry::EntryCreationStore;
use crate::application::create_exit::ExitCreationStore;
use crate::application::create_transfer::TransferCreationStore;
use crate::application::edit_movement::MovementEditingStore;
use crate::application::list_movements::MovementReadStore;
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

pub struct TransferStoreError;

impl fmt::Debug for TransferStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("TransferStoreError")
    }
}

impl fmt::Display for TransferStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("transfer could not be stored")
    }
}

impl std::error::Error for TransferStoreError {}

pub struct MovementReadError;

impl fmt::Debug for MovementReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("MovementReadError")
    }
}

impl fmt::Display for MovementReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("movement history could not be read")
    }
}

impl std::error::Error for MovementReadError {}

pub struct MovementEditStoreError;

impl fmt::Debug for MovementEditStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("MovementEditStoreError")
    }
}

impl fmt::Display for MovementEditStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("movement could not be edited")
    }
}

impl std::error::Error for MovementEditStoreError {}

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

impl TransferCreationStore for AdapterSqliteMovementStore {
    type Error = TransferStoreError;

    fn persist_transfer_with_audit(
        &mut self,
        movement: &Movement,
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<(), Self::Error> {
        let (source_account_id, destination_account_id) = match &movement.kind {
            MovementKind::Transfer {
                source_account_id,
                destination_account_id,
            } => (*source_account_id, *destination_account_id),
            _ => return Err(TransferStoreError),
        };
        movement
            .validate_transfer()
            .map_err(|_| TransferStoreError)?;
        let connection = self.connection.take().ok_or(TransferStoreError)?;
        let result = execute_write_transaction(connection, |transaction| {
            let context = match &movement.context {
                Context::Home => "home",
                Context::Factory => "factory",
            };
            let inserted = transaction.execute(
                "INSERT INTO main.movements (id, kind, amount_cents, context, source_account_id, destination_account_id) VALUES (?1, 'transfer', ?2, ?3, ?4, ?5)",
                rusqlite::params![movement.id.as_slice(), movement.amount_cents, context, source_account_id.as_slice(), destination_account_id.as_slice()],
            ).map_err(|_| ())?;
            if inserted != 1 {
                return Err(());
            }
            append_audit_event(
                transaction,
                &AuditEvent {
                    event_id,
                    occurred_at_ms,
                    operation: "movement_transfer_create",
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
            Err(_) => Err(TransferStoreError),
        }
    }
}

impl MovementReadStore for AdapterSqliteMovementStore {
    type Error = MovementReadError;

    fn read_all_movements(&self) -> Result<Vec<Movement>, Self::Error> {
        let connection = self.connection.as_ref().ok_or(MovementReadError)?;
        let mut stmt = connection
            .prepare("SELECT id, kind, amount_cents, context, account_id, source_account_id, destination_account_id FROM main.movements ORDER BY id")
            .map_err(|_| MovementReadError)?;
        let rows = stmt
            .query_map([], |row| {
                let id_vec: Vec<u8> = row.get(0)?;
                let id: [u8; 16] = id_vec
                    .try_into()
                    .map_err(|_| rusqlite::Error::InvalidQuery)?;
                let kind_str: String = row.get(1)?;
                let amount_cents: i64 = row.get(2)?;
                let context_str: String = row.get(3)?;
                let context = Context::try_from(context_str.as_str())
                    .map_err(|_| rusqlite::Error::InvalidQuery)?;

                let kind = match (
                    kind_str.as_str(),
                    row.get::<_, Option<Vec<u8>>>(4)?,
                    row.get::<_, Option<Vec<u8>>>(5)?,
                    row.get::<_, Option<Vec<u8>>>(6)?,
                ) {
                    ("entry", Some(account), None, None) => MovementKind::Entry {
                        account_id: account
                            .try_into()
                            .map_err(|_| rusqlite::Error::InvalidQuery)?,
                    },
                    ("exit", Some(account), None, None) => MovementKind::Exit {
                        account_id: account
                            .try_into()
                            .map_err(|_| rusqlite::Error::InvalidQuery)?,
                    },
                    ("transfer", None, Some(source), Some(destination)) => MovementKind::Transfer {
                        source_account_id: source
                            .try_into()
                            .map_err(|_| rusqlite::Error::InvalidQuery)?,
                        destination_account_id: destination
                            .try_into()
                            .map_err(|_| rusqlite::Error::InvalidQuery)?,
                    },
                    _ => return Err(rusqlite::Error::InvalidQuery),
                };

                let movement = Movement {
                    id,
                    kind,
                    amount_cents,
                    context,
                };

                match &movement.kind {
                    MovementKind::Entry { .. } => movement.validate_entry(),
                    MovementKind::Exit { .. } => movement.validate_exit(),
                    MovementKind::Transfer { .. } => movement.validate_transfer(),
                }
                .map_err(|_| rusqlite::Error::InvalidQuery)?;

                Ok(movement)
            })
            .map_err(|_| MovementReadError)?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|_| MovementReadError)
    }
}

impl MovementEditingStore for AdapterSqliteMovementStore {
    type Error = MovementEditStoreError;

    fn persist_movement_edit_with_audit(
        &mut self,
        movement: &Movement,
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<(), Self::Error> {
        movement
            .result_effect_cents()
            .map_err(|_| MovementEditStoreError)?;
        let (kind_str, account_id, source_account_id, destination_account_id) = match &movement.kind
        {
            MovementKind::Entry { account_id } => {
                ("entry", Some(account_id.as_slice()), None, None)
            }
            MovementKind::Exit { account_id } => ("exit", Some(account_id.as_slice()), None, None),
            MovementKind::Transfer {
                source_account_id,
                destination_account_id,
            } => (
                "transfer",
                None,
                Some(source_account_id.as_slice()),
                Some(destination_account_id.as_slice()),
            ),
        };
        let connection = self.connection.take().ok_or(MovementEditStoreError)?;
        let result = execute_write_transaction(connection, |transaction| {
            let context = match &movement.context {
                Context::Home => "home",
                Context::Factory => "factory",
            };
            let changed = transaction.execute(
                "UPDATE main.movements SET kind=?1, amount_cents=?2, context=?3, account_id=?4, source_account_id=?5, destination_account_id=?6 WHERE id=?7",
                rusqlite::params![kind_str, movement.amount_cents, context, account_id, source_account_id, destination_account_id, movement.id.as_slice()],
            ).map_err(|_| ())?;
            if changed != 1 {
                return Err(());
            }
            append_audit_event(
                transaction,
                &AuditEvent {
                    event_id,
                    occurred_at_ms,
                    operation: "movement_update",
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
            Err(_) => Err(MovementEditStoreError),
        }
    }
}
