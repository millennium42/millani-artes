use rusqlite::OptionalExtension;
use rusqlite::{Connection, OpenFlags, Transaction, TransactionBehavior};
use std::error::Error;
use std::fmt;
use std::path::Path;

#[derive(PartialEq, Eq)]
pub enum TransactionError<E> {
    BeginFailed,
    OperationFailed(E),
    CommitFailed,
    RollbackFailed,
}

impl<E> fmt::Debug for TransactionError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::BeginFailed => "BeginFailed",
            Self::OperationFailed(_) => "OperationFailed",
            Self::CommitFailed => "CommitFailed",
            Self::RollbackFailed => "RollbackFailed",
        })
    }
}

impl<E> fmt::Display for TransactionError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::BeginFailed => "write transaction could not begin",
            Self::OperationFailed(_) => "write transaction operation failed",
            Self::CommitFailed => "write transaction could not commit",
            Self::RollbackFailed => "write transaction rollback failed",
        })
    }
}

impl<E> Error for TransactionError<E> {}

/// Runs one write unit. The trusted callback must not control transactions, attach databases,
/// vacuum, or perform external effects; this API does not promise recovery after commit failure.
pub fn execute_write_transaction<T, E>(
    connection: Connection,
    operation: impl FnOnce(&Transaction<'_>) -> Result<T, E>,
) -> Result<(Connection, T), TransactionError<E>> {
    execute_transaction_with_behavior(connection, TransactionBehavior::Immediate, operation)
}

#[derive(Debug, PartialEq, Eq)]
pub enum DatabaseOpenError {
    InvalidPath,
    OpenFailed,
}

impl fmt::Display for DatabaseOpenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidPath => "database path is invalid",
            Self::OpenFailed => "database could not be opened",
        })
    }
}

impl Error for DatabaseOpenError {}

#[derive(Debug, PartialEq, Eq)]
pub enum MigrationError {
    BeginFailed,
    ApplyFailed,
    CommitFailed,
    RollbackFailed,
    InvalidVersion,
    InvalidSchemaState,
}

impl fmt::Display for MigrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::BeginFailed => "migration could not begin",
            Self::ApplyFailed => "migration could not be applied",
            Self::CommitFailed => "migration could not commit",
            Self::RollbackFailed => "migration rollback failed",
            Self::InvalidVersion => "migration version is invalid",
            Self::InvalidSchemaState => "database schema state is invalid",
        })
    }
}

impl Error for MigrationError {}

pub fn open_database(path: &Path) -> Result<Connection, DatabaseOpenError> {
    // shortcut: caller owns the parent directory; enforce app-data containment before accepting UI paths.
    if !path.is_absolute() {
        return Err(DatabaseOpenError::InvalidPath);
    }
    let connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
    )
    .map_err(|_| DatabaseOpenError::OpenFailed)?;
    connection
        .query_row("SELECT count(*) FROM sqlite_schema", [], |row| {
            row.get::<_, i64>(0)
        })
        .map_err(|_| DatabaseOpenError::OpenFailed)?;
    connection
        .pragma_update(None, "foreign_keys", true)
        .map_err(|_| DatabaseOpenError::OpenFailed)?;
    let foreign_keys: i64 = connection
        .pragma_query_value(None, "foreign_keys", |row| row.get(0))
        .map_err(|_| DatabaseOpenError::OpenFailed)?;
    if foreign_keys != 1 {
        return Err(DatabaseOpenError::OpenFailed);
    }
    Ok(connection)
}

/// Applies reviewed internal SQL without transaction control, ATTACH/DETACH, VACUUM, or external effects.
/// shortcut: static lifetime does not validate SQL; add a parser only if callers admit external scripts.
pub fn execute_migration(
    connection: Connection,
    sql: &'static str,
) -> Result<Connection, MigrationError> {
    execute_transaction(connection, |transaction| {
        transaction
            .execute_batch(sql)
            .map_err(|_| MigrationError::ApplyFailed)
    })
}

fn execute_transaction(
    connection: Connection,
    apply: impl FnOnce(&Transaction<'_>) -> Result<(), MigrationError>,
) -> Result<Connection, MigrationError> {
    execute_transaction_with_behavior(connection, TransactionBehavior::Deferred, apply)
        .map(|(connection, ())| connection)
        .map_err(|error| match error {
            TransactionError::BeginFailed => MigrationError::BeginFailed,
            TransactionError::OperationFailed(error) => error,
            TransactionError::CommitFailed => MigrationError::CommitFailed,
            TransactionError::RollbackFailed => MigrationError::RollbackFailed,
        })
}

fn execute_transaction_with_behavior<T, E>(
    mut connection: Connection,
    behavior: TransactionBehavior,
    operation: impl FnOnce(&Transaction<'_>) -> Result<T, E>,
) -> Result<(Connection, T), TransactionError<E>> {
    let transaction = connection
        .transaction_with_behavior(behavior)
        .map_err(|_| TransactionError::BeginFailed)?;
    let value = match operation(&transaction) {
        Ok(value) => value,
        Err(error) => {
            return match transaction.rollback() {
                Ok(()) => Err(TransactionError::OperationFailed(error)),
                Err(_) => Err(TransactionError::RollbackFailed),
            };
        }
    };
    transaction
        .commit()
        .map_err(|_| TransactionError::CommitFailed)?;
    Ok((connection, value))
}

/// Reads the main database's contiguous migration ledger up to the caller's compiled version.
pub fn schema_version(
    connection: &Connection,
    supported_version: u32,
) -> Result<u32, MigrationError> {
    let object_type = connection
        .query_row(
            "SELECT type FROM main.sqlite_schema WHERE name = 'schema_migration' AND type IN ('table', 'view')",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|_| MigrationError::InvalidSchemaState)?;
    let Some(object_type) = object_type else {
        return if has_other_user_objects(connection)? {
            Err(MigrationError::InvalidSchemaState)
        } else {
            Ok(0)
        };
    };
    if object_type != "table" {
        return Err(MigrationError::InvalidSchemaState);
    }

    let mut columns = connection
        .prepare("PRAGMA main.table_xinfo('schema_migration')")
        .map_err(|_| MigrationError::InvalidSchemaState)?;
    let mut column_rows = columns
        .query([])
        .map_err(|_| MigrationError::InvalidSchemaState)?;
    let column = column_rows
        .next()
        .map_err(|_| MigrationError::InvalidSchemaState)?
        .ok_or(MigrationError::InvalidSchemaState)?;
    let name = column
        .get::<_, String>(1)
        .map_err(|_| MigrationError::InvalidSchemaState)?;
    let declared_type = column
        .get::<_, String>(2)
        .map_err(|_| MigrationError::InvalidSchemaState)?;
    let not_null = column
        .get::<_, i64>(3)
        .map_err(|_| MigrationError::InvalidSchemaState)?;
    let primary_key = column
        .get::<_, i64>(5)
        .map_err(|_| MigrationError::InvalidSchemaState)?;
    let hidden = column
        .get::<_, i64>(6)
        .map_err(|_| MigrationError::InvalidSchemaState)?;
    if name != "version"
        || !declared_type.eq_ignore_ascii_case("INTEGER")
        || not_null != 1
        || primary_key != 1
        || hidden != 0
        || column_rows
            .next()
            .map_err(|_| MigrationError::InvalidSchemaState)?
            .is_some()
    {
        return Err(MigrationError::InvalidSchemaState);
    }
    drop(column_rows);
    drop(columns);

    let mut statement = connection
        .prepare("SELECT version, typeof(version) FROM main.schema_migration ORDER BY version")
        .map_err(|_| MigrationError::InvalidSchemaState)?;
    let mut rows = statement
        .query([])
        .map_err(|_| MigrationError::InvalidSchemaState)?;
    let mut previous = None;
    while let Some(row) = rows
        .next()
        .map_err(|_| MigrationError::InvalidSchemaState)?
    {
        let storage_class = row
            .get::<_, String>(1)
            .map_err(|_| MigrationError::InvalidSchemaState)?;
        if storage_class != "integer" {
            return Err(MigrationError::InvalidSchemaState);
        }
        let raw_version = row
            .get::<_, i64>(0)
            .map_err(|_| MigrationError::InvalidSchemaState)?;
        let version = u32::try_from(raw_version).map_err(|_| MigrationError::InvalidSchemaState)?;
        if version == 0
            || version > supported_version
            || previous.map_or(version != 1, |last: u32| {
                last.checked_add(1) != Some(version)
            })
        {
            return Err(MigrationError::InvalidSchemaState);
        }
        previous = Some(version);
    }
    drop(rows);
    drop(statement);

    let Some(current) = previous else {
        return if has_other_user_objects(connection)? {
            Err(MigrationError::InvalidSchemaState)
        } else {
            Ok(0)
        };
    };
    Ok(current)
}

/// Applies trusted, reviewed SQL for exactly the next version and records it atomically.
pub fn execute_versioned_migration(
    connection: Connection,
    version: u32,
    sql: &'static str,
) -> Result<Connection, MigrationError> {
    if version == 0 {
        return Err(MigrationError::InvalidVersion);
    }
    execute_transaction(connection, |transaction| {
        let current = schema_version(transaction, version)?;
        if current.checked_add(1) != Some(version) {
            return Err(MigrationError::InvalidVersion);
        }
        transaction
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS main.schema_migration (version INTEGER PRIMARY KEY CHECK (version > 0) NOT NULL)",
            )
            .map_err(|_| MigrationError::ApplyFailed)?;
        transaction
            .execute_batch(sql)
            .map_err(|_| MigrationError::ApplyFailed)?;
        transaction
            .execute(
                "INSERT INTO main.schema_migration(version) VALUES (?1)",
                [version],
            )
            .map_err(|_| MigrationError::ApplyFailed)?;
        Ok(())
    })
}

fn has_other_user_objects(connection: &Connection) -> Result<bool, MigrationError> {
    connection
        .query_row(
            "SELECT EXISTS (SELECT 1 FROM main.sqlite_schema WHERE name NOT GLOB 'sqlite_*' AND NOT (type = 'table' AND name = 'schema_migration'))",
            [],
            |row| row.get(0),
        )
        .map_err(|_| MigrationError::InvalidSchemaState)
}

pub const AUDIT_EVENT_MIGRATION_VERSION: u32 = 2;
pub const AUDIT_EVENT_MIGRATION_SQL: &str = include_str!("../../migrations/002-audit-event.sql");
pub const ACCOUNTS_MIGRATION_VERSION: u32 = 3;
pub const ACCOUNTS_MIGRATION_SQL: &str = include_str!("../../migrations/003-accounts.sql");

pub struct AuditEvent {
    pub event_id: [u8; 16],
    pub occurred_at_ms: i64,
    pub operation: &'static str,
    pub entity: &'static str,
    pub entity_id: [u8; 16],
}

#[derive(PartialEq, Eq)]
pub enum AuditEventError {
    InvalidEvent,
    InsertFailed,
}

impl fmt::Debug for AuditEventError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidEvent => "InvalidEvent",
            Self::InsertFailed => "InsertFailed",
        })
    }
}

impl fmt::Display for AuditEventError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidEvent => "audit event is invalid",
            Self::InsertFailed => "audit event could not be inserted",
        })
    }
}

impl Error for AuditEventError {}

pub fn append_audit_event(
    transaction: &Transaction<'_>,
    event: &AuditEvent,
) -> Result<(), AuditEventError> {
    if event.event_id == [0; 16]
        || event.entity_id == [0; 16]
        || event.occurred_at_ms < 0
        || !valid_audit_tag(event.operation)
        || !valid_audit_tag(event.entity)
    {
        return Err(AuditEventError::InvalidEvent);
    }

    match transaction.execute(
            "INSERT INTO main.audit_event (event_id, occurred_at_ms, operation, entity, entity_id) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                event.event_id.as_slice(),
                event.occurred_at_ms,
                event.operation,
                event.entity,
                event.entity_id.as_slice(),
            ],
        ) {
        Ok(1) => Ok(()),
        Ok(_) | Err(_) => Err(AuditEventError::InsertFailed),
    }
}

fn valid_audit_tag(tag: &str) -> bool {
    let bytes = tag.as_bytes();
    !bytes.is_empty()
        && bytes.len() <= 64
        && bytes[0].is_ascii_lowercase()
        && bytes[1..]
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_')
}
