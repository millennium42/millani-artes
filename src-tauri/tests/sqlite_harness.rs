mod common;

use rusqlite::Connection;
use std::error::Error;
use std::fs;
use std::path::Path;

use millani_artes::database::{
    execute_migration, open_database, DatabaseOpenError, MigrationError,
};

fn schema_object_count(connection: &Connection) -> rusqlite::Result<i64> {
    connection.query_row("SELECT count(*) FROM sqlite_schema", [], |row| row.get(0))
}

#[test]
fn connections_start_clean_and_are_isolated() {
    let first = common::open_clean_database().expect("FIRST_CONNECTION_FAILED");
    let sqlite_version: String = first
        .query_row("SELECT sqlite_version()", [], |row| row.get(0))
        .expect("SQLITE_VERSION_QUERY_FAILED");
    println!("SQLITE_TEST_VERSION={sqlite_version}");
    assert_eq!(schema_object_count(&first).unwrap(), 0);

    first
        .execute_batch("CREATE TABLE harness_synthetic (value TEXT NOT NULL)")
        .unwrap();
    let expected = "O'Brien";
    first
        .execute(
            "INSERT INTO harness_synthetic (value) VALUES (?1)",
            [expected],
        )
        .unwrap();
    let actual: String = first
        .query_row("SELECT value FROM harness_synthetic", [], |row| row.get(0))
        .unwrap();
    assert_eq!(actual, expected);

    let second = common::open_clean_database().expect("SECOND_CONNECTION_FAILED");
    assert_eq!(schema_object_count(&second).unwrap(), 0);
    drop(first);

    let after_close = common::open_clean_database().expect("NEW_CONNECTION_FAILED");
    assert_eq!(schema_object_count(&after_close).unwrap(), 0);
}

#[test]
fn file_database_is_empty_usable_and_persists_after_reopen() {
    let directory = common::TempDirectory::new();
    let path = directory.path().join("first.sqlite");
    let connection = open_database(&path).expect("DATABASE_OPEN_FAILED");
    assert!(path.is_file());
    assert_eq!(schema_object_count(&connection).unwrap(), 0);
    drop(connection);

    let empty_reopened = open_database(&path).expect("EMPTY_DATABASE_REOPEN_FAILED");
    assert_eq!(schema_object_count(&empty_reopened).unwrap(), 0);
    empty_reopened
        .execute_batch("CREATE TABLE harness_fixture (value TEXT NOT NULL)")
        .unwrap();
    let expected = "O'Brien";
    empty_reopened
        .execute(
            "INSERT INTO harness_fixture (value) VALUES (?1)",
            [expected],
        )
        .unwrap();
    drop(empty_reopened);

    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    let actual: String = reopened
        .query_row("SELECT value FROM harness_fixture", [], |row| row.get(0))
        .unwrap();
    assert_eq!(actual, expected);
    let second_path = directory.path().join("second.sqlite");
    let second = open_database(&second_path).expect("SECOND_DATABASE_OPEN_FAILED");
    assert_eq!(schema_object_count(&second).unwrap(), 0);
}

#[test]
fn migration_executor_applies_empty_and_upgrade_migrations() {
    let directory = common::TempDirectory::new();
    let path = directory.path().join("migration.sqlite");
    let connection = open_database(&path).expect("DATABASE_OPEN_FAILED");
    assert!(connection.is_autocommit());
    assert_eq!(schema_object_count(&connection).unwrap(), 0);
    let tracking_count: i64 = connection
        .query_row(
            "SELECT count(*) FROM sqlite_schema WHERE name = 'schema_migration'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(tracking_count, 0);

    let connection = execute_migration(
        connection,
        "CREATE TABLE harness_migration_fixture (value TEXT NOT NULL)",
    )
    .expect("INITIAL_MIGRATION_FAILED");
    assert!(connection.is_autocommit());
    assert_eq!(schema_object_count(&connection).unwrap(), 1);
    let expected = "O'Brien";
    connection
        .execute(
            "INSERT INTO harness_migration_fixture (value) VALUES (?1)",
            [expected],
        )
        .unwrap();
    drop(connection);

    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    let upgraded = execute_migration(
        reopened,
        "ALTER TABLE harness_migration_fixture ADD COLUMN upgrade_marker TEXT NOT NULL DEFAULT 'upgrade'",
    )
    .expect("UPGRADE_MIGRATION_FAILED");
    assert!(upgraded.is_autocommit());
    let (actual, marker): (String, String) = upgraded
        .query_row(
            "SELECT value, upgrade_marker FROM harness_migration_fixture",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(actual, expected);
    assert_eq!(marker, "upgrade");
    drop(upgraded);

    let reopened_upgrade = open_database(&path).expect("UPGRADED_DATABASE_REOPEN_FAILED");
    let (actual, marker): (String, String) = reopened_upgrade
        .query_row(
            "SELECT value, upgrade_marker FROM harness_migration_fixture",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(actual, expected);
    assert_eq!(marker, "upgrade");
    let tracking_count: i64 = reopened_upgrade
        .query_row(
            "SELECT count(*) FROM sqlite_schema WHERE name = 'schema_migration'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(tracking_count, 0);
}

#[test]
fn migration_executor_accepts_empty_sql_without_changes() {
    let directory = common::TempDirectory::new();
    let path = directory.path().join("empty-migration.sqlite");
    let connection = open_database(&path).expect("DATABASE_OPEN_FAILED");
    let unchanged = execute_migration(connection, "").expect("EMPTY_MIGRATION_FAILED");
    assert!(unchanged.is_autocommit());
    assert_eq!(schema_object_count(&unchanged).unwrap(), 0);
    drop(unchanged);

    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert_eq!(schema_object_count(&reopened).unwrap(), 0);
}

#[test]
fn migration_executor_rolls_back_ddl_and_dml_after_apply_failure() {
    let directory = common::TempDirectory::new();
    let path = directory.path().join("apply-failure.sqlite");
    let setup = open_database(&path).expect("DATABASE_OPEN_FAILED");
    setup
        .execute_batch("CREATE TABLE existing_fixture (value TEXT NOT NULL)")
        .unwrap();
    setup
        .execute("INSERT INTO existing_fixture VALUES ('before')", [])
        .unwrap();
    drop(setup);

    let connection = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    let error = execute_migration(
        connection,
        "CREATE TABLE transient_fixture (value TEXT); INSERT INTO existing_fixture VALUES ('during'); INSERT INTO missing_fixture VALUES ('fail');",
    )
    .expect_err("FAILING_MIGRATION_SUCCEEDED");
    assert_eq!(error, MigrationError::ApplyFailed);

    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert_eq!(schema_object_count(&reopened).unwrap(), 1);
    let values: Vec<String> = {
        let mut statement = reopened
            .prepare("SELECT value FROM existing_fixture ORDER BY rowid")
            .unwrap();
        statement
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(values, ["before"]);
}

#[test]
fn migration_executor_rejects_an_existing_outer_transaction() {
    let directory = common::TempDirectory::new();
    let path = directory.path().join("begin-failure.sqlite");
    let setup = open_database(&path).expect("DATABASE_OPEN_FAILED");
    setup
        .execute_batch("CREATE TABLE existing_fixture (value TEXT NOT NULL)")
        .unwrap();
    setup
        .execute("INSERT INTO existing_fixture VALUES ('before')", [])
        .unwrap();
    drop(setup);

    let connection = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    connection
        .execute_batch("BEGIN; CREATE TABLE transient_fixture (value TEXT)")
        .unwrap();
    let error = execute_migration(connection, "SELECT 1")
        .expect_err("NESTED_MIGRATION_TRANSACTION_ACCEPTED");
    assert_eq!(error, MigrationError::BeginFailed);

    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert_eq!(schema_object_count(&reopened).unwrap(), 1);
    let value: String = reopened
        .query_row("SELECT value FROM existing_fixture", [], |row| row.get(0))
        .unwrap();
    assert_eq!(value, "before");
}

#[test]
fn migration_executor_reports_commit_busy_and_preserves_file_state() {
    let directory = common::TempDirectory::new();
    let path = directory.path().join("commit-failure.sqlite");
    let setup = open_database(&path).expect("DATABASE_OPEN_FAILED");
    setup
        .execute_batch(
            "PRAGMA journal_mode=DELETE; CREATE TABLE existing_fixture (value TEXT NOT NULL)",
        )
        .unwrap();
    setup
        .execute("INSERT INTO existing_fixture VALUES ('before')", [])
        .unwrap();
    drop(setup);

    let reader = Connection::open(&path).expect("READER_OPEN_FAILED");
    reader.execute_batch("BEGIN DEFERRED").unwrap();
    let _: i64 = reader
        .query_row("SELECT count(*) FROM existing_fixture", [], |row| {
            row.get(0)
        })
        .unwrap();

    let writer = open_database(&path).expect("WRITER_OPEN_FAILED");
    writer
        .busy_timeout(std::time::Duration::ZERO)
        .expect("BUSY_TIMEOUT_SETUP_FAILED");
    let error = execute_migration(writer, "INSERT INTO existing_fixture VALUES ('during')")
        .expect_err("BUSY_COMMIT_SUCCEEDED");
    assert_eq!(error, MigrationError::CommitFailed);
    drop(reader);

    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    let values: Vec<String> = {
        let mut statement = reopened
            .prepare("SELECT value FROM existing_fixture ORDER BY rowid")
            .unwrap();
        statement
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(values, ["before"]);
}

#[test]
fn migration_errors_have_static_sanitized_presentations() {
    let cases = [
        (
            MigrationError::BeginFailed,
            "migration could not begin",
            "BeginFailed",
        ),
        (
            MigrationError::ApplyFailed,
            "migration could not be applied",
            "ApplyFailed",
        ),
        (
            MigrationError::CommitFailed,
            "migration could not commit",
            "CommitFailed",
        ),
        (
            MigrationError::RollbackFailed,
            "migration rollback failed",
            "RollbackFailed",
        ),
    ];
    for (error, display, debug) in cases {
        assert_eq!(error.to_string(), display);
        assert_eq!(format!("{error:?}"), debug);
        assert!(error.source().is_none());
    }
}

#[test]
fn invalid_database_paths_fail_without_exposing_path_or_source() {
    let directory = common::TempDirectory::new();
    let parent_file = directory.path().join("parent-file");
    let parent_file_bytes = b"synthetic parent";
    fs::write(&parent_file, parent_file_bytes).unwrap();
    let malformed = directory.path().join("malformed.sqlite");
    let malformed_bytes = b"synthetic not sqlite";
    fs::write(&malformed, malformed_bytes).unwrap();

    let invalid = [
        Path::new(""),
        Path::new("relative.sqlite"),
        Path::new(":memory:"),
        Path::new("file:test.sqlite?mode=memory"),
    ];
    for path in invalid {
        assert_open_error(path, DatabaseOpenError::InvalidPath);
    }
    assert_open_error(directory.path(), DatabaseOpenError::OpenFailed);
    assert_open_error(
        &directory.path().join("missing-parent").join("db.sqlite"),
        DatabaseOpenError::OpenFailed,
    );
    assert_open_error(
        &parent_file.join("child.sqlite"),
        DatabaseOpenError::OpenFailed,
    );
    assert_eq!(fs::read(&parent_file).unwrap(), parent_file_bytes);
    assert_open_error(&malformed, DatabaseOpenError::OpenFailed);
    assert_eq!(fs::read(&malformed).unwrap(), malformed_bytes);
}

fn assert_open_error(path: &Path, expected: DatabaseOpenError) {
    let error = open_database(path).expect_err("INVALID_DATABASE_PATH_ACCEPTED");
    assert_eq!(error, expected);
    let (display, debug) = match expected {
        DatabaseOpenError::InvalidPath => ("database path is invalid", "InvalidPath"),
        DatabaseOpenError::OpenFailed => ("database could not be opened", "OpenFailed"),
    };
    assert_eq!(error.to_string(), display);
    assert_eq!(format!("{error:?}"), debug);
    assert!(error.source().is_none());
}
