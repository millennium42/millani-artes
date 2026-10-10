mod common;

use rusqlite::Connection;

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
