use super::entry_persistence::{snapshot, FixedClock, FixedIds, Snapshot};
use super::TempDirectory;
use millani_artes::application::create_product::create_product;
use millani_artes::application::deactivate_product::{
    deactivate_product, DeactivateProductError, ProductDeactivationStore,
};
use millani_artes::database::{
    execute_versioned_migration, open_database, MOVEMENT_CATEGORIES_MIGRATION_SQL,
    PRODUCTS_MIGRATION_SQL, RECURRENCES_MIGRATION_SQL,
};
use millani_artes::infrastructure::products::AdapterSqliteProductStore;
use rusqlite::{types::Value, Connection};
use std::error::Error;
use std::path::PathBuf;

fn setup() -> (TempDirectory, PathBuf) {
    let (d, p, _) = super::entry_persistence::setup();
    let mut c = open_database(&p).unwrap();
    for (v, s) in [
        (6, MOVEMENT_CATEGORIES_MIGRATION_SQL),
        (7, RECURRENCES_MIGRATION_SQL),
        (8, PRODUCTS_MIGRATION_SQL),
    ] {
        c = execute_versioned_migration(c, v, s).unwrap();
    }
    drop(c);
    let mut store = AdapterSqliteProductStore::new(open_database(&p).unwrap());
    for (id, event, name, description, price) in [
        (
            0x11,
            0x21,
            "  Primeiro\0 🧵  ",
            "  Descrição\0 livre\r\n  ",
            0,
        ),
        (0x12, 0x22, "Segundo", "", i64::MAX),
    ] {
        create_product(
            name.into(),
            description.into(),
            price,
            &mut FixedIds::new([id; 16], [event; 16]),
            &FixedClock,
            &mut store,
        )
        .unwrap();
    }
    drop(store);
    open_database(&p)
        .unwrap()
        .execute(
            "UPDATE main.products SET is_active=0 WHERE id=?1",
            rusqlite::params![[0x12_u8; 16].as_slice()],
        )
        .unwrap();
    (d, p)
}
fn rows(c: &Connection, sql: &str) -> Vec<Vec<Value>> {
    let mut s = c.prepare(sql).unwrap();
    let n = s.column_count();
    s.query_map([], |r| {
        (0..n)
            .map(|i| r.get(i))
            .collect::<rusqlite::Result<Vec<Value>>>()
    })
    .unwrap()
    .collect::<rusqlite::Result<_>>()
    .unwrap()
}
fn state(c: &Connection) -> (Snapshot, Vec<Vec<Value>>) {
    (snapshot(c),rows(c,"SELECT id,name,description,suggested_price_cents,is_active FROM main.products ORDER BY id"))
}
fn health(c: &Connection) {
    assert!(c.is_autocommit());
    assert_eq!(snapshot(c).3, [1, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!(
        rows(c, "PRAGMA main.integrity_check"),
        vec![vec![Value::Text("ok".into())]]
    );
    assert!(rows(c, "PRAGMA main.foreign_key_check").is_empty());
}
fn audit(id: u8, event: u8) -> Vec<Value> {
    vec![
        Value::Blob(vec![event; 16]),
        Value::Integer(1234),
        Value::Text("product_deactivate".into()),
        Value::Text("product".into()),
        Value::Blob(vec![id; 16]),
    ]
}
fn opaque(e: &impl Error) {
    assert!(e.source().is_none());
    assert!(!e.to_string().contains("PRIVATE"));
}

#[test]
fn prd007_active_inactive_and_repeated_deactivation_preserve_metadata_and_exact_audits() {
    let (_d, p) = setup();
    let before = state(&open_database(&p).unwrap());
    let mut store = AdapterSqliteProductStore::new(open_database(&p).unwrap());
    let mut expected = before.clone();
    for (id, event, index) in [(0x11, 1, 0usize), (0x11, 2, 0usize), (0x12, 3, 1usize)] {
        let mut ids = FixedIds::new([event; 16], [0xF1; 16]);
        let value = deactivate_product([id; 16], &mut ids, &FixedClock, &mut store).unwrap();
        assert_eq!(value.id, [id; 16]);
        assert!(!value.is_active);
        assert_eq!(ids.calls, 1);
        assert_eq!(Value::Text(value.name), before.1[index][1]);
        assert_eq!(Value::Text(value.description), before.1[index][2]);
        assert_eq!(
            Value::Integer(value.suggested_price_cents),
            before.1[index][3]
        );
        expected.1[index][4] = Value::Integer(0);
        expected.0 .2.push(audit(id, event));
        expected.0 .2.sort_by(|a, b| match (&a[0], &b[0]) {
            (Value::Blob(a), Value::Blob(b)) => a.cmp(b),
            _ => unreachable!(),
        });
        let observer = open_database(&p).unwrap();
        health(&observer);
        assert_eq!(state(&observer), expected);
    }
    drop(store);
    let reopened = open_database(&p).unwrap();
    health(&reopened);
    assert_eq!(state(&reopened), expected);
}

#[test]
fn prd007_missing_product_has_no_effect_and_same_adapter_remains_usable() {
    let (_d, p) = setup();
    let before = state(&open_database(&p).unwrap());
    let mut store = AdapterSqliteProductStore::new(open_database(&p).unwrap());
    let mut ids = FixedIds::new([1; 16], [2; 16]);
    let error = deactivate_product([0xFF; 16], &mut ids, &FixedClock, &mut store)
        .err()
        .unwrap();
    assert_eq!(error, DeactivateProductError::NotFound);
    assert_eq!(ids.calls, 1);
    opaque(&error);
    assert_eq!(state(&open_database(&p).unwrap()), before);
    let value = deactivate_product(
        [0x11; 16],
        &mut FixedIds::new([3; 16], [4; 16]),
        &FixedClock,
        &mut store,
    )
    .unwrap();
    assert!(!value.is_active);
    drop(store);
    let mut expected = before;
    expected.1[0][4] = Value::Integer(0);
    expected.0 .2.insert(0, audit(0x11, 3));
    let c = open_database(&p).unwrap();
    health(&c);
    assert_eq!(state(&c), expected);
}

#[test]
fn prd007_transaction_faults_rollback_poison_adapter_and_allow_fresh_recovery() {
    for fault in [
        "duplicate_audit",
        "audit_abort",
        "audit_ignore",
        "update_ignore",
        "update_abort",
    ] {
        let (_d, p) = setup();
        let c = open_database(&p).unwrap();
        match fault {
            "audit_abort"=>c.execute_batch("CREATE TRIGGER prd007_fault BEFORE INSERT ON main.audit_event BEGIN SELECT RAISE(ABORT,'PRIVATE'); END;").unwrap(),
            "audit_ignore"=>c.execute_batch("CREATE TRIGGER prd007_fault BEFORE INSERT ON main.audit_event BEGIN SELECT RAISE(IGNORE); END;").unwrap(),
            "update_ignore"=>c.execute_batch("CREATE TRIGGER prd007_fault BEFORE UPDATE ON main.products BEGIN SELECT RAISE(IGNORE); END;").unwrap(),
            "update_abort"=>c.execute_batch("CREATE TRIGGER prd007_fault BEFORE UPDATE ON main.products BEGIN SELECT RAISE(ABORT,'PRIVATE'); END;").unwrap(),_=>()
        }
        let before = state(&c);
        let mut store = AdapterSqliteProductStore::new(c);
        let event = if fault == "duplicate_audit" { 0x21 } else { 1 };
        let e = deactivate_product(
            [0x11; 16],
            &mut FixedIds::new([event; 16], [2; 16]),
            &FixedClock,
            &mut store,
        )
        .err()
        .unwrap();
        assert_eq!(e, DeactivateProductError::StoreFailed);
        opaque(&e);
        let c = open_database(&p).unwrap();
        health(&c);
        assert_eq!(state(&c), before);
        c.execute_batch("DROP TRIGGER IF EXISTS main.prd007_fault;")
            .unwrap();
        drop(c);
        let e = deactivate_product(
            [0x11; 16],
            &mut FixedIds::new([3; 16], [4; 16]),
            &FixedClock,
            &mut store,
        )
        .err()
        .expect("poisoned adapter reused");
        assert_eq!(e, DeactivateProductError::StoreFailed);
        assert_eq!(state(&open_database(&p).unwrap()), before);
        drop(store);
        let mut fresh = AdapterSqliteProductStore::new(open_database(&p).unwrap());
        deactivate_product(
            [0x11; 16],
            &mut FixedIds::new([3; 16], [4; 16]),
            &FixedClock,
            &mut fresh,
        )
        .unwrap();
        drop(fresh);
        let c = open_database(&p).unwrap();
        health(&c);
        let after = state(&c);
        assert_eq!(after.1[1], before.1[1]);
        assert_eq!(after.1[0][1..4], before.1[0][1..4]);
        assert_eq!(after.1[0][4], Value::Integer(0));
        assert_eq!(after.0 .0, before.0 .0);
        assert_eq!(after.0 .1, before.0 .1);
        assert_eq!(after.0 .3, before.0 .3);
        assert_eq!(after.0 .2.len(), before.0 .2.len() + 1);
    }
}

#[test]
fn prd007_invalid_audit_metadata_and_query_only_rollback_without_leaking_causes() {
    for fault in ["zero_event", "negative_time", "query_only"] {
        let (_d, p) = setup();
        let c = open_database(&p).unwrap();
        let before = state(&c);
        if fault == "query_only" {
            c.pragma_update(None, "query_only", true).unwrap();
        }
        let mut store = AdapterSqliteProductStore::new(c);
        let event = if fault == "zero_event" {
            [0; 16]
        } else {
            [1; 16]
        };
        let at = if fault == "negative_time" { -1 } else { 1234 };
        let e = store
            .deactivate_product_with_audit([0x11; 16], event, at)
            .err()
            .unwrap();
        opaque(&e);
        assert_eq!(
            format!("{e:?}/{e}"),
            "ProductStoreError/product could not be stored"
        );
        drop(store);
        let c = open_database(&p).unwrap();
        health(&c);
        assert_eq!(state(&c), before);
    }
}

#[test]
fn prd007_select_error_is_opaque_and_preserves_data_then_fresh_adapter_recovers() {
    let (_d, p) = setup();
    let c = open_database(&p).unwrap();
    let before = state(&c);
    let old_schema=rows(&c,"SELECT type,name,tbl_name,sql FROM main.sqlite_schema WHERE name <> 'products' ORDER BY type,name");
    let columns = rows(&c, "PRAGMA main.table_xinfo('products')");
    c.execute_batch("ALTER TABLE main.products RENAME TO prd007_products_backup;")
        .unwrap();
    let mut store = AdapterSqliteProductStore::new(c);
    let e = store
        .deactivate_product_with_audit([0x11; 16], [1; 16], 1234)
        .err()
        .unwrap();
    opaque(&e);
    assert_eq!(
        format!("{e:?}/{e}"),
        "ProductStoreError/product could not be stored"
    );
    drop(store);
    let c = open_database(&p).unwrap();
    assert_eq!(snapshot(&c), before.0);
    assert_eq!(
        rows(&c, "SELECT * FROM main.prd007_products_backup ORDER BY id"),
        before.1
    );
    c.execute_batch("ALTER TABLE main.prd007_products_backup RENAME TO products;")
        .unwrap();
    assert_eq!(rows(&c,"SELECT type,name,tbl_name,sql FROM main.sqlite_schema WHERE name <> 'products' ORDER BY type,name"),old_schema);
    assert_eq!(rows(&c, "PRAGMA main.table_xinfo('products')"), columns);
    assert_eq!(state(&c), before);
    let mut fresh = AdapterSqliteProductStore::new(c);
    deactivate_product(
        [0x11; 16],
        &mut FixedIds::new([1; 16], [2; 16]),
        &FixedClock,
        &mut fresh,
    )
    .unwrap();
    drop(fresh);
    health(&open_database(&p).unwrap());
}
