mod common;

use rusqlite::Connection;
use std::error::Error;
use std::fs;
use std::path::Path;

use millani_artes::database::{open_database, DatabaseOpenError};

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
