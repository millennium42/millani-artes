use super::entry_persistence::{assert_health, setup, snapshot, FixedClock, FixedIds, Snapshot};
use millani_artes::application::create_category::{
    create_category, CategoryCreationStore, CreateCategoryError,
};
use millani_artes::database::open_database;
use millani_artes::domain::Category;
use millani_artes::infrastructure::categories::AdapterSqliteCategoryStore;
use rusqlite::{types::Value, Connection};
use std::error::Error;

type CategoryRow = (Vec<u8>, String, i64);

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
        Value::Text("category_create".into()),
        Value::Text("category".into()),
        Value::Blob(vec![category; 16]),
    ]
}

#[test]
fn category_creation_reuses_adapter_and_preserves_exact_names_after_reopen() {
    let (_directory, path, _) = setup();
    let baseline = state(&open_database(&path).unwrap());
    let mut store = AdapterSqliteCategoryStore::new(open_database(&path).unwrap());
    let name = "  Café '🧵'\0  ";
    for (id, event) in [(0x11, 0x21), (0x12, 0x22)] {
        let mut ids = FixedIds::new([id; 16], [event; 16]);
        let category = create_category(name.into(), &mut ids, &FixedClock, &mut store).unwrap();
        assert_eq!(category.id, [id; 16]);
        assert_eq!(category.name, name);
        assert!(category.is_active);
        assert_eq!(ids.calls, 2);
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
            (vec![0x11; 16], name.into(), 1),
            (vec![0x12; 16], name.into(), 1)
        ]
    );
    let mut expected = vec![audit(0x11, 0x21), audit(0x12, 0x22)];
    expected.extend(baseline.0 .2);
    assert_eq!(events, expected);
}

#[test]
fn invalid_direct_categories_have_no_effects_and_keep_adapter_usable() {
    let (_directory, path, _) = setup();
    let baseline = state(&open_database(&path).unwrap());
    let mut store = AdapterSqliteCategoryStore::new(open_database(&path).unwrap());
    for (name, is_active) in [
        ("", true),
        (" \t\r\n", true),
        ("\u{2003}\u{00A0}", true),
        ("Inativa", false),
    ] {
        let category = Category {
            id: [0x55; 16],
            name: name.into(),
            is_active,
        };
        let error = store
            .persist_category_with_audit(&category, [0x66; 16], 1234)
            .expect_err("invalid category accepted");
        assert_eq!(format!("{error:?}"), "CategoryStoreError");
        assert_eq!(error.to_string(), "category could not be stored");
        assert!(error.source().is_none());
        let connection = open_database(&path).unwrap();
        assert_health(&connection);
        assert_eq!(state(&connection), baseline);
    }
    let valid = Category {
        id: [0x55; 16],
        name: "Válida".into(),
        is_active: true,
    };
    store
        .persist_category_with_audit(&valid, [0x66; 16], 1234)
        .unwrap();
    drop(store);
    let connection = open_database(&path).unwrap();
    assert_health(&connection);
    let ((accounts, movements, events, ledger), categories) = state(&connection);
    assert_eq!(accounts, baseline.0 .0);
    assert_eq!(movements, baseline.0 .1);
    assert_eq!(ledger, baseline.0 .3);
    assert_eq!(categories, vec![(vec![0x55; 16], "Válida".into(), 1)]);
    let mut expected = vec![audit(0x55, 0x66)];
    expected.extend(baseline.0 .2);
    assert_eq!(events, expected);
}

#[test]
fn category_database_faults_rollback_all_rows_and_poison_adapter() {
    for fault in ["duplicate", "audit_abort", "category_ignore"] {
        let (_directory, path, _) = setup();
        let mut store = AdapterSqliteCategoryStore::new(open_database(&path).unwrap());
        create_category(
            "Seed".into(),
            &mut FixedIds::new([0x11; 16], [0x21; 16]),
            &FixedClock,
            &mut store,
        )
        .unwrap();
        drop(store);
        let connection = open_database(&path).unwrap();
        match fault {
            "duplicate" => (),
            "audit_abort" => connection.execute_batch("CREATE TRIGGER category_audit_abort BEFORE INSERT ON main.audit_event BEGIN SELECT RAISE(ABORT,'synthetic'); END;").unwrap(),
            "category_ignore" => connection.execute_batch("CREATE TRIGGER category_insert_ignore BEFORE INSERT ON main.categories BEGIN SELECT RAISE(IGNORE); END;").unwrap(),
            _ => unreachable!(),
        }
        let baseline = state(&connection);
        drop(connection);
        let mut store = AdapterSqliteCategoryStore::new(open_database(&path).unwrap());
        let id = if fault == "duplicate" { 0x11 } else { 0x12 };
        let error = create_category(
            "Rejected".into(),
            &mut FixedIds::new([id; 16], [0x22; 16]),
            &FixedClock,
            &mut store,
        )
        .err()
        .expect("fault did not fail");
        assert_eq!(error, CreateCategoryError::StoreFailed);
        let error = create_category(
            "Retry".into(),
            &mut FixedIds::new([0x33; 16], [0x44; 16]),
            &FixedClock,
            &mut store,
        )
        .err()
        .expect("poisoned store reused");
        assert_eq!(error, CreateCategoryError::StoreFailed);
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
