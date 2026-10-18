use super::entry_persistence::{assert_health, setup, snapshot, FixedClock, FixedIds, Snapshot};
use millani_artes::application::create_category::create_category;
use millani_artes::application::deactivate_category::{
    deactivate_category, DeactivateCategoryError,
};
use millani_artes::database::open_database;
use millani_artes::infrastructure::categories::AdapterSqliteCategoryStore;
use rusqlite::{types::Value, Connection};
use std::path::Path;

type CategoryRow = (Vec<u8>, String, i64);
const NAME: &str = "  Café '🧵'\0  ";

fn state(connection: &Connection) -> (Snapshot, Vec<CategoryRow>) {
    let mut statement = connection
        .prepare("SELECT id,name,is_active FROM main.categories ORDER BY id")
        .unwrap();
    let categories = statement
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .unwrap()
        .collect::<rusqlite::Result<Vec<CategoryRow>>>()
        .unwrap();
    (snapshot(connection), categories)
}

fn audit(category: u8, event: u8) -> Vec<Value> {
    vec![
        Value::Blob(vec![event; 16]),
        Value::Integer(1234),
        Value::Text("category_deactivate".into()),
        Value::Text("category".into()),
        Value::Blob(vec![category; 16]),
    ]
}

fn setup_categories(path: &Path) {
    let mut store = AdapterSqliteCategoryStore::new(open_database(path).unwrap());
    for (id, event, name) in [(0x11, 0x21, NAME), (0x12, 0x22, NAME)] {
        create_category(
            name.into(),
            &mut FixedIds::new([id; 16], [event; 16]),
            &FixedClock,
            &mut store,
        )
        .unwrap();
    }
    drop(store);
    let connection = open_database(path).unwrap();
    assert_eq!(
        connection
            .execute(
                "UPDATE main.categories SET is_active=0 WHERE id=?1",
                rusqlite::params![[0x12u8; 16].as_slice()]
            )
            .unwrap(),
        1
    );
}

#[test]
fn deactivate_active_and_inactive_categories_same_store() {
    let (_directory, path, _) = setup();
    setup_categories(&path);
    let baseline = state(&open_database(&path).unwrap());
    let mut store = AdapterSqliteCategoryStore::new(open_database(&path).unwrap());
    let name = NAME;
    for (id, event) in [(0x11, 1), (0x12, 2)] {
        let mut ids = FixedIds::new([event; 16], [0xF1; 16]);
        let category = deactivate_category([id; 16], &mut ids, &FixedClock, &mut store).unwrap();
        assert_eq!(category.id, [id; 16]);
        assert_eq!(category.name, name);
        assert!(!category.is_active);
        assert_eq!(ids.calls, 1);
    }
    drop(store);
    let connection = open_database(&path).unwrap();
    assert_health(&connection);
    let ((accounts, movements, events, ledger), categories) = state(&connection);
    assert_eq!(accounts, baseline.0 .0);
    assert_eq!(movements, baseline.0 .1);
    assert_eq!(ledger, baseline.0 .3);
    assert_eq!(
        categories,
        vec![
            (vec![0x11; 16], name.into(), 0),
            (vec![0x12; 16], name.into(), 0)
        ]
    );
    let mut expected = vec![audit(0x11, 1), audit(0x12, 2)];
    expected.extend(baseline.0 .2);
    assert_eq!(events, expected);
}

#[test]
fn missing_category_not_found_and_subsequent_deactivation_works() {
    let (_directory, path, _) = setup();
    setup_categories(&path);
    let baseline = state(&open_database(&path).unwrap());
    let mut store = AdapterSqliteCategoryStore::new(open_database(&path).unwrap());
    let mut ids = FixedIds::new([1; 16], [0xF1; 16]);
    let error = deactivate_category([0xFF; 16], &mut ids, &FixedClock, &mut store)
        .err()
        .expect("missing category accepted");
    assert_eq!(error, DeactivateCategoryError::NotFound);
    assert_eq!(ids.calls, 1);
    assert_eq!(state(&open_database(&path).unwrap()), baseline);
    let category = deactivate_category(
        [0x11; 16],
        &mut FixedIds::new([3; 16], [0xF1; 16]),
        &FixedClock,
        &mut store,
    )
    .unwrap();
    assert_eq!(category.id, [0x11; 16]);
    assert!(!category.is_active);
    drop(store);
    let connection = open_database(&path).unwrap();
    assert_health(&connection);
    let ((accounts, movements, events, ledger), categories) = state(&connection);
    assert_eq!(accounts, baseline.0 .0);
    assert_eq!(movements, baseline.0 .1);
    assert_eq!(ledger, baseline.0 .3);
    assert_eq!(
        categories,
        vec![
            (vec![0x11; 16], NAME.into(), 0),
            (vec![0x12; 16], NAME.into(), 0)
        ]
    );
    let mut expected = vec![audit(0x11, 3)];
    expected.extend(baseline.0 .2);
    assert_eq!(events, expected);
}

#[test]
fn database_faults_rollback_and_poison_adapter() {
    for fault in ["duplicate_audit", "audit_abort", "update_ignore"] {
        let (_directory, path, _) = setup();
        setup_categories(&path);
        let connection = open_database(&path).unwrap();
        match fault {
            "duplicate_audit" => (),
            "audit_abort" => connection.execute_batch("CREATE TRIGGER deactivate_audit_abort BEFORE INSERT ON main.audit_event BEGIN SELECT RAISE(ABORT,'synthetic'); END;").unwrap(),
            "update_ignore" => connection.execute_batch("CREATE TRIGGER deactivate_update_ignore BEFORE UPDATE ON main.categories BEGIN SELECT RAISE(IGNORE); END;").unwrap(),
            _ => unreachable!(),
        }
        let baseline = state(&connection);
        let mut store = AdapterSqliteCategoryStore::new(connection);
        let event = if fault == "duplicate_audit" { 0xB2 } else { 1 };
        let error = deactivate_category(
            [0x11; 16],
            &mut FixedIds::new([event; 16], [0xF1; 16]),
            &FixedClock,
            &mut store,
        )
        .err()
        .expect("fault did not fail");
        assert_eq!(error, DeactivateCategoryError::StoreFailed);
        let error = deactivate_category(
            [0x11; 16],
            &mut FixedIds::new([2; 16], [0xF1; 16]),
            &FixedClock,
            &mut store,
        )
        .err()
        .expect("poisoned store reused");
        assert_eq!(error, DeactivateCategoryError::StoreFailed);
        drop(store);
        let connection = open_database(&path).unwrap();
        assert_health(&connection);
        assert_eq!(
            state(&connection),
            baseline,
            "fault {fault} changed stored state"
        );
    }
}
