mod common;

use rusqlite::Connection;
use std::error::Error;
use std::fs;
use std::path::Path;

use millani_artes::database::{
    execute_migration, execute_versioned_migration, open_database, schema_version,
    DatabaseOpenError, MigrationError,
};

fn schema_object_count(connection: &Connection) -> rusqlite::Result<i64> {
    connection.query_row("SELECT count(*) FROM sqlite_schema", [], |row| row.get(0))
}

fn assert_empty_migration_ledger(connection: &Connection) {
    assert!(connection.is_autocommit());
    assert_eq!(schema_object_count(connection).unwrap(), 1);
    assert_eq!(schema_version(connection, 1).unwrap(), 1);
    let ledger_sql: String = connection
        .query_row(
            "SELECT sql FROM main.sqlite_schema WHERE type = 'table' AND name = 'schema_migration'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(ledger_sql.contains("CHECK (version > 0)"));
    let mut statement = connection
        .prepare("PRAGMA main.table_xinfo('schema_migration')")
        .unwrap();
    let columns: Vec<(String, String, i64, i64, i64)> = statement
        .query_map([], |row| {
            Ok((
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(5)?,
                row.get(6)?,
            ))
        })
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert_eq!(columns, vec![("version".into(), "INTEGER".into(), 1, 1, 0)]);
    let versions: Vec<i64> = {
        let mut statement = connection
            .prepare("SELECT version FROM main.schema_migration ORDER BY version")
            .unwrap();
        statement
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(versions, [1]);
}

fn assert_db006_upgrade_snapshot(connection: &Connection) {
    type SnapshotColumn = (i64, String, String, i64, Option<String>, i64, i64);

    assert!(connection.is_autocommit());
    assert_eq!(schema_object_count(connection).unwrap(), 2);
    assert_eq!(schema_version(connection, 1).unwrap(), 1);
    let objects: Vec<String> = {
        let mut statement = connection
            .prepare(
                "SELECT name FROM main.sqlite_schema WHERE name NOT GLOB 'sqlite_*' ORDER BY name",
            )
            .unwrap();
        statement
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(objects, ["schema_migration", "upgrade_fixture"]);
    let ledger_sql: String = connection
        .query_row(
            "SELECT sql FROM main.sqlite_schema WHERE type = 'table' AND name = 'schema_migration'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(ledger_sql.contains("CHECK (version > 0)"));
    let ledger_versions: Vec<i64> = {
        let mut statement = connection
            .prepare("SELECT version FROM main.schema_migration ORDER BY version")
            .unwrap();
        statement
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(ledger_versions, [1]);

    let ledger_columns: Vec<SnapshotColumn> = {
        let mut statement = connection
            .prepare("PRAGMA main.table_xinfo('schema_migration')")
            .unwrap();
        statement
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                ))
            })
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(
        ledger_columns,
        [(0, "version".into(), "INTEGER".into(), 1, None, 1, 0)]
    );

    let fixture_columns: Vec<SnapshotColumn> = {
        let mut statement = connection
            .prepare("PRAGMA main.table_xinfo('upgrade_fixture')")
            .unwrap();
        statement
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                ))
            })
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(
        fixture_columns,
        [
            (0, "id".into(), "INTEGER".into(), 1, None, 1, 0),
            (1, "value".into(), "TEXT".into(), 1, None, 0, 0),
        ]
    );

    let rows: Vec<(i64, String)> = {
        let mut statement = connection
            .prepare("SELECT id, value FROM main.upgrade_fixture ORDER BY id")
            .unwrap();
        statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(
        rows,
        [(1, "O'Brien".into()), (2, "synthetic-before".into())]
    );
    let integrity: String = connection
        .query_row("PRAGMA main.integrity_check", [], |row| row.get(0))
        .unwrap();
    assert_eq!(integrity, "ok");
}

fn seed_version_one(connection: Connection) -> Connection {
    execute_versioned_migration(
        connection,
        1,
        "CREATE TABLE versioned_fixture (value TEXT NOT NULL)",
    )
    .expect("VERSION_ONE_MIGRATION_FAILED")
}

fn assert_invalid_ledger(path: &Path, setup_sql: &str, supported_version: u32) {
    let connection = open_database(path).expect("DATABASE_OPEN_FAILED");
    connection.execute_batch(setup_sql).unwrap();
    assert_eq!(
        schema_version(&connection, supported_version),
        Err(MigrationError::InvalidSchemaState)
    );
    assert!(connection.is_autocommit());
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
        (
            MigrationError::InvalidVersion,
            "migration version is invalid",
            "InvalidVersion",
        ),
        (
            MigrationError::InvalidSchemaState,
            "database schema state is invalid",
            "InvalidSchemaState",
        ),
    ];
    for (error, display, debug) in cases {
        assert_eq!(error.to_string(), display);
        assert_eq!(format!("{error:?}"), debug);
        assert!(error.source().is_none());
    }
}

#[test]
fn versioned_migrations_record_and_read_versions_after_reopen() {
    let directory = common::TempDirectory::new();
    let path = directory.path().join("versioned-migration.sqlite");
    let connection = open_database(&path).expect("DATABASE_OPEN_FAILED");
    assert_eq!(
        schema_version(&connection, 2).expect("INITIAL_SCHEMA_VERSION_READ_FAILED"),
        0
    );
    assert_eq!(schema_object_count(&connection).unwrap(), 0);

    let connection = execute_versioned_migration(
        connection,
        1,
        "CREATE TABLE versioned_fixture (value TEXT NOT NULL)",
    )
    .expect("VERSION_ONE_MIGRATION_FAILED");
    assert_eq!(schema_version(&connection, 2).unwrap(), 1);

    let connection = execute_versioned_migration(
        connection,
        2,
        "ALTER TABLE versioned_fixture ADD COLUMN upgrade_marker TEXT NOT NULL DEFAULT 'upgrade'",
    )
    .expect("VERSION_TWO_MIGRATION_FAILED");
    connection
        .execute(
            "INSERT INTO versioned_fixture (value) VALUES (?1)",
            ["O'Brien"],
        )
        .unwrap();
    drop(connection);

    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert_eq!(schema_version(&reopened, 2).unwrap(), 2);
    let versions: Vec<i64> = {
        let mut statement = reopened
            .prepare("SELECT version FROM schema_migration ORDER BY version")
            .unwrap();
        statement
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(versions, [1, 2]);
    let (value, marker): (String, String) = reopened
        .query_row(
            "SELECT value, upgrade_marker FROM versioned_fixture",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(value, "O'Brien");
    assert_eq!(marker, "upgrade");
}

#[test]
fn empty_versioned_migration_starts_on_independent_files() {
    let directory = common::TempDirectory::new();
    let first_path = directory.path().join("empty-migration-first.sqlite");
    let second_path = directory.path().join("empty-migration-second.sqlite");
    let first = open_database(&first_path).expect("FIRST_DATABASE_OPEN_FAILED");
    let second = open_database(&second_path).expect("SECOND_DATABASE_OPEN_FAILED");
    assert!(first_path.is_file());
    assert!(second_path.is_file());
    assert_eq!(schema_object_count(&first).unwrap(), 0);
    assert_eq!(schema_version(&first, 1).unwrap(), 0);
    assert_eq!(schema_object_count(&second).unwrap(), 0);
    assert_eq!(schema_version(&second, 1).unwrap(), 0);

    let first = execute_versioned_migration(first, 1, "").expect("FIRST_MIGRATION_FAILED");
    assert_empty_migration_ledger(&first);
    drop(first);

    assert_eq!(schema_object_count(&second).unwrap(), 0);
    assert_eq!(schema_version(&second, 1).unwrap(), 0);
    let second = execute_versioned_migration(second, 1, "").expect("SECOND_MIGRATION_FAILED");
    assert_empty_migration_ledger(&second);
    drop(second);

    for path in [&first_path, &second_path] {
        let reopened = open_database(path).expect("DATABASE_REOPEN_FAILED");
        assert_empty_migration_ledger(&reopened);
        let integrity: String = reopened
            .query_row("PRAGMA main.integrity_check", [], |row| row.get(0))
            .unwrap();
        assert_eq!(integrity, "ok");
    }
}

#[test]
fn empty_versioned_migration_rejects_invalid_ledger_rows() {
    let directory = common::TempDirectory::new();
    let path = directory.path().join("empty-migration-constraints.sqlite");
    let connection = open_database(&path).expect("DATABASE_OPEN_FAILED");
    assert!(path.is_file());
    assert_eq!(schema_object_count(&connection).unwrap(), 0);
    assert_eq!(schema_version(&connection, 1).unwrap(), 0);
    let connection = execute_versioned_migration(connection, 1, "").expect("MIGRATION_FAILED");
    assert_empty_migration_ledger(&connection);

    for (version, expected_code) in [
        (1, rusqlite::ffi::SQLITE_CONSTRAINT_PRIMARYKEY),
        (0, rusqlite::ffi::SQLITE_CONSTRAINT_CHECK),
        (-1, rusqlite::ffi::SQLITE_CONSTRAINT_CHECK),
    ] {
        let error = connection
            .execute(
                "INSERT INTO main.schema_migration(version) VALUES (?1)",
                [version],
            )
            .expect_err("INVALID_LEDGER_VERSION_ACCEPTED");
        match error {
            rusqlite::Error::SqliteFailure(error, _) => {
                assert_eq!(error.code, rusqlite::ErrorCode::ConstraintViolation);
                assert_eq!(error.extended_code, expected_code);
            }
            other => panic!("UNEXPECTED_SQLITE_ERROR: {other:?}"),
        }
        assert_empty_migration_ledger(&connection);
    }
    drop(connection);

    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert_empty_migration_ledger(&reopened);
    let integrity: String = reopened
        .query_row("PRAGMA main.integrity_check", [], |row| row.get(0))
        .unwrap();
    assert_eq!(integrity, "ok");
}

#[test]
fn db006_materializes_and_reopens_isolated_upgrade_snapshots() {
    let directory = common::TempDirectory::new();
    let first_path = directory.path().join("db006-first.sqlite");
    let second_path = directory.path().join("db006-second.sqlite");
    let first = open_database(&first_path).expect("FIRST_DATABASE_OPEN_FAILED");
    let second = open_database(&second_path).expect("SECOND_DATABASE_OPEN_FAILED");
    assert!(first_path.is_file());
    assert!(second_path.is_file());
    assert_eq!(schema_object_count(&first).unwrap(), 0);
    assert_eq!(schema_version(&first, 1).unwrap(), 0);
    assert_eq!(schema_object_count(&second).unwrap(), 0);
    assert_eq!(schema_version(&second, 1).unwrap(), 0);

    let first = execute_migration(first, include_str!("fixtures/db006-upgrade-v1.sql"))
        .expect("FIRST_SNAPSHOT_MATERIALIZATION_FAILED");
    assert_db006_upgrade_snapshot(&first);
    drop(first);

    assert_eq!(schema_object_count(&second).unwrap(), 0);
    assert_eq!(schema_version(&second, 1).unwrap(), 0);
    let first = open_database(&first_path).expect("FIRST_DATABASE_REOPEN_FAILED");
    assert_db006_upgrade_snapshot(&first);
    drop(first);

    let second = execute_migration(second, include_str!("fixtures/db006-upgrade-v1.sql"))
        .expect("SECOND_SNAPSHOT_MATERIALIZATION_FAILED");
    assert_db006_upgrade_snapshot(&second);
    drop(second);

    for path in [&first_path, &second_path] {
        let reopened = open_database(path).expect("DATABASE_REOPEN_FAILED");
        assert_db006_upgrade_snapshot(&reopened);
    }
}

fn assert_db007_upgraded_snapshot(connection: &Connection) {
    type SnapshotColumn = (i64, String, String, i64, Option<String>, i64, i64);

    assert!(connection.is_autocommit());
    let ledger_versions: Vec<i64> = {
        let mut statement = connection
            .prepare("SELECT version FROM main.schema_migration ORDER BY version")
            .unwrap();
        statement
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(ledger_versions, [1, 2]);
    assert_eq!(schema_version(connection, 2).unwrap(), 2);

    let objects: Vec<(String, String)> = {
        let mut statement = connection
            .prepare(
                "SELECT type, name FROM main.sqlite_schema WHERE name NOT GLOB 'sqlite_*' ORDER BY name",
            )
            .unwrap();
        statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(
        objects,
        [
            ("table".into(), "schema_migration".into()),
            ("table".into(), "upgrade_fixture".into()),
        ]
    );
    let ledger_sql: String = connection
        .query_row(
            "SELECT sql FROM main.sqlite_schema WHERE type = 'table' AND name = 'schema_migration'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(ledger_sql.contains("CHECK (version > 0)"));
    let ledger_columns: Vec<SnapshotColumn> = {
        let mut statement = connection
            .prepare("PRAGMA main.table_xinfo('schema_migration')")
            .unwrap();
        statement
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                ))
            })
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(
        ledger_columns,
        [(0, "version".into(), "INTEGER".into(), 1, None, 1, 0)]
    );
    let fixture_columns: Vec<SnapshotColumn> = {
        let mut statement = connection
            .prepare("PRAGMA main.table_xinfo('upgrade_fixture')")
            .unwrap();
        statement
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                ))
            })
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(
        fixture_columns,
        [
            (0, "id".into(), "INTEGER".into(), 1, None, 1, 0),
            (1, "value".into(), "TEXT".into(), 1, None, 0, 0),
            (
                2,
                "upgrade_marker".into(),
                "TEXT".into(),
                1,
                Some("'upgrade'".into()),
                0,
                0,
            ),
        ]
    );

    let rows: Vec<(i64, String, String)> = {
        let mut statement = connection
            .prepare("SELECT id, value, upgrade_marker FROM main.upgrade_fixture ORDER BY id")
            .unwrap();
        statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(
        rows,
        [
            (1, "O'Brien".into(), "upgrade".into()),
            (2, "synthetic-before".into(), "upgrade".into()),
        ]
    );
    let integrity: String = connection
        .query_row("PRAGMA main.integrity_check", [], |row| row.get(0))
        .unwrap();
    assert_eq!(integrity, "ok");
}

#[test]
fn versioned_migration_upgrades_snapshot_and_preserves_sentinels() {
    let directory = common::TempDirectory::new();
    let path = directory.path().join("db007-upgrade.sqlite");
    let connection = open_database(&path).expect("DATABASE_OPEN_FAILED");
    assert!(path.is_file());
    assert_eq!(schema_object_count(&connection).unwrap(), 0);
    assert_eq!(schema_version(&connection, 1).unwrap(), 0);

    let connection = execute_migration(connection, include_str!("fixtures/db006-upgrade-v1.sql"))
        .expect("SNAPSHOT_MATERIALIZATION_FAILED");
    assert_db006_upgrade_snapshot(&connection);
    let connection = execute_versioned_migration(
        connection,
        2,
        "ALTER TABLE main.upgrade_fixture ADD COLUMN upgrade_marker TEXT NOT NULL DEFAULT 'upgrade';",
    )
    .expect("SNAPSHOT_UPGRADE_FAILED");
    assert_db007_upgraded_snapshot(&connection);
    drop(connection);

    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert_db007_upgraded_snapshot(&reopened);
}

#[test]
fn schema_version_accepts_only_empty_database_or_empty_ledger() {
    let directory = common::TempDirectory::new();
    let empty_path = directory.path().join("schema-empty.sqlite");
    let empty = open_database(&empty_path).expect("DATABASE_OPEN_FAILED");
    assert_eq!(schema_version(&empty, 0).unwrap(), 0);
    assert_eq!(schema_object_count(&empty).unwrap(), 0);
    drop(empty);

    let ledger_path = directory.path().join("empty-ledger.sqlite");
    let ledger = open_database(&ledger_path).expect("DATABASE_OPEN_FAILED");
    ledger
        .execute_batch("CREATE TABLE schema_migration (version INTEGER PRIMARY KEY NOT NULL)")
        .unwrap();
    assert_eq!(schema_version(&ledger, 0).unwrap(), 0);
}

#[test]
fn versioned_migrations_reject_invalid_requests_without_changing_file() {
    let directory = common::TempDirectory::new();
    let path = directory.path().join("invalid-versions.sqlite");
    let empty = open_database(&path).expect("DATABASE_OPEN_FAILED");
    assert_eq!(
        execute_versioned_migration(empty, 0, "CREATE TABLE should_not_exist (value TEXT)")
            .expect_err("ZERO_VERSION_ACCEPTED"),
        MigrationError::InvalidVersion
    );
    let empty = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert_eq!(schema_object_count(&empty).unwrap(), 0);
    assert_eq!(
        execute_versioned_migration(empty, 2, "CREATE TABLE should_not_exist (value TEXT)")
            .expect_err("GAP_VERSION_ACCEPTED"),
        MigrationError::InvalidVersion
    );
    let empty = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert_eq!(schema_object_count(&empty).unwrap(), 0);

    let version_one = seed_version_one(empty);
    version_one
        .execute("INSERT INTO versioned_fixture VALUES ('before')", [])
        .unwrap();
    drop(version_one);
    let replay = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert_eq!(
        execute_versioned_migration(replay, 1, "INSERT INTO versioned_fixture VALUES ('replay')",)
            .expect_err("REPLAYED_VERSION_ACCEPTED"),
        MigrationError::InvalidVersion
    );
    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert_eq!(schema_version(&reopened, 1).unwrap(), 1);
    let value: String = reopened
        .query_row("SELECT value FROM versioned_fixture", [], |row| row.get(0))
        .unwrap();
    assert_eq!(value, "before");

    let future_path = directory.path().join("unsupported-future.sqlite");
    let future = open_database(&future_path).expect("DATABASE_OPEN_FAILED");
    future
        .execute_batch(
            "CREATE TABLE schema_migration (version INTEGER PRIMARY KEY NOT NULL); INSERT INTO schema_migration VALUES (1), (2)",
        )
        .unwrap();
    assert_eq!(
        execute_versioned_migration(future, 1, "SELECT 1")
            .expect_err("UNSUPPORTED_FUTURE_SCHEMA_ACCEPTED"),
        MigrationError::InvalidSchemaState
    );
    let future_reopened = open_database(&future_path).expect("DATABASE_REOPEN_FAILED");
    assert_eq!(schema_version(&future_reopened, 2).unwrap(), 2);
}

#[test]
fn schema_version_rejects_malformed_ledger_fixtures() {
    let directory = common::TempDirectory::new();
    let cases = [
        (
            "missing-ledger-with-object",
            "CREATE TABLE app_fixture (value TEXT)",
            10,
        ),
        (
            "empty-ledger-with-object",
            "CREATE TABLE schema_migration (version INTEGER PRIMARY KEY NOT NULL); CREATE TABLE app_fixture (value TEXT)",
            10,
        ),
        (
            "empty-ledger-with-homonym-trigger",
            "CREATE TABLE schema_migration (version INTEGER PRIMARY KEY NOT NULL); CREATE TRIGGER schema_migration AFTER INSERT ON schema_migration BEGIN SELECT 1; END",
            10,
        ),
        (
            "empty-ledger-with-sqlitex-user-object",
            "CREATE TABLE schema_migration (version INTEGER PRIMARY KEY NOT NULL); CREATE TABLE sqliteX_shadow (value TEXT)",
            10,
        ),
        (
            "ledger-view",
            "CREATE VIEW schema_migration AS SELECT 1 AS version",
            10,
        ),
        (
            "wrong-shape",
            "CREATE TABLE schema_migration (version INTEGER PRIMARY KEY NOT NULL, extra TEXT)",
            10,
        ),
        (
            "missing-version-column",
            "CREATE TABLE schema_migration (other INTEGER)",
            10,
        ),
        (
            "generated-extra-column",
            "CREATE TABLE schema_migration (version INTEGER PRIMARY KEY NOT NULL, extra TEXT GENERATED ALWAYS AS (version) VIRTUAL)",
            10,
        ),
        (
            "wrong-declared-type",
            "CREATE TABLE schema_migration (version TEXT PRIMARY KEY NOT NULL)",
            10,
        ),
        (
            "text-value",
            "CREATE TABLE schema_migration (version TEXT NOT NULL); INSERT INTO schema_migration VALUES ('one')",
            10,
        ),
        (
            "float-value",
            "CREATE TABLE schema_migration (version REAL NOT NULL); INSERT INTO schema_migration VALUES (1.5)",
            10,
        ),
        (
            "negative-value",
            "CREATE TABLE schema_migration (version INTEGER PRIMARY KEY NOT NULL); INSERT INTO schema_migration VALUES (-1)",
            10,
        ),
        (
            "duplicate-values",
            "CREATE TABLE schema_migration (version INTEGER NOT NULL); INSERT INTO schema_migration VALUES (1), (1)",
            10,
        ),
        (
            "gap-values",
            "CREATE TABLE schema_migration (version INTEGER PRIMARY KEY NOT NULL); INSERT INTO schema_migration VALUES (1), (3)",
            10,
        ),
        (
            "overflow-value",
            "CREATE TABLE schema_migration (version INTEGER PRIMARY KEY NOT NULL); INSERT INTO schema_migration VALUES (4294967296)",
            4294967295,
        ),
        (
            "unsupported-future-version",
            "CREATE TABLE schema_migration (version INTEGER PRIMARY KEY NOT NULL); INSERT INTO schema_migration VALUES (1), (2)",
            1,
        ),
        (
            "nonempty-ledger-with-zero-supported-version",
            "CREATE TABLE schema_migration (version INTEGER PRIMARY KEY NOT NULL); INSERT INTO schema_migration VALUES (1)",
            0,
        ),
    ];
    for (name, setup_sql, supported_version) in cases {
        assert_invalid_ledger(
            &directory.path().join(format!("{name}.sqlite")),
            setup_sql,
            supported_version,
        );
    }
}

#[test]
fn versioned_migration_sql_failure_rolls_back_schema_data_and_ledger() {
    let directory = common::TempDirectory::new();
    let path = directory.path().join("versioned-sql-failure.sqlite");
    let connection = seed_version_one(open_database(&path).expect("DATABASE_OPEN_FAILED"));
    connection
        .execute("INSERT INTO versioned_fixture VALUES ('before')", [])
        .unwrap();
    drop(connection);

    let connection = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert_eq!(
        execute_versioned_migration(
            connection,
            2,
            "CREATE TABLE transient_fixture (value TEXT); INSERT INTO versioned_fixture VALUES ('during'); INSERT INTO missing_fixture VALUES ('fail');",
        )
        .expect_err("FAILING_SQL_MIGRATION_SUCCEEDED"),
        MigrationError::ApplyFailed
    );
    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert_eq!(schema_version(&reopened, 2).unwrap(), 1);
    let transient_count: i64 = reopened
        .query_row(
            "SELECT count(*) FROM main.sqlite_schema WHERE name = 'transient_fixture'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(transient_count, 0);
    let values: Vec<String> = {
        let mut statement = reopened
            .prepare("SELECT value FROM versioned_fixture ORDER BY rowid")
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
fn versioned_migration_ledger_trigger_failure_rolls_back_schema_data_and_version() {
    let directory = common::TempDirectory::new();
    let path = directory.path().join("versioned-ledger-failure.sqlite");
    let connection = seed_version_one(open_database(&path).expect("DATABASE_OPEN_FAILED"));
    connection
        .execute("INSERT INTO versioned_fixture VALUES ('before')", [])
        .unwrap();
    connection
        .execute_batch("CREATE TRIGGER fail_version_two BEFORE INSERT ON schema_migration WHEN NEW.version = 2 BEGIN SELECT RAISE(ABORT, 'synthetic'); END")
        .unwrap();
    drop(connection);

    let connection = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert_eq!(
        execute_versioned_migration(
            connection,
            2,
            "CREATE TABLE transient_fixture (value TEXT); INSERT INTO versioned_fixture VALUES ('during')",
        )
        .expect_err("FAILING_LEDGER_INSERT_SUCCEEDED"),
        MigrationError::ApplyFailed
    );
    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert_eq!(schema_version(&reopened, 2).unwrap(), 1);
    let transient_count: i64 = reopened
        .query_row(
            "SELECT count(*) FROM main.sqlite_schema WHERE name = 'transient_fixture'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(transient_count, 0);
    let values: Vec<String> = {
        let mut statement = reopened
            .prepare("SELECT value FROM versioned_fixture ORDER BY rowid")
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
fn versioned_migrations_use_main_ledger_when_temp_table_shadows_it() {
    let directory = common::TempDirectory::new();
    let path = directory.path().join("temp-shadow.sqlite");
    let connection = seed_version_one(open_database(&path).expect("DATABASE_OPEN_FAILED"));
    connection
        .execute_batch("CREATE TEMP TABLE schema_migration (version INTEGER); INSERT INTO temp.schema_migration VALUES (99)")
        .unwrap();
    assert_eq!(schema_version(&connection, 1).unwrap(), 1);

    let connection = execute_versioned_migration(
        connection,
        2,
        "ALTER TABLE main.versioned_fixture ADD COLUMN upgrade_marker TEXT NOT NULL DEFAULT 'upgrade'",
    )
    .expect("VERSION_TWO_MIGRATION_FAILED");
    assert_eq!(schema_version(&connection, 2).unwrap(), 2);
    let temp_version: i64 = connection
        .query_row("SELECT version FROM temp.schema_migration", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(temp_version, 99);
    let main_versions: Vec<i64> = {
        let mut statement = connection
            .prepare("SELECT version FROM main.schema_migration ORDER BY version")
            .unwrap();
        statement
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(main_versions, [1, 2]);
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

#[test]
fn database_open_enables_foreign_keys_on_each_connection() {
    let directory = common::TempDirectory::new();
    let path = directory.path().join("foreign-keys.sqlite");
    let other_path = directory.path().join("foreign-keys-other.sqlite");
    let first = open_database(&path).expect("FIRST_DATABASE_OPEN_FAILED");
    let second = open_database(&path).expect("SECOND_DATABASE_OPEN_FAILED");
    assert!(first.is_autocommit());
    assert!(second.is_autocommit());
    let first_foreign_keys: i64 = first
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .unwrap();
    let second_foreign_keys: i64 = second
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .unwrap();
    assert_eq!(first_foreign_keys, 1);
    assert_eq!(second_foreign_keys, 1);

    second.pragma_update(None, "foreign_keys", false).unwrap();
    let second_foreign_keys: i64 = second
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .unwrap();
    let first_foreign_keys: i64 = first
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .unwrap();
    assert_eq!(second_foreign_keys, 0);
    assert_eq!(first_foreign_keys, 1);

    let other = open_database(&other_path).expect("OTHER_DATABASE_OPEN_FAILED");
    assert!(other.is_autocommit());
    let other_foreign_keys: i64 = other
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .unwrap();
    assert_eq!(other_foreign_keys, 1);
    drop(second);
    drop(first);
    drop(other);

    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert!(reopened.is_autocommit());
    let reopened_foreign_keys: i64 = reopened
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .unwrap();
    assert_eq!(reopened_foreign_keys, 1);
}
