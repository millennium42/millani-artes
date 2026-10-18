use super::TempDirectory;
use millani_artes::application::create_account::create_account;
use millani_artes::application::create_entry::{create_entry, EntryCreationStore};
use millani_artes::database::{
    execute_versioned_migration, open_database, ACCOUNTS_MIGRATION_SQL, ACCOUNTS_MIGRATION_VERSION,
    AUDIT_EVENT_MIGRATION_SQL, AUDIT_EVENT_MIGRATION_VERSION, CATEGORIES_MIGRATION_SQL,
    CATEGORIES_MIGRATION_VERSION, MOVEMENTS_MIGRATION_SQL, MOVEMENTS_MIGRATION_VERSION,
};
use millani_artes::domain::ports::{Clock, UuidGenerator};
use millani_artes::domain::{AccountType, Context, Movement, MovementKind};
use millani_artes::infrastructure::accounts::AdapterSqliteAccountStore;
use millani_artes::infrastructure::movements::AdapterSqliteMovementStore;
use rusqlite::{types::Value, Connection};
use std::error::Error;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub(super) struct FixedIds {
    values: [[u8; 16]; 2],
    pub(super) calls: usize,
}

impl FixedIds {
    pub(super) fn new(movement: [u8; 16], audit: [u8; 16]) -> Self {
        Self {
            values: [movement, audit],
            calls: 0,
        }
    }
}

impl UuidGenerator for FixedIds {
    type Error = ();

    fn generate(&mut self) -> Result<[u8; 16], Self::Error> {
        let id = *self.values.get(self.calls).ok_or(())?;
        self.calls += 1;
        Ok(id)
    }
}

pub(super) struct FixedClock;

impl Clock for FixedClock {
    fn now(&self) -> SystemTime {
        UNIX_EPOCH + Duration::from_millis(1234)
    }
}

pub(super) fn setup() -> (TempDirectory, PathBuf, [u8; 16]) {
    let directory = TempDirectory::new();
    let path = directory.path().join("fin017.sqlite");
    let connection = open_database(&path).expect("open");
    let mut connection = execute_versioned_migration(connection, 1, "").expect("v1");
    for (version, sql) in [
        (AUDIT_EVENT_MIGRATION_VERSION, AUDIT_EVENT_MIGRATION_SQL),
        (ACCOUNTS_MIGRATION_VERSION, ACCOUNTS_MIGRATION_SQL),
        (CATEGORIES_MIGRATION_VERSION, CATEGORIES_MIGRATION_SQL),
        (MOVEMENTS_MIGRATION_VERSION, MOVEMENTS_MIGRATION_SQL),
    ] {
        connection = execute_versioned_migration(connection, version, sql).expect("migrate");
    }
    let account_id = [0xA1; 16];
    let mut account_store = AdapterSqliteAccountStore::new(connection);
    let mut ids = FixedIds::new(account_id, [0xB2; 16]);
    let account = create_account(
        "Conta sintética".to_string(),
        AccountType::BankPix,
        -1234,
        &mut ids,
        &FixedClock,
        &mut account_store,
    )
    .expect("seed account through use case");
    assert_eq!(account.id, account_id);
    drop(account_store);
    (directory, path, account_id)
}

pub(super) type Snapshot = (Vec<Vec<Value>>, Vec<Vec<Value>>, Vec<Vec<Value>>, Vec<i64>);

fn rows(connection: &Connection, sql: &str, columns: usize) -> Vec<Vec<Value>> {
    let mut statement = connection.prepare(sql).expect("snapshot query");
    statement
        .query_map([], |row| {
            (0..columns)
                .map(|index| row.get(index))
                .collect::<rusqlite::Result<Vec<Value>>>()
        })
        .expect("snapshot rows")
        .collect::<rusqlite::Result<Vec<_>>>()
        .expect("snapshot values")
}

pub(super) fn snapshot(connection: &Connection) -> Snapshot {
    let accounts = rows(
        connection,
        "SELECT id,name,account_type,initial_balance_cents FROM main.accounts ORDER BY id",
        4,
    );
    let movements = rows(connection, "SELECT id,kind,amount_cents,context,account_id,source_account_id,destination_account_id FROM main.movements ORDER BY id", 7);
    let audit = rows(connection, "SELECT event_id,occurred_at_ms,operation,entity,entity_id FROM main.audit_event ORDER BY event_id", 5);
    let mut statement = connection
        .prepare("SELECT version FROM main.schema_migration ORDER BY version")
        .unwrap();
    let ledger = statement
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<rusqlite::Result<Vec<i64>>>()
        .unwrap();
    (accounts, movements, audit, ledger)
}

pub(super) fn assert_health(connection: &Connection) {
    assert!(connection.is_autocommit());
    assert_eq!(snapshot(connection).3, [1, 2, 3, 4, 5]);
    assert_eq!(
        connection
            .query_row("PRAGMA main.integrity_check", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    let mut statement = connection.prepare("PRAGMA main.foreign_key_check").unwrap();
    assert!(statement.query([]).unwrap().next().unwrap().is_none());
}

#[test]
fn entry_use_case_persists_both_contexts_and_audit_in_one_database() {
    let (_directory, path, account_id) = setup();
    let mut store = AdapterSqliteMovementStore::new(open_database(&path).unwrap());
    for (movement_id, audit_id, amount, context) in [
        ([0x11; 16], [0x21; 16], i64::MAX, Context::Home),
        ([0x12; 16], [0x22; 16], 1, Context::Factory),
    ] {
        let mut ids = FixedIds::new(movement_id, audit_id);
        let movement = create_entry(
            account_id,
            amount,
            context,
            &mut ids,
            &FixedClock,
            &mut store,
        )
        .expect("persist entry");
        assert_eq!(movement.id, movement_id);
        assert_eq!(movement.amount_cents, amount);
        assert!(
            matches!(&movement.kind, MovementKind::Entry { account_id: id } if *id == account_id)
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
                Value::Text("entry".into()),
                Value::Integer(i64::MAX),
                Value::Text("home".into()),
                Value::Blob(account_id.to_vec()),
                Value::Null,
                Value::Null
            ],
            vec![
                Value::Blob(vec![0x12; 16]),
                Value::Text("entry".into()),
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
                Value::Text("movement_entry_create".into()),
                Value::Text("movement".into()),
                Value::Blob(vec![0x11; 16])
            ],
            vec![
                Value::Blob(vec![0x22; 16]),
                Value::Integer(1234),
                Value::Text("movement_entry_create".into()),
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
            3 => MovementKind::Exit { account_id },
            4 => MovementKind::Transfer {
                source_account_id: account_id,
                destination_account_id: account_id,
            },
            _ => MovementKind::Entry { account_id },
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
            .persist_entry_with_audit(&movement, [0x40 + case; 16], 1234)
            .expect_err("invalid movement rejected");
        assert!(!format!("{error}").is_empty());
        assert!(!format!("{error:?}").contains("PRIVATE_"));
        assert!(error.source().is_none());
        let observer = open_database(&path).unwrap();
        assert_eq!(snapshot(&observer), baseline);
        assert_health(&observer);
    }
    let mut ids = FixedIds::new([0x51; 16], [0x61; 16]);
    let result = create_entry(
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
            Value::Text("entry".into()),
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
            Value::Text("movement_entry_create".into()),
            Value::Text("movement".into()),
            Value::Blob(vec![0x51; 16]),
        ]
    );
    assert_eq!(audit[1..], baseline.2);
}
