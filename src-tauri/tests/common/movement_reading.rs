use super::exit_persistence::{assert_health, setup, snapshot, FixedClock, FixedIds};
use millani_artes::application::create_exit::create_exit;
use millani_artes::application::list_movements::{
    list_movements, ListMovementsError, MovementReadStore,
};
use millani_artes::database::open_database;
use millani_artes::domain::{Context, Movement};
use millani_artes::infrastructure::movements::{AdapterSqliteMovementStore, MovementReadError};
use rusqlite::{types::Value, Connection};

fn assert_app_error(result: Result<Vec<Movement>, ListMovementsError>) {
    let error = match result {
        Err(e) => e,
        Ok(_) => panic!("invalid history admitted"),
    };
    assert_eq!(format!("{error}"), "movements could not be read");
    assert_eq!(format!("{error:?}"), "ListMovementsError");
    assert!(std::error::Error::source(&error).is_none());
}
fn assert_store_error(result: Result<Vec<Movement>, MovementReadError>) {
    let error = match result {
        Err(e) => e,
        Ok(_) => panic!("invalid history admitted"),
    };
    assert_eq!(format!("{error}"), "movement history could not be read");
    assert_eq!(format!("{error:?}"), "MovementReadError");
    assert!(std::error::Error::source(&error).is_none());
}
#[test]
fn empty_real_history_is_read_only_and_ignores_temp_shadow() {
    let (_directory, path, _source) = setup();
    let connection = open_database(&path).unwrap();
    let before = snapshot(&connection);
    connection
        .execute_batch(
            "CREATE TEMP TABLE movements(id INTEGER); INSERT INTO temp.movements VALUES(123);",
        )
        .unwrap();
    connection.pragma_update(None, "query_only", true).unwrap();
    assert_health(&connection);
    let store = AdapterSqliteMovementStore::new(connection);
    assert!(list_movements(&store).unwrap().is_empty());
    assert!(list_movements(&store).unwrap().is_empty());
    assert_eq!(snapshot(&open_database(&path).unwrap()), before);
}
#[test]
fn malformed_rows_or_missing_schema_fail_whole_read_without_private_errors() {
    let missing = AdapterSqliteMovementStore::new(Connection::open_in_memory().unwrap());
    assert_app_error(list_movements(&missing));
    assert_store_error(missing.read_all_movements());
    let valid = vec![
        Value::Blob(vec![0x10; 16]),
        Value::Text("entry".into()),
        Value::Integer(5),
        Value::Text("home".into()),
        Value::Blob(vec![0xA1; 16]),
        Value::Null,
        Value::Null,
    ];
    let mut cases = Vec::new();
    for (column, value) in [
        (0, Value::Blob(vec![0x20; 15])),
        (0, Value::Blob(vec![0x20; 17])),
        (0, Value::Text("PRIVATE_FIXTURE".into())),
        (1, Value::Text("unknown".into())),
        (3, Value::Text("unknown".into())),
        (3, Value::Null),
        (2, Value::Real(1.5)),
        (2, Value::Text("PRIVATE_FIXTURE".into())),
        (2, Value::Null),
        (4, Value::Null),
        (4, Value::Blob(vec![0xA1; 15])),
        (5, Value::Blob(vec![0xA2; 16])),
        (6, Value::Blob(vec![0xA2; 16])),
    ] {
        let mut row = valid.clone();
        row[0] = Value::Blob(vec![0x20; 16]);
        row[column] = value;
        cases.push(row);
    }
    for kind in ["entry", "exit", "transfer"] {
        for amount in [0, -1, i64::MIN] {
            let mut row = valid.clone();
            row[0] = Value::Blob(vec![0x20; 16]);
            row[1] = Value::Text(kind.into());
            row[2] = Value::Integer(amount);
            if kind == "transfer" {
                row[4] = Value::Null;
                row[5] = Value::Blob(vec![0xA1; 16]);
                row[6] = Value::Blob(vec![0xA2; 16]);
            }
            cases.push(row);
        }
    }
    for (column, value) in [
        (4, Value::Blob(vec![0xA1; 16])),
        (5, Value::Null),
        (6, Value::Null),
        (5, Value::Blob(vec![0xA1; 15])),
        (6, Value::Blob(vec![0xA2; 17])),
        (6, Value::Blob(vec![0xA1; 16])),
    ] {
        let mut row = valid.clone();
        row[0] = Value::Blob(vec![0x20; 16]);
        row[1] = Value::Text("transfer".into());
        row[4] = Value::Null;
        row[5] = Value::Blob(vec![0xA1; 16]);
        row[6] = Value::Blob(vec![0xA2; 16]);
        row[column] = value;
        cases.push(row);
    }
    for (index, bad) in cases.iter().enumerate() {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch("CREATE TABLE main.movements(id,kind,amount_cents,context,account_id,source_account_id,destination_account_id);").unwrap();
        connection
            .execute(
                "INSERT INTO main.movements VALUES(?1,?2,?3,?4,?5,?6,?7)",
                rusqlite::params_from_iter(valid.iter()),
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO main.movements VALUES(?1,?2,?3,?4,?5,?6,?7)",
                rusqlite::params_from_iter(bad.iter()),
            )
            .unwrap();
        let store = AdapterSqliteMovementStore::new(connection);
        let result = list_movements(&store);
        assert!(
            result.is_err(),
            "invalid fixture {index} returned partial or complete success"
        );
        assert_app_error(result);
        assert_store_error(store.read_all_movements());
    }
}
#[test]
fn poisoned_write_adapter_cannot_read_and_fresh_reader_sees_rollback() {
    let (_directory, path, _source) = setup();
    let before = snapshot(&open_database(&path).unwrap());
    let mut store = AdapterSqliteMovementStore::new(open_database(&path).unwrap());
    assert!(create_exit(
        [0xFE; 16],
        1,
        Context::Home,
        &mut FixedIds::new([0x31; 16], [0x41; 16]),
        &FixedClock,
        &mut store
    )
    .is_err());
    assert_app_error(list_movements(&store));
    assert_store_error(store.read_all_movements());
    let reopened = open_database(&path).unwrap();
    assert_health(&reopened);
    assert_eq!(snapshot(&reopened), before);
    assert_eq!(
        reopened
            .query_row("SELECT initial_balance_cents FROM main.accounts", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        -1234
    );
    assert!(list_movements(&AdapterSqliteMovementStore::new(reopened))
        .unwrap()
        .is_empty());
}
