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

#[test]
fn foreign_keys_reject_invalid_references_in_file() {
    let directory = common::TempDirectory::new();
    let path = directory.path().join("foreign-key-enforcement.sqlite");
    let connection = open_database(&path).expect("DATABASE_OPEN_FAILED");
    assert!(path.is_file());
    assert!(connection.is_autocommit());
    let foreign_keys: i64 = connection
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .unwrap();
    assert_eq!(foreign_keys, 1);
    connection
        .execute_batch(
            "CREATE TABLE parent (id INTEGER PRIMARY KEY); CREATE TABLE child (id INTEGER PRIMARY KEY, parent_id INTEGER NOT NULL REFERENCES parent(id));",
        )
        .unwrap();
    connection
        .execute("INSERT INTO parent(id) VALUES (?1)", [1])
        .unwrap();
    connection
        .execute("INSERT INTO child(id, parent_id) VALUES (?1, ?2)", [1, 1])
        .unwrap();

    let assert_valid_state = |connection: &rusqlite::Connection| {
        let parents: Vec<i64> = {
            let mut statement = connection
                .prepare("SELECT id FROM parent ORDER BY id")
                .unwrap();
            statement
                .query_map([], |row| row.get(0))
                .unwrap()
                .collect::<rusqlite::Result<_>>()
                .unwrap()
        };
        assert_eq!(parents, [1]);
        let children: Vec<(i64, i64)> = {
            let mut statement = connection
                .prepare("SELECT id, parent_id FROM child ORDER BY id")
                .unwrap();
            statement
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .unwrap()
                .collect::<rusqlite::Result<_>>()
                .unwrap()
        };
        assert_eq!(children, [(1, 1)]);
        let mut statement = connection.prepare("PRAGMA foreign_key_check").unwrap();
        assert!(statement.query([]).unwrap().next().unwrap().is_none());
    };
    let assert_foreign_key_violation = |result: rusqlite::Result<usize>| {
        let error = result.expect_err("INVALID_REFERENCE_ACCEPTED");
        match error {
            rusqlite::Error::SqliteFailure(error, _) => {
                assert_eq!(error.code, rusqlite::ErrorCode::ConstraintViolation);
                assert_eq!(
                    error.extended_code,
                    rusqlite::ffi::SQLITE_CONSTRAINT_FOREIGNKEY
                );
            }
            other => panic!("UNEXPECTED_SQLITE_ERROR: {other:?}"),
        }
    };

    assert_valid_state(&connection);
    assert_foreign_key_violation(
        connection.execute("INSERT INTO child(id, parent_id) VALUES (?1, ?2)", [2, 99]),
    );
    assert_valid_state(&connection);
    assert_foreign_key_violation(
        connection.execute("UPDATE child SET parent_id = ?1 WHERE id = ?2", [99, 1]),
    );
    assert_valid_state(&connection);
    assert_foreign_key_violation(connection.execute("DELETE FROM parent WHERE id = ?1", [1]));
    assert_valid_state(&connection);
    drop(connection);

    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert!(reopened.is_autocommit());
    let foreign_keys: i64 = reopened
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .unwrap();
    assert_eq!(foreign_keys, 1);
    assert_valid_state(&reopened);
    assert_foreign_key_violation(
        reopened.execute("INSERT INTO child(id, parent_id) VALUES (?1, ?2)", [2, 99]),
    );
    assert_valid_state(&reopened);
    assert_foreign_key_violation(
        reopened.execute("UPDATE child SET parent_id = ?1 WHERE id = ?2", [99, 1]),
    );
    assert_valid_state(&reopened);
    assert_foreign_key_violation(reopened.execute("DELETE FROM parent WHERE id = ?1", [1]));
    assert_valid_state(&reopened);
}

#[test]
fn write_transaction_returns_committed_value_in_file() {
    use millani_artes::database::execute_write_transaction;

    let directory = common::TempDirectory::new();
    let path = directory.path().join("write-transaction-success.sqlite");
    let connection = open_database(&path).expect("DATABASE_OPEN_FAILED");
    connection
        .execute_batch("CREATE TABLE write_transaction_fixture (value INTEGER NOT NULL)")
        .unwrap();
    let observer = open_database(&path).expect("OBSERVER_OPEN_FAILED");

    let (connection, value) = execute_write_transaction(connection, |transaction| {
        assert!(!transaction.is_autocommit());
        transaction.execute(
            "INSERT INTO write_transaction_fixture(value) VALUES (?1)",
            [42],
        )?;
        let visible: i64 = observer.query_row(
            "SELECT count(*) FROM write_transaction_fixture",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(visible, 0);
        Ok::<_, rusqlite::Error>(42)
    })
    .expect("WRITE_TRANSACTION_FAILED");
    assert_eq!(value, 42);
    assert!(connection.is_autocommit());
    drop(observer);
    drop(connection);

    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    let persisted: i64 = reopened
        .query_row("SELECT value FROM write_transaction_fixture", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(persisted, 42);
}

#[test]
fn write_transaction_does_not_run_callback_when_writer_is_busy() {
    use millani_artes::database::{execute_write_transaction, TransactionError};
    use std::cell::Cell;
    use std::time::Duration;

    let directory = common::TempDirectory::new();
    let path = directory.path().join("write-transaction-busy.sqlite");
    let mut writer = open_database(&path).expect("WRITER_OPEN_FAILED");
    writer
        .execute_batch("CREATE TABLE write_transaction_busy_fixture (value INTEGER)")
        .unwrap();
    let held_transaction = writer
        .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
        .expect("WRITER_TRANSACTION_FAILED");
    let challenger = open_database(&path).expect("CHALLENGER_OPEN_FAILED");
    challenger.busy_timeout(Duration::ZERO).unwrap();
    let called = Cell::new(false);

    let result = execute_write_transaction(challenger, |_| {
        called.set(true);
        Ok::<_, ()>(())
    });
    assert!(matches!(result, Err(TransactionError::BeginFailed)));
    assert!(!called.get());
    drop(held_transaction);
}

#[test]
fn write_transaction_rejects_deferred_foreign_key_at_commit() {
    use millani_artes::database::{execute_write_transaction, TransactionError};

    let directory = common::TempDirectory::new();
    let path = directory
        .path()
        .join("write-transaction-deferred-fk.sqlite");
    let connection = open_database(&path).expect("DATABASE_OPEN_FAILED");
    connection
        .execute_batch(
            "CREATE TABLE write_transaction_parent (id INTEGER PRIMARY KEY); CREATE TABLE write_transaction_child (parent_id INTEGER REFERENCES write_transaction_parent(id) DEFERRABLE INITIALLY DEFERRED)",
        )
        .unwrap();
    let result = execute_write_transaction(connection, |transaction| {
        transaction.execute(
            "INSERT INTO write_transaction_child(parent_id) VALUES (?1)",
            [999],
        )?;
        Ok::<_, rusqlite::Error>(42)
    });
    assert!(matches!(result, Err(TransactionError::CommitFailed)));
}

#[test]
fn write_transaction_errors_hide_callback_payload() {
    use millani_artes::database::{execute_write_transaction, TransactionError};

    let directory = common::TempDirectory::new();
    let path = directory.path().join("write-transaction-error.sqlite");
    let connection = open_database(&path).expect("DATABASE_OPEN_FAILED");
    let result = execute_write_transaction(connection, |transaction| {
        transaction.query_row(
            "SELECT value FROM synthetic_secret_identifier_7f31",
            [],
            |row| row.get::<_, i64>(0),
        )
    });
    let error = result.expect_err("CALLBACK_ERROR_WAS_ACCEPTED");
    assert!(matches!(&error, TransactionError::OperationFailed(_)));
    match &error {
        TransactionError::OperationFailed(callback_error) => assert!(callback_error
            .to_string()
            .contains("synthetic_secret_identifier_7f31")),
        _ => unreachable!(),
    }
    assert!(error.source().is_none());
    assert!(!error
        .to_string()
        .contains("synthetic_secret_identifier_7f31"));
    assert!(!format!("{error:?}").contains("synthetic_secret_identifier_7f31"));

    let sentinel = "synthetic_secret_identifier_7f31";
    let variants = [
        TransactionError::<String>::BeginFailed,
        TransactionError::OperationFailed(sentinel.to_owned()),
        TransactionError::CommitFailed,
        TransactionError::RollbackFailed,
    ];
    for variant in variants {
        assert!(!variant.to_string().contains(sentinel));
        assert!(!format!("{variant:?}").contains(sentinel));
        assert!(variant.source().is_none());
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Db012Snapshot {
    schema: Vec<(String, String)>,
    parents: Vec<(i64, String)>,
    children: Vec<(i64, i64, String)>,
    integrity: String,
    foreign_key_violation: Option<(String, Option<i64>, String, i64)>,
}

fn db012_seed_baseline(connection: &Connection) -> Db012Snapshot {
    connection
        .execute_batch(
            "CREATE TABLE db012_parent (id INTEGER PRIMARY KEY, value TEXT NOT NULL UNIQUE); CREATE TABLE db012_child (id INTEGER PRIMARY KEY, parent_id INTEGER NOT NULL REFERENCES db012_parent(id) DEFERRABLE INITIALLY DEFERRED, value TEXT NOT NULL CHECK (value <> 'rejected')); INSERT INTO db012_parent VALUES (1, 'original'); INSERT INTO db012_child VALUES (1, 1, 'original-child');",
        )
        .unwrap();
    db012_snapshot(connection)
}

fn db012_snapshot(connection: &Connection) -> Db012Snapshot {
    use rusqlite::OptionalExtension;

    let schema = {
        let mut statement = connection
            .prepare("SELECT name, sql FROM main.sqlite_schema WHERE name NOT GLOB 'sqlite_*' ORDER BY name")
            .unwrap();
        statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    let parents = {
        let mut statement = connection
            .prepare("SELECT id, value FROM db012_parent ORDER BY id")
            .unwrap();
        statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    let children = {
        let mut statement = connection
            .prepare("SELECT id, parent_id, value FROM db012_child ORDER BY id")
            .unwrap();
        statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    let integrity = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .unwrap();
    let foreign_key_violation = connection
        .query_row("PRAGMA foreign_key_check", [], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .optional()
        .unwrap();
    Db012Snapshot {
        schema,
        parents,
        children,
        integrity,
        foreign_key_violation,
    }
}

#[test]
fn db012_write_failure_rolls_back_schema_and_rows() {
    use millani_artes::database::{execute_write_transaction, TransactionError};
    use std::cell::Cell;

    let directory = common::TempDirectory::new();
    let path = directory.path().join("db012-operation-failure.sqlite");
    let connection = open_database(&path).expect("DATABASE_OPEN_FAILED");
    let original = db012_seed_baseline(&connection);
    assert_eq!(original.integrity, "ok");
    assert_eq!(original.foreign_key_violation, None);
    let completed_steps = Cell::new(0);

    let result = execute_write_transaction(connection, |transaction| {
        transaction.execute("UPDATE db012_parent SET value = 'changed' WHERE id = 1", [])?;
        completed_steps.set(1);
        transaction.execute("INSERT INTO db012_parent VALUES (2, 'new-parent')", [])?;
        completed_steps.set(2);
        transaction.execute_batch("CREATE TABLE db012_transient (value TEXT NOT NULL)")?;
        completed_steps.set(3);
        transaction.execute("INSERT INTO db012_child VALUES (2, 1, 'rejected')", [])?;
        Ok::<_, rusqlite::Error>(())
    });
    let error = result.expect_err("CONSTRAINT_FAILURE_WAS_ACCEPTED");
    match error {
        TransactionError::OperationFailed(rusqlite::Error::SqliteFailure(code, _)) => {
            assert_eq!(code.code, rusqlite::ErrorCode::ConstraintViolation);
            assert_eq!(code.extended_code, rusqlite::ffi::SQLITE_CONSTRAINT_CHECK);
        }
        other => panic!("UNEXPECTED_TRANSACTION_ERROR: {other:?}"),
    }
    assert_eq!(completed_steps.get(), 3);

    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert_eq!(db012_snapshot(&reopened), original);
}

#[test]
fn db012_validation_rejection_preserves_file() {
    use millani_artes::database::{execute_write_transaction, TransactionError};
    use millani_artes::domain::DomainError;

    let directory = common::TempDirectory::new();
    let path = directory.path().join("db012-validation.sqlite");
    let connection = open_database(&path).expect("DATABASE_OPEN_FAILED");
    let original = db012_seed_baseline(&connection);
    let result = execute_write_transaction(connection, |transaction| {
        let value: String = transaction
            .query_row("SELECT value FROM db012_parent WHERE id = 1", [], |row| {
                row.get(0)
            })
            .expect("STATE_READ_FAILED");
        assert_eq!(value, "original");
        Err::<(), _>(DomainError::InvalidInput)
    });
    assert!(matches!(
        result,
        Err(TransactionError::OperationFailed(DomainError::InvalidInput))
    ));
    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert_eq!(db012_snapshot(&reopened), original);
}

#[test]
fn db012_commit_failure_preserves_file() {
    use millani_artes::database::{execute_write_transaction, TransactionError};

    let directory = common::TempDirectory::new();
    let path = directory.path().join("db012-commit-failure.sqlite");
    let connection = open_database(&path).expect("DATABASE_OPEN_FAILED");
    let original = db012_seed_baseline(&connection);
    let result = execute_write_transaction(connection, |transaction| {
        transaction.execute("UPDATE db012_parent SET value = 'changed' WHERE id = 1", [])?;
        transaction.execute("INSERT INTO db012_parent VALUES (2, 'new-parent')", [])?;
        transaction.execute("INSERT INTO db012_child VALUES (2, 999, 'orphan')", [])?;
        Ok::<_, rusqlite::Error>(42)
    });
    assert!(matches!(result, Err(TransactionError::CommitFailed)));
    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert_eq!(db012_snapshot(&reopened), original);
}

#[test]
fn db012_panic_unwind_preserves_file() {
    use millani_artes::database::execute_write_transaction;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    let directory = common::TempDirectory::new();
    let path = directory.path().join("db012-panic.sqlite");
    let connection = open_database(&path).expect("DATABASE_OPEN_FAILED");
    let original = db012_seed_baseline(&connection);
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = execute_write_transaction(connection, |transaction| -> rusqlite::Result<()> {
            transaction.execute("UPDATE db012_parent SET value = 'changed' WHERE id = 1", [])?;
            transaction
                .execute_batch("CREATE TABLE db012_panic_transient (value TEXT NOT NULL)")?;
            panic!("DB012_SYNTHETIC_UNWIND");
        });
    }));
    assert!(panic.is_err());
    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert_eq!(db012_snapshot(&reopened), original);
}

#[test]
fn db012_abrupt_exit_recovers_hot_journal() {
    use millani_artes::database::{execute_write_transaction, TransactionError};
    use std::process::Command;

    const CHILD_PATH_ENV: &str = "MILLANI_DB012_ABRUPT_CHILD_PATH";
    if let Some(child_path) = std::env::var_os(CHILD_PATH_ENV) {
        let path = Path::new(&child_path);
        let parent = path.parent().expect("CHILD_DATABASE_PARENT_MISSING");
        let temp_root =
            std::fs::canonicalize(std::env::temp_dir()).expect("TEMP_DIRECTORY_ROOT_UNAVAILABLE");
        assert!(path.is_absolute());
        assert_eq!(std::fs::canonicalize(parent).unwrap(), parent);
        assert_eq!(parent.parent(), Some(temp_root.as_path()));
        assert!(parent
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("millani-db002-")));
        assert!(path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("db012-abrupt-")));
        assert!(path.is_file());

        let connection = open_database(path).expect("CHILD_DATABASE_OPEN_FAILED");
        assert_eq!(db012_snapshot(&connection).integrity, "ok");
        connection
            .pragma_update(None, "journal_mode", "DELETE")
            .unwrap();
        let journal_mode: String = connection
            .pragma_query_value(None, "journal_mode", |row| row.get(0))
            .unwrap();
        assert_eq!(journal_mode, "delete");
        connection.pragma_update(None, "cache_size", 2).unwrap();
        connection.pragma_update(None, "cache_spill", true).unwrap();
        let large_value = "x".repeat(32 * 1024);
        let _: Result<(Connection, ()), TransactionError<rusqlite::Error>> =
            execute_write_transaction(connection, |transaction| {
                for id in 2..66 {
                    transaction.execute(
                        "INSERT INTO db012_parent(id, value) VALUES (?1, ?2)",
                        rusqlite::params![id, format!("{id}-{large_value}")],
                    )?;
                }
                std::process::exit(86);
            });
        unreachable!("CHILD_PROCESS_DID_NOT_EXIT");
    }

    let directory = common::TempDirectory::new();
    let path = directory.path().join("db012-abrupt-recovery.sqlite");
    let connection = open_database(&path).expect("DATABASE_OPEN_FAILED");
    let original = db012_seed_baseline(&connection);
    drop(connection);

    let output = Command::new(std::env::current_exe().expect("TEST_EXE_UNAVAILABLE"))
        .args([
            "--exact",
            "db012_abrupt_exit_recovers_hot_journal",
            "--nocapture",
        ])
        .env(CHILD_PATH_ENV, &path)
        .output()
        .expect("CHILD_PROCESS_START_FAILED");
    assert_eq!(output.status.code(), Some(86), "CHILD_EXIT_WAS_NOT_86");

    let journal_path = std::path::PathBuf::from(format!("{}-journal", path.display()));
    let journal = fs::read(&journal_path).expect("HOT_JOURNAL_MISSING");
    assert!(journal.len() > 512);
    assert_eq!(
        &journal[..8],
        &[0xd9, 0xd5, 0x05, 0xf9, 0x20, 0xa1, 0x63, 0xd7]
    );

    let reopened = open_database(&path).expect("DATABASE_RECOVERY_OPEN_FAILED");
    assert_eq!(db012_snapshot(&reopened), original);
    assert!(!journal_path.exists());
    let (reopened, ()) = execute_write_transaction(reopened, |_| Ok::<_, rusqlite::Error>(()))
        .expect("DATABASE_WRITE_LOCK_REMAINS");
    assert_eq!(db012_snapshot(&reopened), original);
}

#[test]
fn db013_audit_event_persists_metadata() {
    use millani_artes::database::{append_audit_event, execute_write_transaction, AuditEventError};

    let directory = common::TempDirectory::new();
    let path = directory.path().join("db013-audit-event.sqlite");
    let connection = open_database(&path).expect("DATABASE_OPEN_FAILED");
    let connection =
        execute_versioned_migration(connection, 1, "").expect("BASELINE_MIGRATION_FAILED");
    let connection = execute_versioned_migration(
        connection,
        millani_artes::database::AUDIT_EVENT_MIGRATION_VERSION,
        millani_artes::database::AUDIT_EVENT_MIGRATION_SQL,
    )
    .expect("AUDIT_EVENT_MIGRATION_FAILED");
    let event = millani_artes::database::AuditEvent {
        event_id: [1; 16],
        occurred_at_ms: 1_765_000_000_123,
        operation: "sale_create",
        entity: "sale",
        entity_id: [2; 16],
    };

    let (connection, ()) = execute_write_transaction(connection, |transaction| {
        append_audit_event(transaction, &event)?;
        let observer = open_database(&path).expect("OBSERVER_DATABASE_OPEN_FAILED");
        let before_commit: i64 = observer
            .query_row("SELECT count(*) FROM main.audit_event", [], |row| {
                row.get(0)
            })
            .expect("OBSERVER_READ_FAILED");
        assert_eq!(before_commit, 0);
        Ok::<(), AuditEventError>(())
    })
    .expect("AUDIT_EVENT_TRANSACTION_FAILED");
    drop(connection);

    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert_eq!(schema_version(&reopened, 2).unwrap(), 2);
    let stored: (Vec<u8>, i64, String, String, Vec<u8>) = reopened
        .query_row(
            "SELECT event_id, occurred_at_ms, operation, entity, entity_id FROM main.audit_event",
            [],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .expect("AUDIT_EVENT_NOT_PERSISTED");
    assert_eq!(
        stored,
        (
            [1; 16].to_vec(),
            1_765_000_000_123,
            "sale_create".into(),
            "sale".into(),
            [2; 16].to_vec()
        )
    );
    assert_eq!(
        reopened
            .query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
}

fn db013_migrate_v2(connection: Connection) -> Connection {
    let connection =
        execute_versioned_migration(connection, 1, "").expect("BASELINE_MIGRATION_FAILED");
    execute_versioned_migration(
        connection,
        millani_artes::database::AUDIT_EVENT_MIGRATION_VERSION,
        millani_artes::database::AUDIT_EVENT_MIGRATION_SQL,
    )
    .expect("AUDIT_EVENT_MIGRATION_FAILED")
}

fn db013_event(
    event_id: [u8; 16],
    occurred_at_ms: i64,
    operation: &'static str,
    entity: &'static str,
    entity_id: [u8; 16],
) -> millani_artes::database::AuditEvent {
    millani_artes::database::AuditEvent {
        event_id,
        occurred_at_ms,
        operation,
        entity,
        entity_id,
    }
}

#[test]
fn db013_audit_schema_initializes_empty_and_upgrades_file() {
    let directory = common::TempDirectory::new();
    let empty_path = directory.path().join("db013-empty-upgrade.sqlite");
    let empty = open_database(&empty_path).expect("EMPTY_DATABASE_OPEN_FAILED");
    assert_eq!(schema_version(&empty, 2).unwrap(), 0);
    let empty = db013_migrate_v2(empty);
    assert_eq!(schema_version(&empty, 2).unwrap(), 2);
    drop(empty);
    let reopened = open_database(&empty_path).expect("EMPTY_DATABASE_REOPEN_FAILED");
    assert_eq!(schema_version(&reopened, 2).unwrap(), 2);
    assert_eq!(
        reopened
            .query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    drop(reopened);

    let fixture_path = directory.path().join("db013-existing-v1.sqlite");
    let fixture = open_database(&fixture_path).expect("FIXTURE_DATABASE_OPEN_FAILED");
    let fixture = execute_versioned_migration(
        fixture,
        1,
        "CREATE TABLE db013_probe(id INTEGER PRIMARY KEY, value TEXT NOT NULL); CREATE INDEX db013_probe_value ON db013_probe(value); CREATE TRIGGER db013_probe_no_delete BEFORE DELETE ON db013_probe BEGIN SELECT RAISE(ABORT, 'PROBE_DELETE_BLOCKED'); END; INSERT INTO db013_probe VALUES (7, 'preserved');",
    )
    .expect("FIXTURE_V1_MIGRATION_FAILED");
    let fixture = execute_versioned_migration(
        fixture,
        millani_artes::database::AUDIT_EVENT_MIGRATION_VERSION,
        millani_artes::database::AUDIT_EVENT_MIGRATION_SQL,
    )
    .expect("FIXTURE_UPGRADE_FAILED");
    let probe: (i64, String) = fixture
        .query_row("SELECT id, value FROM db013_probe", [], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
        .unwrap();
    assert_eq!(probe, (7, "preserved".into()));
    assert_eq!(schema_version(&fixture, 2).unwrap(), 2);
    let objects: Vec<String> = {
        let mut statement = fixture.prepare(
            "SELECT name FROM main.sqlite_schema WHERE name IN ('db013_probe_value', 'db013_probe_no_delete', 'audit_event_no_update', 'audit_event_no_delete') ORDER BY name",
        ).unwrap();
        statement
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(
        objects,
        [
            "audit_event_no_delete",
            "audit_event_no_update",
            "db013_probe_no_delete",
            "db013_probe_value"
        ]
    );
    let columns: Vec<(String, String, i64)> = {
        let mut statement = fixture
            .prepare("PRAGMA main.table_xinfo('audit_event')")
            .unwrap();
        statement
            .query_map([], |row| Ok((row.get(1)?, row.get(2)?, row.get(3)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(
        columns,
        [
            ("event_id".into(), "BLOB".into(), 1),
            ("occurred_at_ms".into(), "INTEGER".into(), 1),
            ("operation".into(), "TEXT".into(), 1),
            ("entity".into(), "TEXT".into(), 1),
            ("entity_id".into(), "BLOB".into(), 1)
        ]
    );
    drop(fixture);

    let reopened = open_database(&fixture_path).expect("UPGRADED_DATABASE_REOPEN_FAILED");
    assert_eq!(schema_version(&reopened, 2).unwrap(), 2);
    assert_eq!(
        reopened
            .query_row("SELECT value FROM db013_probe WHERE id = 7", [], |row| {
                row.get::<_, String>(0)
            })
            .unwrap(),
        "preserved"
    );
    assert_eq!(
        reopened
            .query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
}

#[test]
fn db013_audit_event_rejects_invalid_metadata_and_duplicates() {
    use millani_artes::database::{append_audit_event, execute_write_transaction, AuditEventError};

    let directory = common::TempDirectory::new();
    let path = directory.path().join("db013-validation.sqlite");
    let connection = db013_migrate_v2(open_database(&path).expect("DATABASE_OPEN_FAILED"));
    let valid_64: &'static str = Box::leak("a".repeat(64).into_boxed_str());
    let invalid_tags = [
        "",
        "Upper",
        "é",
        "a b",
        "a\0b",
        "a-",
        "_a",
        "a".repeat(65).leak(),
    ];
    let (connection, ()) = execute_write_transaction(connection, |transaction| {
        for tag in invalid_tags {
            let event = db013_event([1; 16], 0, tag, "sale", [2; 16]);
            assert_eq!(
                append_audit_event(transaction, &event),
                Err(AuditEventError::InvalidEvent)
            );
            let event = db013_event([1; 16], 0, "sale_create", tag, [2; 16]);
            assert_eq!(
                append_audit_event(transaction, &event),
                Err(AuditEventError::InvalidEvent)
            );
        }
        let mut nil_event_id = db013_event([0; 16], 0, "a", "a", [2; 16]);
        assert_eq!(
            append_audit_event(transaction, &nil_event_id),
            Err(AuditEventError::InvalidEvent)
        );
        nil_event_id.event_id = [1; 16];
        nil_event_id.entity_id = [0; 16];
        assert_eq!(
            append_audit_event(transaction, &nil_event_id),
            Err(AuditEventError::InvalidEvent)
        );
        let negative_time = db013_event([3; 16], -1, "a", "a", [4; 16]);
        assert_eq!(
            append_audit_event(transaction, &negative_time),
            Err(AuditEventError::InvalidEvent)
        );

        let boundary_min = db013_event([5; 16], 0, "a", "a", [6; 16]);
        append_audit_event(transaction, &boundary_min)?;
        let boundary_max = db013_event([7; 16], 1, valid_64, valid_64, [8; 16]);
        append_audit_event(transaction, &boundary_max)?;
        assert_eq!(
            append_audit_event(transaction, &boundary_min),
            Err(AuditEventError::InsertFailed)
        );
        let preserved: i64 = transaction
            .query_row(
                "SELECT occurred_at_ms FROM main.audit_event WHERE event_id = ?1",
                [boundary_min.event_id.as_slice()],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(preserved, 0);
        Ok::<(), AuditEventError>(())
    })
    .expect("VALIDATION_TRANSACTION_FAILED");
    drop(connection);
    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert_eq!(schema_version(&reopened, 2).unwrap(), 2);
    assert_eq!(
        reopened
            .query_row("SELECT count(*) FROM main.audit_event", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        2
    );
    assert_eq!(
        reopened
            .query_row(
                "SELECT operation, length(operation) FROM main.audit_event WHERE event_id = ?1",
                [[7; 16].as_slice()],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            )
            .unwrap(),
        ("a".repeat(64), 64)
    );
}

fn db013_assert_constraint_row(connection: &Connection, values: &[rusqlite::types::Value; 5]) {
    let result = connection.execute(
        "INSERT INTO main.audit_event (event_id, occurred_at_ms, operation, entity, entity_id) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params_from_iter(values.iter()),
    );
    match result {
        Err(rusqlite::Error::SqliteFailure(code, _)) => {
            assert_eq!(code.code, rusqlite::ErrorCode::ConstraintViolation);
        }
        other => panic!("INVALID_ROW_DID_NOT_FAIL_CONSTRAINT: {other:?}"),
    }
}

#[test]
fn db013_audit_schema_rejects_invalid_rows_and_mutations() {
    use rusqlite::types::Value;

    let directory = common::TempDirectory::new();
    let path = directory.path().join("db013-constraints.sqlite");
    let connection = db013_migrate_v2(open_database(&path).expect("DATABASE_OPEN_FAILED"));
    let valid = [
        Value::Blob(vec![1; 16]),
        Value::Integer(0),
        Value::Text("a".into()),
        Value::Text("a".into()),
        Value::Blob(vec![2; 16]),
    ];
    connection.execute(
        "INSERT INTO main.audit_event (event_id, occurred_at_ms, operation, entity, entity_id) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params_from_iter(valid.iter()),
    ).expect("VALID_ROW_INSERT_FAILED");
    let mut invalid_rows = Vec::new();
    for column in 0..5 {
        let mut row = valid.clone();
        row[column] = Value::Null;
        invalid_rows.push(row);
    }
    for id in [vec![0; 16], vec![1; 15], vec![1; 17]] {
        let mut row = valid.clone();
        row[0] = Value::Blob(id.clone());
        invalid_rows.push(row);
        let mut row = valid.clone();
        row[4] = Value::Blob(id);
        invalid_rows.push(row);
    }
    for wrong_type in [Value::Text("x".into()), Value::Integer(1)] {
        let mut row = valid.clone();
        row[0] = wrong_type.clone();
        invalid_rows.push(row);
        let mut row = valid.clone();
        row[4] = wrong_type;
        invalid_rows.push(row);
    }
    for bad_time in [
        Value::Real(1.5),
        Value::Text("not-time".into()),
        Value::Integer(-1),
    ] {
        let mut row = valid.clone();
        row[1] = bad_time;
        invalid_rows.push(row);
    }
    for bad_tag in [
        Value::Text(String::new()),
        Value::Text("a".repeat(65)),
        Value::Text("A".into()),
        Value::Text("é".into()),
        Value::Text("a b".into()),
        Value::Text("a\0b".into()),
    ] {
        let mut row = valid.clone();
        row[2] = bad_tag.clone();
        invalid_rows.push(row);
        let mut row = valid.clone();
        row[3] = bad_tag;
        invalid_rows.push(row);
    }
    for row in &invalid_rows {
        db013_assert_constraint_row(&connection, row);
    }

    for sql in [
        "UPDATE main.audit_event SET operation = 'changed' WHERE event_id = X'01010101010101010101010101010101'",
        "DELETE FROM main.audit_event WHERE event_id = X'01010101010101010101010101010101'",
    ] {
        match connection.execute(sql, []) {
            Err(rusqlite::Error::SqliteFailure(code, _)) => assert_eq!(code.code, rusqlite::ErrorCode::ConstraintViolation),
            other => panic!("AUDIT_MUTATION_DID_NOT_FAIL: {other:?}"),
        }
    }
    drop(connection);
    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert_eq!(schema_version(&reopened, 2).unwrap(), 2);
    assert_eq!(
        reopened
            .query_row("SELECT count(*) FROM main.audit_event", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        reopened
            .query_row("SELECT operation FROM main.audit_event", [], |row| row
                .get::<_, String>(
                0
            ))
            .unwrap(),
        "a"
    );
    assert_eq!(
        reopened
            .query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
}

#[test]
fn db013_audit_errors_hide_driver_payload() {
    use millani_artes::database::{append_audit_event, execute_write_transaction, AuditEventError};

    const MARKER: &str = "DB013_SYNTHETIC_DRIVER_MARKER";
    let directory = common::TempDirectory::new();
    let path = directory.path().join("db013-sanitized-error.sqlite");
    let connection = db013_migrate_v2(open_database(&path).expect("DATABASE_OPEN_FAILED"));
    connection.execute_batch(
        "CREATE TRIGGER db013_fail_insert BEFORE INSERT ON audit_event BEGIN SELECT RAISE(ABORT, 'DB013_SYNTHETIC_DRIVER_MARKER'); END;",
    ).expect("SENTINEL_TRIGGER_CREATE_FAILED");
    let event = db013_event([9; 16], 1, "sale_create", "sale", [10; 16]);
    let (connection, ()) = execute_write_transaction(connection, |transaction| {
        let driver_error = transaction.execute(
            "INSERT INTO main.audit_event (event_id, occurred_at_ms, operation, entity, entity_id) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![event.event_id.as_slice(), event.occurred_at_ms, event.operation, event.entity, event.entity_id.as_slice()],
        ).expect_err("SYNTHETIC_TRIGGER_DID_NOT_FAIL");
        assert!(driver_error.to_string().contains(MARKER));

        let error = append_audit_event(transaction, &event).expect_err("API_ACCEPTED_TRIGGER_FAILURE");
        assert_eq!(error, AuditEventError::InsertFailed);
        assert_eq!(error.to_string(), "audit event could not be inserted");
        assert_eq!(format!("{error:?}"), "InsertFailed");
        assert!(error.source().is_none());
        assert!(!error.to_string().contains(MARKER));
        assert!(!format!("{error:?}").contains(MARKER));

        let invalid = db013_event([0; 16], 1, "sale_create", "sale", [10; 16]);
        let error = append_audit_event(transaction, &invalid).expect_err("INVALID_EVENT_ACCEPTED");
        assert_eq!(error, AuditEventError::InvalidEvent);
        assert_eq!(error.to_string(), "audit event is invalid");
        assert_eq!(format!("{error:?}"), "InvalidEvent");
        assert!(error.source().is_none());

        transaction.execute_batch(
            "DROP TRIGGER db013_fail_insert; CREATE TRIGGER db013_ignore_insert BEFORE INSERT ON audit_event BEGIN SELECT RAISE(IGNORE); END;",
        ).expect("IGNORE_TRIGGER_SETUP_FAILED");
        let ignored = db013_event([11; 16], 1, "sale_create", "sale", [12; 16]);
        let error = append_audit_event(transaction, &ignored).expect_err("IGNORED_INSERT_REPORTED_SUCCESS");
        assert_eq!(error, AuditEventError::InsertFailed);
        let count: i64 = transaction
            .query_row("SELECT count(*) FROM main.audit_event", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
        Ok::<(), AuditEventError>(())
    }).expect("ERROR_SANITIZATION_TRANSACTION_FAILED");
    drop(connection);
    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert_eq!(
        reopened
            .query_row("SELECT count(*) FROM main.audit_event", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
}

type Db014AuditRow = (Vec<u8>, i64, String, String, Vec<u8>);

#[derive(Debug, PartialEq, Eq)]
struct Db014Snapshot {
    database: Db012Snapshot,
    ledger: Vec<i64>,
    probe: Vec<(i64, String)>,
    audit: Vec<Db014AuditRow>,
}

fn db014_snapshot(connection: &Connection) -> Db014Snapshot {
    Db014Snapshot {
        database: db012_snapshot(connection),
        ledger: {
            let mut statement = connection
                .prepare("SELECT version FROM main.schema_migration ORDER BY version")
                .unwrap();
            statement
                .query_map([], |row| row.get(0))
                .unwrap()
                .collect::<rusqlite::Result<_>>()
                .unwrap()
        },
        probe: {
            let mut statement = connection
                .prepare("SELECT id, value FROM main.db014_probe ORDER BY id")
                .unwrap();
            statement
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .unwrap()
                .collect::<rusqlite::Result<_>>()
                .unwrap()
        },
        audit: {
            let mut statement = connection.prepare(
                "SELECT event_id, occurred_at_ms, operation, entity, entity_id FROM main.audit_event ORDER BY event_id",
            ).unwrap();
            statement
                .query_map([], |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                })
                .unwrap()
                .collect::<rusqlite::Result<_>>()
                .unwrap()
        },
    }
}

fn db014_seed(connection: Connection) -> (Connection, Db014Snapshot) {
    let connection = db013_migrate_v2(connection);
    let original = db012_seed_baseline(&connection);
    assert_eq!(original.integrity, "ok");
    assert_eq!(original.foreign_key_violation, None);
    connection.execute_batch(
        "CREATE TABLE db014_probe (id INTEGER PRIMARY KEY, value TEXT NOT NULL); INSERT INTO db014_probe VALUES (1, 'sentinel');",
    ).expect("DB014_FIXTURE_SETUP_FAILED");
    let sentinel = db013_event([21; 16], 100, "sale_create", "sale", [22; 16]);
    let (connection, ()) = millani_artes::database::execute_write_transaction(connection, |tx| {
        millani_artes::database::append_audit_event(tx, &sentinel)
    })
    .expect("DB014_SENTINEL_AUDIT_SETUP_FAILED");
    let snapshot = db014_snapshot(&connection);
    (connection, snapshot)
}

#[test]
fn db014_audit_failure_rolls_back_operation() {
    use millani_artes::database::{
        append_audit_event, execute_write_transaction, AuditEventError, TransactionError,
    };

    let directory = common::TempDirectory::new();
    for (case, event, ignore) in [
        (
            "duplicate",
            db013_event([21; 16], 101, "sale_create", "sale", [23; 16]),
            false,
        ),
        (
            "invalid",
            db013_event([0; 16], 101, "sale_create", "sale", [23; 16]),
            false,
        ),
        (
            "ignore",
            db013_event([24; 16], 101, "sale_create", "sale", [23; 16]),
            true,
        ),
    ] {
        let case_path = directory.path().join(format!("db014-audit-{case}.sqlite"));
        let (connection, _) = db014_seed(open_database(&case_path).expect("DATABASE_OPEN_FAILED"));
        if ignore {
            connection.execute_batch("CREATE TRIGGER db014_ignore_audit BEFORE INSERT ON audit_event BEGIN SELECT RAISE(IGNORE); END;")
                .expect("IGNORE_TRIGGER_SETUP_FAILED");
        }
        let baseline = db014_snapshot(&connection);
        let result = execute_write_transaction(connection, |transaction| {
            transaction
                .execute(
                    "UPDATE main.db014_probe SET value = 'changed' WHERE id = 1",
                    [],
                )
                .expect("PROBE_UPDATE_FAILED");
            transaction
                .execute(
                    "INSERT INTO main.db014_probe VALUES (2, 'new-operation')",
                    [],
                )
                .expect("PROBE_INSERT_FAILED");
            append_audit_event(transaction, &event)
        });
        let expected_error = if case == "invalid" {
            AuditEventError::InvalidEvent
        } else {
            AuditEventError::InsertFailed
        };
        assert!(
            matches!(result, Err(TransactionError::OperationFailed(error)) if error == expected_error)
        );

        let reopened = open_database(&case_path).expect("DATABASE_REOPEN_FAILED");
        assert!(reopened.is_autocommit());
        assert_eq!(
            reopened
                .query_row("PRAGMA foreign_keys", [], |row| row.get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(db014_snapshot(&reopened), baseline);
    }
}

#[test]
fn db014_operation_and_audit_commit_together() {
    use millani_artes::database::{append_audit_event, execute_write_transaction, AuditEventError};

    let directory = common::TempDirectory::new();
    let path = directory.path().join("db014-success.sqlite");
    let (connection, baseline) = db014_seed(open_database(&path).expect("DATABASE_OPEN_FAILED"));
    let observer = open_database(&path).expect("OBSERVER_OPEN_FAILED");
    let event = db013_event([24; 16], 101, "sale_create", "sale", [23; 16]);
    let (connection, committed_value) = execute_write_transaction(connection, |transaction| {
        transaction
            .execute(
                "UPDATE main.db014_probe SET value = 'changed' WHERE id = 1",
                [],
            )
            .expect("PROBE_UPDATE_FAILED");
        transaction
            .execute(
                "INSERT INTO main.db014_probe VALUES (2, 'new-operation')",
                [],
            )
            .expect("PROBE_INSERT_FAILED");
        append_audit_event(transaction, &event)?;
        assert!(!transaction.is_autocommit());
        assert_eq!(
            observer
                .query_row("PRAGMA foreign_keys", [], |row| row.get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(db014_snapshot(&observer), baseline);
        Ok::<_, AuditEventError>(77)
    })
    .expect("DB014_SUCCESS_TRANSACTION_FAILED");
    assert_eq!(committed_value, 77);
    assert!(connection.is_autocommit());
    drop(observer);
    drop(connection);

    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert!(reopened.is_autocommit());
    assert_eq!(
        reopened
            .query_row("PRAGMA foreign_keys", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        1
    );
    let mut expected = baseline;
    expected.probe = vec![(1, "changed".into()), (2, "new-operation".into())];
    expected.audit.push((
        [24; 16].to_vec(),
        101,
        "sale_create".into(),
        "sale".into(),
        [23; 16].to_vec(),
    ));
    assert_eq!(db014_snapshot(&reopened), expected);
}

#[test]
fn db014_operation_failure_rolls_back_audit() {
    use millani_artes::database::{
        append_audit_event, execute_write_transaction, TransactionError,
    };

    let directory = common::TempDirectory::new();
    let path = directory.path().join("db014-operation-failure.sqlite");
    let (connection, baseline) = db014_seed(open_database(&path).expect("DATABASE_OPEN_FAILED"));
    let event = db013_event([24; 16], 101, "sale_create", "sale", [23; 16]);
    let result = execute_write_transaction(connection, |transaction| {
        transaction
            .execute(
                "UPDATE main.db014_probe SET value = 'changed' WHERE id = 1",
                [],
            )
            .expect("PROBE_UPDATE_FAILED");
        append_audit_event(transaction, &event).expect("AUDIT_APPEND_FAILED");
        transaction.execute("INSERT INTO main.db014_probe VALUES (1, 'duplicate')", [])?;
        Ok::<(), rusqlite::Error>(())
    });
    assert!(matches!(result, Err(TransactionError::OperationFailed(_))));
    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert!(reopened.is_autocommit());
    assert_eq!(
        reopened
            .query_row("PRAGMA foreign_keys", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(db014_snapshot(&reopened), baseline);
}

#[test]
fn db014_commit_failure_rolls_back_operation_and_audit() {
    use millani_artes::database::{
        append_audit_event, execute_write_transaction, TransactionError,
    };

    let directory = common::TempDirectory::new();
    let path = directory.path().join("db014-commit-failure.sqlite");
    let (connection, baseline) = db014_seed(open_database(&path).expect("DATABASE_OPEN_FAILED"));
    let event = db013_event([24; 16], 101, "sale_create", "sale", [23; 16]);
    let result = execute_write_transaction(connection, |transaction| {
        transaction
            .execute(
                "UPDATE main.db014_probe SET value = 'changed' WHERE id = 1",
                [],
            )
            .expect("PROBE_UPDATE_FAILED");
        append_audit_event(transaction, &event).expect("AUDIT_APPEND_FAILED");
        transaction.execute("INSERT INTO main.db012_child VALUES (2, 999, 'orphan')", [])?;
        Ok::<_, rusqlite::Error>(())
    });
    assert!(matches!(result, Err(TransactionError::CommitFailed)));
    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert!(reopened.is_autocommit());
    assert_eq!(
        reopened
            .query_row("PRAGMA foreign_keys", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(db014_snapshot(&reopened), baseline);
}

#[test]
fn db014_panic_unwind_rolls_back_operation_and_audit() {
    use millani_artes::database::{append_audit_event, execute_write_transaction};
    use std::panic::{catch_unwind, AssertUnwindSafe};

    let directory = common::TempDirectory::new();
    let path = directory.path().join("db014-panic.sqlite");
    let (connection, baseline) = db014_seed(open_database(&path).expect("DATABASE_OPEN_FAILED"));
    let event = db013_event([24; 16], 101, "sale_create", "sale", [23; 16]);
    let audit_appended = std::cell::Cell::new(false);
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = execute_write_transaction(connection, |transaction| -> rusqlite::Result<()> {
            transaction
                .execute(
                    "UPDATE main.db014_probe SET value = 'changed' WHERE id = 1",
                    [],
                )
                .expect("PROBE_UPDATE_FAILED");
            append_audit_event(transaction, &event).expect("AUDIT_APPEND_FAILED");
            audit_appended.set(true);
            panic!("DB014_SYNTHETIC_UNWIND");
        });
    }));
    assert!(panic.is_err());
    assert!(audit_appended.get());
    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert!(reopened.is_autocommit());
    assert_eq!(
        reopened
            .query_row("PRAGMA foreign_keys", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(db014_snapshot(&reopened), baseline);
}

#[test]
fn fin006_accounts_initialize_empty_and_roundtrip() {
    let directory = common::TempDirectory::new();
    let path = directory.path().join("fin006-accounts-empty.sqlite");
    let connection = open_database(&path).expect("DATABASE_OPEN_FAILED");
    assert_eq!(schema_version(&connection, 3).unwrap(), 0);
    let connection =
        execute_versioned_migration(connection, 1, "").expect("BASELINE_MIGRATION_FAILED");
    let connection = execute_versioned_migration(
        connection,
        millani_artes::database::AUDIT_EVENT_MIGRATION_VERSION,
        millani_artes::database::AUDIT_EVENT_MIGRATION_SQL,
    )
    .expect("AUDIT_EVENT_MIGRATION_FAILED");
    let connection = execute_versioned_migration(
        connection,
        millani_artes::database::ACCOUNTS_MIGRATION_VERSION,
        millani_artes::database::ACCOUNTS_MIGRATION_SQL,
    )
    .expect("ACCOUNTS_MIGRATION_FAILED");

    assert!(connection.is_autocommit());
    assert_eq!(schema_version(&connection, 3).unwrap(), 3);
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
    assert_eq!(versions, [1, 2, 3]);
    let columns: Vec<(String, String, i64, i64, i64)> = {
        let mut statement = connection
            .prepare("PRAGMA main.table_xinfo('accounts')")
            .unwrap();
        statement
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
            .unwrap()
    };
    assert_eq!(
        columns,
        [
            ("id".into(), "BLOB".into(), 1, 1, 0),
            ("name".into(), "TEXT".into(), 1, 0, 0),
            ("account_type".into(), "TEXT".into(), 1, 0, 0),
            ("initial_balance_cents".into(), "INTEGER".into(), 1, 0, 0),
        ]
    );
    let strict: i64 = connection
        .query_row(
            "SELECT strict FROM pragma_table_list WHERE schema = 'main' AND name = 'accounts'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(strict, 1);
    let account_count: i64 = connection
        .query_row("SELECT count(*) FROM main.accounts", [], |row| row.get(0))
        .unwrap();
    assert_eq!(account_count, 0);
    assert_eq!(
        connection
            .query_row("PRAGMA main.integrity_check", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        "ok"
    );

    let account_types = ["cash", "bank_pix", "card", "other"];
    let balances = [i64::MIN, -123456, 0, 1, 123456, i64::MAX];
    let mut expected = Vec::new();
    for account_type in account_types {
        for (balance_index, initial_balance_cents) in balances.into_iter().enumerate() {
            let mut id = [0; 16];
            id[..8].copy_from_slice(&((expected.len() as u64) + 1).to_be_bytes());
            id[15] = 255;
            let name = if balance_index % 2 == 0 {
                "Ateliê Millani 🧵"
            } else {
                "\0Conta"
            };
            connection
                .execute(
                    "INSERT INTO main.accounts (id, name, account_type, initial_balance_cents) VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![id.as_slice(), name, account_type, initial_balance_cents],
                )
                .unwrap();
            expected.push((
                id.to_vec(),
                name.to_owned(),
                account_type.to_owned(),
                initial_balance_cents,
            ));
        }
    }
    let stored: Vec<(Vec<u8>, String, String, i64)> = {
        let mut statement = connection
            .prepare("SELECT id, name, account_type, initial_balance_cents FROM main.accounts ORDER BY id")
            .unwrap();
        statement
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(stored, expected);
    assert!(connection.is_autocommit());
    drop(connection);

    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert_eq!(schema_version(&reopened, 3).unwrap(), 3);
    let reopened_rows: Vec<(Vec<u8>, String, String, i64)> = {
        let mut statement = reopened
            .prepare("SELECT id, name, account_type, initial_balance_cents FROM main.accounts ORDER BY id")
            .unwrap();
        statement
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(reopened_rows, expected);
    assert_eq!(
        reopened
            .query_row("PRAGMA main.integrity_check", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        "ok"
    );
}

#[test]
fn fin006_accounts_upgrade_preserves_v2_audit_event() {
    use millani_artes::database::{append_audit_event, execute_write_transaction, AuditEventError};

    let directory = common::TempDirectory::new();
    let path = directory.path().join("fin006-accounts-upgrade.sqlite");
    let connection = db013_migrate_v2(open_database(&path).expect("DATABASE_OPEN_FAILED"));
    let event = db013_event(
        [0xA1; 16],
        1_765_000_000_123,
        "account_schema_upgrade",
        "account",
        [0xB2; 16],
    );
    let (connection, ()) = execute_write_transaction(connection, |transaction| {
        append_audit_event(transaction, &event)?;
        Ok::<(), AuditEventError>(())
    })
    .expect("AUDIT_EVENT_INSERT_FAILED");
    let read_audit = |connection: &Connection| {
        connection
            .query_row(
                "SELECT event_id, occurred_at_ms, operation, entity, entity_id FROM main.audit_event",
                [],
                |row| {
                    Ok((
                        row.get::<_, Vec<u8>>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, Vec<u8>>(4)?,
                    ))
                },
            )
            .unwrap()
    };
    let read_audit_schema = |connection: &Connection| {
        let mut statement = connection
            .prepare(
                "SELECT type, name, sql FROM main.sqlite_schema WHERE name IN ('audit_event', 'audit_event_no_update', 'audit_event_no_delete') ORDER BY name",
            )
            .unwrap();
        statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap()
    };
    let original_audit = read_audit(&connection);
    let original_audit_schema = read_audit_schema(&connection);
    assert_eq!(schema_version(&connection, 3).unwrap(), 2);
    assert!(connection.is_autocommit());

    let connection = execute_versioned_migration(
        connection,
        millani_artes::database::ACCOUNTS_MIGRATION_VERSION,
        millani_artes::database::ACCOUNTS_MIGRATION_SQL,
    )
    .expect("ACCOUNTS_MIGRATION_FAILED");
    assert_eq!(schema_version(&connection, 3).unwrap(), 3);
    assert!(connection.is_autocommit());
    assert_eq!(read_audit(&connection), original_audit);
    assert_eq!(read_audit_schema(&connection), original_audit_schema);
    assert_eq!(
        connection
            .query_row("SELECT count(*) FROM main.accounts", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        connection
            .query_row("PRAGMA main.integrity_check", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    drop(connection);

    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert_eq!(schema_version(&reopened, 3).unwrap(), 3);
    assert!(reopened.is_autocommit());
    assert_eq!(read_audit(&reopened), original_audit);
    assert_eq!(read_audit_schema(&reopened), original_audit_schema);
    assert_eq!(
        reopened
            .query_row("SELECT count(*) FROM main.accounts", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        reopened
            .query_row("PRAGMA main.integrity_check", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        "ok"
    );
}

#[test]
fn fin007_accounts_constraints_reject_invalid_rows_and_preserve_seed() {
    use rusqlite::types::Value;

    let directory = common::TempDirectory::new();
    let path = directory.path().join("fin007-accounts-invalid.sqlite");
    let connection = db013_migrate_v2(open_database(&path).expect("DATABASE_OPEN_FAILED"));
    let connection = execute_versioned_migration(
        connection,
        millani_artes::database::ACCOUNTS_MIGRATION_VERSION,
        millani_artes::database::ACCOUNTS_MIGRATION_SQL,
    )
    .expect("ACCOUNTS_MIGRATION_FAILED");
    let seed = [
        Value::Blob(vec![0xA1; 16]),
        Value::Text("Conta semente".into()),
        Value::Text("bank_pix".into()),
        Value::Integer(12345),
    ];
    connection
        .execute(
            "INSERT INTO main.accounts (id, name, account_type, initial_balance_cents) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![seed[0], seed[1], seed[2], seed[3]],
        )
        .expect("ACCOUNT_SEED_INSERT_FAILED");

    let assert_state = |connection: &Connection| {
        assert!(connection.is_autocommit());
        let stored: Vec<(Vec<u8>, String, String, i64)> = {
            let mut statement = connection
                .prepare(
                    "SELECT id, name, account_type, initial_balance_cents FROM main.accounts ORDER BY id"
                )
                .unwrap();
            statement
                .query_map([], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
                })
                .unwrap()
                .collect::<rusqlite::Result<_>>()
                .unwrap()
        };
        assert_eq!(
            stored,
            [(
                vec![0xA1; 16],
                "Conta semente".into(),
                "bank_pix".into(),
                12345
            )]
        );
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
        assert_eq!(versions, [1, 2, 3]);
        assert_eq!(schema_version(connection, 3).unwrap(), 3);
        assert_eq!(
            connection
                .query_row("PRAGMA main.integrity_check", [], |row| row
                    .get::<_, String>(0))
                .unwrap(),
            "ok"
        );
    };
    assert_state(&connection);

    let valid = [
        Value::Blob(vec![0xB2; 16]),
        Value::Text("Outra conta".into()),
        Value::Text("cash".into()),
        Value::Integer(7),
    ];
    let mut invalid_rows = Vec::new();
    for column in 0..4 {
        let mut row = valid.clone();
        row[column] = Value::Null;
        invalid_rows.push((format!("null-{column}"), row));
    }
    for id in [
        Value::Blob(vec![0xB1; 15]),
        Value::Blob(vec![]),
        Value::Blob(vec![0xB1; 17]),
        Value::Text("1234567890abcdef".into()),
    ] {
        let mut row = valid.clone();
        row[0] = id;
        invalid_rows.push(("invalid-id".into(), row));
    }
    let mut row = valid.clone();
    row[1] = Value::Text(String::new());
    invalid_rows.push(("empty-name".into(), row));
    let mut row = valid.clone();
    row[1] = Value::Blob(vec![0xFF]);
    invalid_rows.push(("blob-name".into(), row));
    for account_type in [
        Value::Text("unknown".into()),
        Value::Text("Cash".into()),
        Value::Blob(b"cash".to_vec()),
    ] {
        let mut row = valid.clone();
        row[2] = account_type;
        invalid_rows.push(("invalid-account-type".into(), row));
    }
    for balance in [
        Value::Real(1.5),
        Value::Text("not-a-number".into()),
        Value::Blob(vec![1]),
    ] {
        let mut row = valid.clone();
        row[3] = balance;
        invalid_rows.push(("invalid-balance".into(), row));
    }

    for (case, row) in invalid_rows {
        let result = connection.execute(
            "INSERT INTO main.accounts (id, name, account_type, initial_balance_cents) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![row[0], row[1], row[2], row[3]],
        );
        assert!(
            matches!(result, Err(rusqlite::Error::SqliteFailure(error, _)) if error.code == rusqlite::ErrorCode::ConstraintViolation),
            "{case} was not rejected with ConstraintViolation"
        );
        assert_state(&connection);
    }
    drop(connection);

    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert_state(&reopened);
}

#[test]
fn fin007_accounts_constraints_reject_duplicate_id_and_preserve_original() {
    let directory = common::TempDirectory::new();
    let path = directory.path().join("fin007-accounts-duplicate.sqlite");
    let connection = db013_migrate_v2(open_database(&path).expect("DATABASE_OPEN_FAILED"));
    let connection = execute_versioned_migration(
        connection,
        millani_artes::database::ACCOUNTS_MIGRATION_VERSION,
        millani_artes::database::ACCOUNTS_MIGRATION_SQL,
    )
    .expect("ACCOUNTS_MIGRATION_FAILED");
    let seed_id = [0xC1; 16];
    connection
        .execute(
            "INSERT INTO main.accounts (id, name, account_type, initial_balance_cents) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![seed_id.as_slice(), "Conta original", "other", -987],
        )
        .expect("ACCOUNT_SEED_INSERT_FAILED");
    let assert_state = |connection: &Connection| {
        assert!(connection.is_autocommit());
        let stored: (Vec<u8>, String, String, i64) = connection
            .query_row(
                "SELECT id, name, account_type, initial_balance_cents FROM main.accounts",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!(
            stored,
            (
                seed_id.to_vec(),
                "Conta original".into(),
                "other".into(),
                -987
            )
        );
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
        assert_eq!(versions, [1, 2, 3]);
        assert_eq!(schema_version(connection, 3).unwrap(), 3);
        assert_eq!(
            connection
                .query_row("PRAGMA main.integrity_check", [], |row| row
                    .get::<_, String>(0))
                .unwrap(),
            "ok"
        );
    };
    assert_state(&connection);

    let result = connection.execute(
        "INSERT INTO main.accounts (id, name, account_type, initial_balance_cents) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![seed_id.as_slice(), "Nome substituto", "cash", 100],
    );
    assert!(matches!(
        result,
        Err(rusqlite::Error::SqliteFailure(error, _))
            if error.code == rusqlite::ErrorCode::ConstraintViolation
    ));
    assert_state(&connection);
    drop(connection);

    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert_state(&reopened);
}

mod fin008_account_creation_tests {
    use super::*;
    use millani_artes::application::create_account::create_account;
    use millani_artes::domain::ports::{Clock, UuidGenerator};
    use millani_artes::domain::AccountType;
    use millani_artes::infrastructure::accounts::AdapterSqliteAccountStore;
    use std::cell::Cell;
    use std::collections::VecDeque;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    type AuditSnapshotRow = (Vec<u8>, i64, String, String, Vec<u8>);

    #[derive(Debug, PartialEq, Eq)]
    struct Snapshot {
        accounts: Vec<(Vec<u8>, String, String, i64)>,
        audit: Vec<AuditSnapshotRow>,
        versions: Vec<i64>,
        integrity: String,
    }

    struct FixedClock {
        now: SystemTime,
        calls: Cell<usize>,
    }

    impl FixedClock {
        fn new(now: SystemTime) -> Self {
            Self {
                now,
                calls: Cell::new(0),
            }
        }
    }

    impl Clock for FixedClock {
        fn now(&self) -> SystemTime {
            self.calls.set(self.calls.get() + 1);
            self.now
        }
    }

    struct FixedIds {
        values: VecDeque<[u8; 16]>,
        fail_at: Option<usize>,
        calls: usize,
    }

    impl FixedIds {
        fn new(values: impl IntoIterator<Item = [u8; 16]>, fail_at: Option<usize>) -> Self {
            Self {
                values: values.into_iter().collect(),
                fail_at,
                calls: 0,
            }
        }
    }

    impl UuidGenerator for FixedIds {
        type Error = ();

        fn generate(&mut self) -> Result<[u8; 16], Self::Error> {
            let call = self.calls;
            self.calls += 1;
            if self.fail_at == Some(call) {
                return Err(());
            }
            self.values.pop_front().ok_or(())
        }
    }

    fn migrate_accounts(path: &Path) -> Connection {
        let connection = db013_migrate_v2(open_database(path).expect("DATABASE_OPEN_FAILED"));
        execute_versioned_migration(
            connection,
            millani_artes::database::ACCOUNTS_MIGRATION_VERSION,
            millani_artes::database::ACCOUNTS_MIGRATION_SQL,
        )
        .expect("ACCOUNTS_MIGRATION_FAILED")
    }

    fn snapshot(connection: &Connection) -> Snapshot {
        assert!(connection.is_autocommit());
        let accounts = {
            let mut statement = connection
                .prepare(
                    "SELECT id, name, account_type, initial_balance_cents FROM main.accounts ORDER BY id",
                )
                .unwrap();
            statement
                .query_map([], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
                })
                .unwrap()
                .collect::<rusqlite::Result<Vec<_>>>()
                .unwrap()
        };
        let audit = {
            let mut statement = connection
                .prepare(
                    "SELECT event_id, occurred_at_ms, operation, entity, entity_id FROM main.audit_event ORDER BY event_id",
                )
                .unwrap();
            statement
                .query_map([], |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                })
                .unwrap()
                .collect::<rusqlite::Result<Vec<_>>>()
                .unwrap()
        };
        let versions = {
            let mut statement = connection
                .prepare("SELECT version FROM main.schema_migration ORDER BY version")
                .unwrap();
            statement
                .query_map([], |row| row.get(0))
                .unwrap()
                .collect::<rusqlite::Result<Vec<_>>>()
                .unwrap()
        };
        Snapshot {
            accounts,
            audit,
            versions,
            integrity: connection
                .query_row("PRAGMA main.integrity_check", [], |row| row.get(0))
                .unwrap(),
        }
    }

    fn id(value: u128) -> [u8; 16] {
        value.to_be_bytes()
    }

    #[test]
    fn create_account_persists_all_types_balances_and_audit_after_reopen() {
        let directory = common::TempDirectory::new();
        let path = directory.path().join("fin008-create-success.sqlite");
        let connection = migrate_accounts(&path);
        let mut generated = (1..=48).map(id);
        let mut uuid_generator = FixedIds::new(generated.by_ref(), None);
        let clock = FixedClock::new(UNIX_EPOCH + Duration::from_millis(1_765_000_000_123));
        let mut store = AdapterSqliteAccountStore::new(connection);
        let account_types = ["cash", "bank_pix", "card", "other"];
        let balances = [i64::MIN, -123456, 0, 1, 123456, i64::MAX];
        let mut expected_accounts = Vec::new();
        let mut expected_audit = Vec::new();

        for type_key in account_types {
            for (balance_index, balance) in balances.into_iter().enumerate() {
                let account_id = id((expected_accounts.len() as u128) * 2 + 1);
                let event_id = id((expected_accounts.len() as u128) * 2 + 2);
                let name = format!("Conta {type_key} {balance_index}");
                let account_type = match type_key {
                    "cash" => AccountType::Cash,
                    "bank_pix" => AccountType::BankPix,
                    "card" => AccountType::Card,
                    _ => AccountType::Other,
                };
                let account = create_account(
                    name.clone(),
                    account_type,
                    balance,
                    &mut uuid_generator,
                    &clock,
                    &mut store,
                )
                .expect("ACCOUNT_CREATE_FAILED");
                assert_eq!(account.id, account_id);
                expected_accounts.push((account_id.to_vec(), name, type_key.to_owned(), balance));
                expected_audit.push((
                    event_id.to_vec(),
                    1_765_000_000_123,
                    "account_create".into(),
                    "account".into(),
                    account_id.to_vec(),
                ));
            }
        }
        assert_eq!(uuid_generator.calls, 48);
        assert_eq!(clock.calls.get(), 24);
        drop(store);

        let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
        let actual = snapshot(&reopened);
        expected_accounts.sort_by(|left, right| left.0.cmp(&right.0));
        expected_audit.sort_by(|left, right| left.0.cmp(&right.0));
        assert_eq!(actual.accounts, expected_accounts);
        assert_eq!(actual.audit, expected_audit);
        assert_eq!(actual.versions, [1, 2, 3]);
        assert_eq!(actual.integrity, "ok");
        assert_eq!(
            reopened
                .query_row(
                    "SELECT count(*) FROM main.sqlite_schema WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
                    [],
                    |row| row.get::<_, i64>(0),
                )
                .unwrap(),
            3
        );
    }

    #[test]
    fn rejected_inputs_and_clock_do_not_write_to_sqlite() {
        let directory = common::TempDirectory::new();
        let path = directory.path().join("fin008-create-rejections.sqlite");
        let connection = migrate_accounts(&path);
        let baseline = snapshot(&connection);
        let observer = open_database(&path).expect("OBSERVER_OPEN_FAILED");
        let mut store = AdapterSqliteAccountStore::new(connection);

        for name in [String::new(), "\u{2003}\u{00A0}".into()] {
            let clock = FixedClock::new(UNIX_EPOCH);
            let mut uuid_generator = FixedIds::new([id(1), id(2)], None);
            let result = create_account(
                name,
                AccountType::Cash,
                0,
                &mut uuid_generator,
                &clock,
                &mut store,
            );
            assert!(result.is_err());
            assert_eq!(uuid_generator.calls, 0);
            assert_eq!(clock.calls.get(), 0);
            assert_eq!(snapshot(&observer), baseline);
        }

        for fail_at in [0, 1] {
            let clock = FixedClock::new(UNIX_EPOCH);
            let mut uuid_generator = FixedIds::new([id(3), id(4)], Some(fail_at));
            let result = create_account(
                "Conta válida".into(),
                AccountType::BankPix,
                10,
                &mut uuid_generator,
                &clock,
                &mut store,
            );
            assert!(result.is_err());
            assert_eq!(uuid_generator.calls, fail_at + 1);
            assert_eq!(clock.calls.get(), 0);
            assert_eq!(snapshot(&observer), baseline);
        }

        let clock = FixedClock::new(UNIX_EPOCH - Duration::from_millis(1));
        let mut uuid_generator = FixedIds::new([id(5), id(6)], None);
        let result = create_account(
            "Conta válida".into(),
            AccountType::Card,
            10,
            &mut uuid_generator,
            &clock,
            &mut store,
        );
        assert!(result.is_err());
        assert_eq!(uuid_generator.calls, 2);
        assert_eq!(clock.calls.get(), 1);
        assert_eq!(snapshot(&observer), baseline);

        drop(store);
        drop(observer);
        let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
        assert_eq!(snapshot(&reopened), baseline);
    }

    #[test]
    fn failed_atomic_persistence_preserves_rows_and_discards_context() {
        for case in [
            "duplicate-account",
            "ignored-account",
            "duplicate-audit",
            "ignored-audit",
        ] {
            let directory = common::TempDirectory::new();
            let path = directory.path().join(format!("fin008-{case}.sqlite"));
            let connection = migrate_accounts(&path);
            let seed_account_id = [0xA1; 16];
            let duplicate_event_id = [0xE1; 16];
            connection
                .execute(
                    "INSERT INTO main.accounts VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![seed_account_id.as_slice(), "Conta existente", "other", 99],
                )
                .expect("SEED_ACCOUNT_FAILED");
            connection
                .execute(
                    "INSERT INTO main.audit_event VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![
                        duplicate_event_id.as_slice(),
                        77_i64,
                        "seed_event",
                        "account",
                        seed_account_id.as_slice(),
                    ],
                )
                .expect("SEED_AUDIT_FAILED");
            match case {
                "ignored-account" => connection
                    .execute_batch("CREATE TRIGGER ignore_account BEFORE INSERT ON accounts BEGIN SELECT RAISE(IGNORE); END;")
                    .unwrap(),
                "ignored-audit" => connection
                    .execute_batch("CREATE TRIGGER ignore_audit BEFORE INSERT ON audit_event BEGIN SELECT RAISE(IGNORE); END;")
                    .unwrap(),
                _ => {}
            }
            let baseline = snapshot(&connection);
            let observer = open_database(&path).expect("OBSERVER_OPEN_FAILED");
            let (account_id, event_id) = match case {
                "duplicate-account" => (seed_account_id, [0xD1; 16]),
                "duplicate-audit" => ([0xD2; 16], duplicate_event_id),
                "ignored-account" => ([0xD3; 16], [0xD4; 16]),
                _ => ([0xD5; 16], [0xD6; 16]),
            };
            let mut uuid_generator = FixedIds::new([account_id, event_id], None);
            let clock = FixedClock::new(UNIX_EPOCH + Duration::from_millis(100));
            let mut store = AdapterSqliteAccountStore::new(connection);
            let result = create_account(
                "Conta candidata".into(),
                AccountType::Cash,
                123,
                &mut uuid_generator,
                &clock,
                &mut store,
            );
            assert!(result.is_err(), "{case} incorrectly returned success");
            assert_eq!(snapshot(&observer), baseline, "{case} changed visible rows");

            let mut retry_ids = FixedIds::new([[0xF1; 16], [0xF2; 16]], None);
            let retry_clock = FixedClock::new(UNIX_EPOCH + Duration::from_millis(101));
            let retry = create_account(
                "Não repetir".into(),
                AccountType::Cash,
                124,
                &mut retry_ids,
                &retry_clock,
                &mut store,
            );
            assert!(retry.is_err(), "{case} reused a discarded connection");
            assert_eq!(retry_ids.calls, 2);
            assert_eq!(snapshot(&observer), baseline);

            drop(store);
            drop(observer);
            let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
            assert_eq!(
                snapshot(&reopened),
                baseline,
                "{case} changed reopened rows"
            );
        }
    }
}
