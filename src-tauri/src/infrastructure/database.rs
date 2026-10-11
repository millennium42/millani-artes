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
