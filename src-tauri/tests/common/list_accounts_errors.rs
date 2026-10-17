use super::fin008_account_creation_tests::{migrate_accounts, snapshot, FixedClock, FixedIds};
use super::fin009_list_accounts_tests::schema;
use super::*;
use common::TempDirectory;
use millani_artes::application::create_account::create_account;
use millani_artes::application::list_accounts::list_accounts;
use millani_artes::domain::AccountType;
use millani_artes::infrastructure::accounts::AdapterSqliteAccountStore;
use std::time::UNIX_EPOCH;

#[test]
fn fin009_list_accounts_missing_table_error_static() {
    let tmp = TempDirectory::new();
    let path = tmp.path().join("fin009_missing.sqlite");
    let conn = open_database(&path).expect("open");
    let observer = open_database(&path).unwrap();
    let before_schema = schema(&observer);
    let store = AdapterSqliteAccountStore::new(conn);
    let err = match list_accounts(&store) {
        Ok(_) => panic!("expected error"),
        Err(e) => e,
    };
    assert_eq!(format!("{}", err), "accounts could not be read");
    let err2 = match list_accounts(&store) {
        Ok(_) => panic!("expected error"),
        Err(e) => e,
    };
    assert_eq!(format!("{}", err2), "accounts could not be read");
    assert_eq!(before_schema, schema(&observer));
}

#[test]
fn fin009_create_success_duplicate_id_atomic_failure() {
    let tmp = TempDirectory::new();
    let path = tmp.path().join("fin009_dup.sqlite");
    migrate_accounts(&path);
    let conn = rusqlite::Connection::open(&path).unwrap();
    let mut store = AdapterSqliteAccountStore::new(conn);
    let mut ids = FixedIds::new(vec![[1u8; 16], [2u8; 16], [1u8; 16], [3u8; 16]], None);
    let clock = FixedClock::new(UNIX_EPOCH);
    let acc = create_account(
        "A".into(),
        AccountType::Cash,
        100,
        &mut ids,
        &clock,
        &mut store,
    )
    .expect("first create");
    assert_eq!(acc.id, [1u8; 16]);
    let observer = open_database(&path).unwrap();
    let before_snapshot = snapshot(&observer);
    let before_schema = schema(&observer);
    let res = create_account(
        "B".into(),
        AccountType::Cash,
        200,
        &mut ids,
        &clock,
        &mut store,
    );
    assert!(res.is_err());
    let err = match list_accounts(&store) {
        Ok(_) => panic!("expected error"),
        Err(e) => e,
    };
    assert_eq!(format!("{}", err), "accounts could not be read");
    assert_eq!(before_snapshot, snapshot(&observer));
    assert_eq!(before_schema, schema(&observer));
}

#[test]
fn fin009_list_error_on_invalid_id_length_or_enum() {
    let tmp1 = TempDirectory::new();
    let path1 = tmp1.path().join("fin009_invalid_id.sqlite");
    migrate_accounts(&path1);
    let conn1 = rusqlite::Connection::open(&path1).unwrap();
    conn1
        .execute_batch("PRAGMA ignore_check_constraints=ON;")
        .unwrap();
    conn1.execute(
        "INSERT INTO main.accounts (id, name, account_type, initial_balance_cents) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![[1u8;16].as_slice(), "valid", "cash", 100i64],
    ).unwrap();
    let short_id = [2u8; 15];
    conn1.execute(
        "INSERT INTO main.accounts (id, name, account_type, initial_balance_cents) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![short_id.as_slice(), "invalid", "cash", 200i64],
    ).unwrap();
    let before_snapshot1 = snapshot(&conn1);
    let before_schema1 = schema(&conn1);
    let store1 = AdapterSqliteAccountStore::new(rusqlite::Connection::open(&path1).unwrap());
    let err = match list_accounts(&store1) {
        Ok(_) => panic!("expected error"),
        Err(e) => e,
    };
    assert_eq!(format!("{}", err), "accounts could not be read");
    assert_eq!(before_snapshot1, snapshot(&conn1));
    assert_eq!(before_schema1, schema(&conn1));

    let tmp2 = TempDirectory::new();
    let path2 = tmp2.path().join("fin009_unknown_enum.sqlite");
    migrate_accounts(&path2);
    let conn2 = rusqlite::Connection::open(&path2).unwrap();
    conn2
        .execute_batch("PRAGMA ignore_check_constraints=ON;")
        .unwrap();
    conn2.execute(
        "INSERT INTO main.accounts (id, name, account_type, initial_balance_cents) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![[1u8;16].as_slice(), "valid", "cash", 100i64],
    ).unwrap();
    conn2.execute(
        "INSERT INTO main.accounts (id, name, account_type, initial_balance_cents) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![[2u8;16].as_slice(), "invalid", "unknown_type", 200i64],
    ).unwrap();
    let before_snapshot2 = snapshot(&conn2);
    let before_schema2 = schema(&conn2);
    let store2 = AdapterSqliteAccountStore::new(rusqlite::Connection::open(&path2).unwrap());
    let err2 = match list_accounts(&store2) {
        Ok(_) => panic!("expected error"),
        Err(e) => e,
    };
    assert_eq!(format!("{}", err2), "accounts could not be read");
    assert_eq!(before_snapshot2, snapshot(&conn2));
    assert_eq!(before_schema2, schema(&conn2));
}

#[test]
fn fin009_list_error_on_unreadable_types() {
    use rusqlite::types::Value;

    fn raw_rows(connection: &Connection) -> Vec<(String, String, String, String)> {
        let mut statement = connection.prepare(
            "SELECT quote(id), quote(name), quote(account_type), quote(initial_balance_cents) FROM main.accounts ORDER BY rowid",
        ).unwrap();
        let rows = statement
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .unwrap();
        rows.collect::<rusqlite::Result<_>>().unwrap()
    }

    for (column, bad_value) in [
        (0, Value::Text("bad_id".into())),
        (1, Value::Blob(vec![0x41])),
        (1, Value::Null),
        (2, Value::Blob(b"cash".to_vec())),
        (3, Value::Text("100".into())),
    ] {
        let tmp = TempDirectory::new();
        let path = tmp.path().join("fin009_unreadable.sqlite");
        let conn = migrate_accounts(&path);
        // No affinities: each fixture retains the SQLite type being tested.
        conn.execute_batch(
            "DROP TABLE main.accounts; CREATE TABLE main.accounts (id, name, account_type, initial_balance_cents);",
        ).unwrap();
        let valid = [
            Value::Blob(vec![1; 16]),
            Value::Text("valid".into()),
            Value::Text("cash".into()),
            Value::Integer(100),
        ];
        let insert = "INSERT INTO main.accounts (id, name, account_type, initial_balance_cents) VALUES (?1, ?2, ?3, ?4)";
        conn.execute(insert, rusqlite::params_from_iter(valid.iter()))
            .unwrap();
        let reader = AdapterSqliteAccountStore::new(open_database(&path).unwrap());
        let baseline = list_accounts(&reader).expect("valid seed must be readable");
        assert_eq!(baseline.len(), 1);
        assert_eq!(baseline[0].initial_balance_cents, 100);
        drop(reader);

        let mut invalid = valid;
        invalid[0] = Value::Blob(vec![2; 16]);
        invalid[column] = bad_value;
        conn.execute(insert, rusqlite::params_from_iter(invalid.iter()))
            .unwrap();
        let before_rows = raw_rows(&conn);
        let before_schema = schema(&conn);
        let reader = AdapterSqliteAccountStore::new(open_database(&path).unwrap());
        let error = match list_accounts(&reader) {
            Ok(_) => panic!("expected whole-read error for column {column}"),
            Err(error) => error,
        };
        assert_eq!(error.to_string(), "accounts could not be read");
        assert_eq!(format!("{error:?}"), "ListAccountsError");
        assert!(std::error::Error::source(&error).is_none());
        assert_eq!(before_rows, raw_rows(&conn));
        assert_eq!(before_schema, schema(&conn));
    }
}

#[test]
fn fin009_list_success_on_empty_name_and_min_cents() {
    let tmp = TempDirectory::new();
    let path = tmp.path().join("fin009_empty_min.sqlite");
    migrate_accounts(&path);
    let conn = rusqlite::Connection::open(&path).unwrap();
    conn.execute_batch("PRAGMA ignore_check_constraints=ON;")
        .unwrap();
    conn.execute(
        "INSERT INTO main.accounts (id, name, account_type, initial_balance_cents) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![[1u8;16].as_slice(), "", "cash", i64::MIN],
    ).unwrap();
    let before_snapshot = snapshot(&conn);
    let before_schema = schema(&conn);
    let store = AdapterSqliteAccountStore::new(rusqlite::Connection::open(&path).unwrap());
    let accounts = list_accounts(&store).expect("list should succeed");
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].id, [1; 16]);
    assert!(matches!(accounts[0].account_type, AccountType::Cash));
    assert_eq!(accounts[0].name, "");
    assert_eq!(accounts[0].initial_balance_cents, i64::MIN);
    assert_eq!(before_snapshot, snapshot(&conn));
    assert_eq!(before_schema, schema(&conn));
}
