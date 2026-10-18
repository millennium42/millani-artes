use super::entry_persistence::{setup, snapshot, FixedClock, FixedIds, Snapshot};
use super::TempDirectory;
use millani_artes::application::create_category::create_category;
use millani_artes::application::create_entry::create_entry;
use millani_artes::application::deactivate_category::deactivate_category;
use millani_artes::application::rename_category::rename_category;
use millani_artes::database::{
    execute_versioned_migration, open_database, schema_version, ACCOUNTS_MIGRATION_SQL,
    ACCOUNTS_MIGRATION_VERSION, AUDIT_EVENT_MIGRATION_SQL, AUDIT_EVENT_MIGRATION_VERSION,
    CATEGORIES_MIGRATION_SQL, CATEGORIES_MIGRATION_VERSION, MOVEMENTS_MIGRATION_SQL,
    MOVEMENTS_MIGRATION_VERSION, MOVEMENT_CATEGORIES_MIGRATION_SQL,
    MOVEMENT_CATEGORIES_MIGRATION_VERSION,
};
use millani_artes::domain::Context;
use millani_artes::infrastructure::categories::AdapterSqliteCategoryStore;
use millani_artes::infrastructure::movements::AdapterSqliteMovementStore;
use rusqlite::{types::Value, Connection};

fn rows(connection: &Connection, sql: &str) -> Vec<Vec<Value>> {
    let mut statement = connection.prepare(sql).unwrap();
    let columns = statement.column_count();
    statement
        .query_map([], |row| {
            (0..columns)
                .map(|index| row.get(index))
                .collect::<rusqlite::Result<Vec<Value>>>()
        })
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap()
}

fn state(connection: &Connection) -> (Snapshot, Vec<Vec<Value>>, Vec<Vec<Value>>) {
    (
        snapshot(connection),
        rows(
            connection,
            "SELECT id,name,is_active FROM main.categories ORDER BY id",
        ),
        rows(
            connection,
            "SELECT id,category_id FROM main.movements ORDER BY id",
        ),
    )
}

fn health(connection: &Connection) {
    assert!(connection.is_autocommit());
    assert_eq!(schema_version(connection, 6).unwrap(), 6);
    assert_eq!(snapshot(connection).3, [1, 2, 3, 4, 5, 6]);
    assert_eq!(
        connection
            .query_row("PRAGMA main.integrity_check", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    assert!(rows(connection, "PRAGMA main.foreign_key_check").is_empty());
}

fn audit(event: u8, operation: &str) -> Vec<Value> {
    vec![
        Value::Blob(vec![event; 16]),
        Value::Integer(1234),
        Value::Text(operation.into()),
        Value::Text("category".into()),
        Value::Blob(vec![0x11; 16]),
    ]
}

fn reject_parent_mutation(connection: &Connection) {
    let before = state(connection);
    assert!(connection
        .execute(
            "DELETE FROM main.categories WHERE id=?1",
            rusqlite::params![[0x11u8; 16].as_slice()]
        )
        .is_err());
    assert_eq!(state(connection), before);
    assert!(connection
        .execute(
            "UPDATE main.categories SET id=?1 WHERE id=?2",
            rusqlite::params![[0x99u8; 16].as_slice(), [0x11u8; 16].as_slice()]
        )
        .is_err());
    assert_eq!(state(connection), before);
    health(connection);
}

#[test]
fn empty_database_adds_nullable_category_reference_and_reopens() {
    let directory = TempDirectory::new();
    let path = directory.path().join("cat007-empty.sqlite");
    let connection = open_database(&path).unwrap();
    assert!(rows(&connection, "SELECT name FROM main.sqlite_schema").is_empty());
    let mut connection = execute_versioned_migration(connection, 1, "").unwrap();
    for (version, sql) in [
        (AUDIT_EVENT_MIGRATION_VERSION, AUDIT_EVENT_MIGRATION_SQL),
        (ACCOUNTS_MIGRATION_VERSION, ACCOUNTS_MIGRATION_SQL),
        (CATEGORIES_MIGRATION_VERSION, CATEGORIES_MIGRATION_SQL),
        (MOVEMENTS_MIGRATION_VERSION, MOVEMENTS_MIGRATION_SQL),
        (
            MOVEMENT_CATEGORIES_MIGRATION_VERSION,
            MOVEMENT_CATEGORIES_MIGRATION_SQL,
        ),
    ] {
        connection = execute_versioned_migration(connection, version, sql).unwrap();
    }
    health(&connection);
    let columns = rows(&connection, "PRAGMA main.table_xinfo(movements)");
    assert_eq!(columns.len(), 8);
    assert_eq!(
        columns[7],
        vec![
            Value::Integer(7),
            Value::Text("category_id".into()),
            Value::Text("BLOB".into()),
            Value::Integer(0),
            Value::Null,
            Value::Integer(0),
            Value::Integer(0),
        ]
    );
    let fks = rows(&connection, "PRAGMA main.foreign_key_list(movements)");
    assert_eq!(fks.len(), 4);
    let reference = fks
        .iter()
        .filter(|row| row[2] == Value::Text("categories".into()))
        .collect::<Vec<_>>();
    assert_eq!(reference.len(), 1);
    assert_eq!(
        &reference[0][2..],
        &[
            Value::Text("categories".into()),
            Value::Text("category_id".into()),
            Value::Text("id".into()),
            Value::Text("NO ACTION".into()),
            Value::Text("NO ACTION".into()),
            Value::Text("NONE".into()),
        ]
    );
    for name in ["account_id", "source_account_id", "destination_account_id"] {
        assert!(fks
            .iter()
            .any(|row| row[2] == Value::Text("accounts".into())
                && row[3] == Value::Text(name.into())
                && row[4] == Value::Text("id".into())
                && row[5] == Value::Text("NO ACTION".into())
                && row[6] == Value::Text("NO ACTION".into())));
    }
    let before = state(&connection);
    assert!(before.0 .0.is_empty() && before.0 .1.is_empty() && before.0 .2.is_empty());
    drop(connection);
    let reopened = open_database(&path).unwrap();
    health(&reopened);
    assert_eq!(state(&reopened), before);
}

#[test]
fn upgraded_movement_keeps_reference_after_rename_and_deactivate() {
    let (_directory, path, account_id) = setup();
    let mut category_store = AdapterSqliteCategoryStore::new(open_database(&path).unwrap());
    let old_name = "  Café '🧵'\0  ";
    create_category(
        old_name.into(),
        &mut FixedIds::new([0x11; 16], [0x21; 16]),
        &FixedClock,
        &mut category_store,
    )
    .unwrap();
    drop(category_store);
    let mut movement_store = AdapterSqliteMovementStore::new(open_database(&path).unwrap());
    create_entry(
        account_id,
        4321,
        Context::Home,
        &mut FixedIds::new([0x31; 16], [0x32; 16]),
        &FixedClock,
        &mut movement_store,
    )
    .unwrap();
    drop(movement_store);
    let connection = open_database(&path).unwrap();
    let old = snapshot(&connection);
    let old_categories = rows(
        &connection,
        "SELECT id,name,is_active FROM main.categories ORDER BY id",
    );
    let connection = execute_versioned_migration(
        connection,
        MOVEMENT_CATEGORIES_MIGRATION_VERSION,
        MOVEMENT_CATEGORIES_MIGRATION_SQL,
    )
    .unwrap();
    health(&connection);
    let upgraded = state(&connection);
    assert_eq!(upgraded.0 .0, old.0);
    assert_eq!(upgraded.0 .1, old.1);
    assert_eq!(upgraded.0 .2, old.2);
    assert_eq!(old.3, [1, 2, 3, 4, 5]);
    assert_eq!(upgraded.1, old_categories);
    assert_eq!(
        upgraded.2,
        vec![vec![Value::Blob(vec![0x31; 16]), Value::Null]]
    );
    for invalid in [
        Value::Integer(1),
        Value::Real(1.5),
        Value::Text("invalid".into()),
        Value::Blob(vec![]),
        Value::Blob(vec![0; 15]),
        Value::Blob(vec![0; 17]),
        Value::Blob(vec![0xFF; 16]),
    ] {
        assert!(connection
            .execute(
                "UPDATE main.movements SET category_id=?1 WHERE id=?2",
                rusqlite::params![invalid, [0x31u8; 16].as_slice()]
            )
            .is_err());
        assert_eq!(state(&connection), upgraded);
        health(&connection);
    }
    // Fixture assignment uses the real new FK; no UI/IPC assignment is exercised.
    assert_eq!(
        connection
            .execute(
                "UPDATE main.movements SET category_id=?1 WHERE id=?2",
                rusqlite::params![[0x11u8; 16].as_slice(), [0x31u8; 16].as_slice()]
            )
            .unwrap(),
        1
    );
    let linked = state(&connection);
    reject_parent_mutation(&connection);
    drop(connection);
    let mut store = AdapterSqliteCategoryStore::new(open_database(&path).unwrap());
    let new_name = "  Novo '🧵'\0  ";
    let mut rename_ids = FixedIds::new([1; 16], [0xF1; 16]);
    let renamed = rename_category(
        [0x11; 16],
        new_name.into(),
        &mut rename_ids,
        &FixedClock,
        &mut store,
    )
    .unwrap();
    assert_eq!(renamed.id, [0x11; 16]);
    assert_eq!(renamed.name, new_name);
    assert!(renamed.is_active);
    assert_eq!(rename_ids.calls, 1);
    let mut deactivate_ids = FixedIds::new([2; 16], [0xF1; 16]);
    let inactive =
        deactivate_category([0x11; 16], &mut deactivate_ids, &FixedClock, &mut store).unwrap();
    assert_eq!(inactive.id, [0x11; 16]);
    assert_eq!(inactive.name, new_name);
    assert!(!inactive.is_active);
    assert_eq!(deactivate_ids.calls, 1);
    drop(store);
    let connection = open_database(&path).unwrap();
    health(&connection);
    let joined: (Vec<u8>, Vec<u8>, String, bool) = connection.query_row(
        "SELECT m.id,c.id,c.name,c.is_active FROM main.movements m JOIN main.categories c ON m.category_id=c.id WHERE m.id=?1",
        rusqlite::params![[0x31u8; 16].as_slice()],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    ).unwrap();
    assert_eq!(
        joined,
        (vec![0x31; 16], vec![0x11; 16], new_name.into(), false)
    );
    let after = state(&connection);
    assert_eq!(after.0 .0, old.0);
    assert_eq!(after.0 .1, old.1);
    let mut expected_events = vec![audit(1, "category_rename"), audit(2, "category_deactivate")];
    expected_events.extend(old.2);
    assert_eq!(after.0 .2, expected_events);
    assert_eq!(after.0 .3, linked.0 .3);
    assert_eq!(
        after.1,
        vec![vec![
            Value::Blob(vec![0x11; 16]),
            Value::Text(new_name.into()),
            Value::Integer(0)
        ]]
    );
    assert_eq!(after.2, linked.2);
    reject_parent_mutation(&connection);
}
