pub(super) use super::entry_persistence::{snapshot, FixedClock, FixedIds};
use super::TempDirectory;
use millani_artes::application::create_exit::{create_exit, ExitCreationStore};
use millani_artes::database::{
    execute_versioned_migration, open_database, MOVEMENT_CATEGORIES_MIGRATION_SQL,
    MOVEMENT_CATEGORIES_MIGRATION_VERSION,
};
use millani_artes::domain::{Context, Movement, MovementKind};
use millani_artes::infrastructure::movements::AdapterSqliteMovementStore;
use rusqlite::{types::Value, Connection};
use std::error::Error;
use std::path::PathBuf;

pub(super) fn setup() -> (TempDirectory, PathBuf, [u8; 16]) {
    let (directory, path, account_id) = super::entry_persistence::setup();
    drop(
        execute_versioned_migration(
            open_database(&path).unwrap(),
            MOVEMENT_CATEGORIES_MIGRATION_VERSION,
            MOVEMENT_CATEGORIES_MIGRATION_SQL,
        )
        .unwrap(),
    );
    (directory, path, account_id)
}

pub(super) fn assert_health(connection: &Connection) {
    assert!(connection.is_autocommit());
    assert_eq!(snapshot(connection).3, [1, 2, 3, 4, 5, 6]);
    assert_eq!(
        connection
            .query_row("PRAGMA main.integrity_check", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    assert!(connection
        .prepare("PRAGMA main.foreign_key_check")
        .unwrap()
        .query([])
        .unwrap()
        .next()
        .unwrap()
        .is_none());
    assert_eq!(
        connection
            .query_row(
                "SELECT COUNT(*) FROM main.movements WHERE category_id IS NOT NULL",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        0
    );
    assert_eq!(
        connection
            .query_row("SELECT COUNT(*) FROM main.categories", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
}

#[test]
fn exit_use_case_persists_both_contexts_and_audit_in_one_database() {
    let (_directory, path, account_id) = setup();
    let mut store = AdapterSqliteMovementStore::new(open_database(&path).unwrap());
    for (movement_id, audit_id, amount, context) in [
        ([0x11; 16], [0x21; 16], i64::MAX, Context::Home),
        ([0x12; 16], [0x22; 16], 1, Context::Factory),
    ] {
        let mut ids = FixedIds::new(movement_id, audit_id);
        let movement = create_exit(
            account_id,
            amount,
            context,
            &mut ids,
            &FixedClock,
            &mut store,
        )
        .expect("persist exit");
        assert_eq!(movement.id, movement_id);
        assert_eq!(movement.amount_cents, amount);
        assert!(
            matches!(&movement.kind, MovementKind::Exit { account_id: id } if *id == account_id)
        );
        assert_eq!(ids.calls, 2);
    }
    drop(store);
    let connection = open_database(&path).unwrap();
    assert_health(&connection);
    let (accounts, movements, audit, _) = snapshot(&connection);
    assert_eq!(
        accounts,
        vec![vec![
            Value::Blob(account_id.to_vec()),
            Value::Text("Conta sintética".into()),
            Value::Text("bank_pix".into()),
            Value::Integer(-1234),
        ]]
    );
    assert_eq!(
        movements,
        vec![
            vec![
                Value::Blob(vec![0x11; 16]),
                Value::Text("exit".into()),
                Value::Integer(i64::MAX),
                Value::Text("home".into()),
                Value::Blob(account_id.to_vec()),
                Value::Null,
                Value::Null
            ],
            vec![
                Value::Blob(vec![0x12; 16]),
                Value::Text("exit".into()),
                Value::Integer(1),
                Value::Text("factory".into()),
                Value::Blob(account_id.to_vec()),
                Value::Null,
                Value::Null
            ],
        ]
    );
    assert_eq!(
        audit,
        vec![
            vec![
                Value::Blob(vec![0x21; 16]),
                Value::Integer(1234),
                Value::Text("movement_exit_create".into()),
                Value::Text("movement".into()),
                Value::Blob(vec![0x11; 16])
            ],
            vec![
                Value::Blob(vec![0x22; 16]),
                Value::Integer(1234),
                Value::Text("movement_exit_create".into()),
                Value::Text("movement".into()),
                Value::Blob(vec![0x12; 16])
            ],
            vec![
                Value::Blob(vec![0xB2; 16]),
                Value::Integer(1234),
                Value::Text("account_create".into()),
                Value::Text("account".into()),
                Value::Blob(account_id.to_vec())
            ],
        ]
    );
}

#[test]
fn invalid_direct_movements_leave_store_usable_and_database_unchanged() {
    let (_directory, path, account_id) = setup();
    let baseline = {
        let connection = open_database(&path).unwrap();
        snapshot(&connection)
    };
    let mut store = AdapterSqliteMovementStore::new(open_database(&path).unwrap());
    for case in 0..5 {
        let kind = match case {
            3 => MovementKind::Entry { account_id },
            4 => MovementKind::Transfer {
                source_account_id: account_id,
                destination_account_id: account_id,
            },
            _ => MovementKind::Exit { account_id },
        };
        let context = if case % 2 == 0 {
            Context::Home
        } else {
            Context::Factory
        };
        let movement = Movement {
            id: [0x30 + case; 16],
            kind,
            amount_cents: match case {
                0 => 0,
                1 => -1,
                2 => i64::MIN,
                _ => 1,
            },
            context,
        };
        let error = store
            .persist_exit_with_audit(&movement, [0x40 + case; 16], 1234)
            .expect_err("invalid movement rejected");
        assert_eq!(format!("{error}"), "exit could not be stored");
        assert_eq!(format!("{error:?}"), "ExitStoreError");
        assert!(error.source().is_none());
        let observer = open_database(&path).unwrap();
        assert_eq!(snapshot(&observer), baseline);
        assert_health(&observer);
    }
    let mut ids = FixedIds::new([0x51; 16], [0x61; 16]);
    let result = create_exit(
        account_id,
        1,
        Context::Factory,
        &mut ids,
        &FixedClock,
        &mut store,
    );
    assert!(result.is_ok());
    drop(store);
    let connection = open_database(&path).unwrap();
    assert_health(&connection);
    let (accounts, movements, audit, ledger) = snapshot(&connection);
    assert_eq!(accounts, baseline.0);
    assert_eq!(ledger, baseline.3);
    assert_eq!(
        movements,
        vec![vec![
            Value::Blob(vec![0x51; 16]),
            Value::Text("exit".into()),
            Value::Integer(1),
            Value::Text("factory".into()),
            Value::Blob(account_id.to_vec()),
            Value::Null,
            Value::Null,
        ]]
    );
    assert_eq!(audit.len(), baseline.2.len() + 1);
    assert_eq!(
        audit[0],
        vec![
            Value::Blob(vec![0x61; 16]),
            Value::Integer(1234),
            Value::Text("movement_exit_create".into()),
            Value::Text("movement".into()),
            Value::Blob(vec![0x51; 16]),
        ]
    );
    assert_eq!(audit[1..], baseline.2);
}
