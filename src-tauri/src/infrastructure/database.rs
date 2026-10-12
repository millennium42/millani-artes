use rusqlite::{Connection, OpenFlags};
use std::error::Error;
use std::fmt;
use std::path::Path;

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
}

impl fmt::Display for MigrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::BeginFailed => "migration could not begin",
            Self::ApplyFailed => "migration could not be applied",
            Self::CommitFailed => "migration could not commit",
            Self::RollbackFailed => "migration rollback failed",
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
    Ok(connection)
}

/// Applies reviewed internal SQL without transaction control, ATTACH/DETACH, VACUUM, or external effects.
/// shortcut: static lifetime does not validate SQL; add a parser only if callers admit external scripts.
pub fn execute_migration(
    mut connection: Connection,
    sql: &'static str,
) -> Result<Connection, MigrationError> {
    let transaction = connection
        .transaction()
        .map_err(|_| MigrationError::BeginFailed)?;
    if transaction.execute_batch(sql).is_err() {
        return match transaction.rollback() {
            Ok(()) => Err(MigrationError::ApplyFailed),
            Err(_) => Err(MigrationError::RollbackFailed),
        };
    }
    transaction
        .commit()
        .map_err(|_| MigrationError::CommitFailed)?;
    Ok(connection)
}
