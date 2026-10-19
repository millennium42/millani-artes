use super::entry_persistence::{snapshot, FixedClock, FixedIds, Snapshot};
use super::TempDirectory;
use millani_artes::application::create_product::{
    create_product, CreateProductError, ProductCreationStore,
};
use millani_artes::database::{
    execute_versioned_migration, open_database, MOVEMENT_CATEGORIES_MIGRATION_SQL,
    PRODUCTS_MIGRATION_SQL, RECURRENCES_MIGRATION_SQL,
};
use millani_artes::domain::Product;
use millani_artes::infrastructure::products::AdapterSqliteProductStore;
use rusqlite::{types::Value, Connection};
use std::error::Error;
use std::path::PathBuf;

fn setup() -> (TempDirectory, PathBuf) {
    let (d, p, _) = super::entry_persistence::setup();
    let mut c = open_database(&p).unwrap();
    for (v, sql) in [
        (6, MOVEMENT_CATEGORIES_MIGRATION_SQL),
        (7, RECURRENCES_MIGRATION_SQL),
        (8, PRODUCTS_MIGRATION_SQL),
    ] {
        c = execute_versioned_migration(c, v, sql).unwrap();
    }
    drop(c);
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
fn product(id: u8, name: &str, price: i64, active: bool) -> Product {
    Product {
        id: [id; 16],
        name: name.into(),
        description: "  Descrição‘livre’\0  ".into(),
        suggested_price_cents: price,
        is_active: active,
    }
}
fn audit(id: u8, event: u8) -> Vec<Value> {
    vec![
        Value::Blob(vec![event; 16]),
        Value::Integer(1234),
        Value::Text("product_create".into()),
        Value::Text("product".into()),
        Value::Blob(vec![id; 16]),
    ]
}
fn opaque(e: &impl Error) {
    assert!(e.source().is_none());
    assert!(!e.to_string().contains("PRIVATE"));
}

#[test]
fn prd005_creation_reuses_adapter_preserves_fields_bounds_and_one_audit_each_after_reopen() {
    let (_d, p) = setup();
    let baseline = state(&open_database(&p).unwrap());
    let mut store = AdapterSqliteProductStore::new(open_database(&p).unwrap());
    let name = "  Café '🧵'\0; DROP TABLE products; --  ";
    let desc = "  Descrição\r\n‘livre’\0  ";
    for (id, event, price, description) in [
        (0x11, 0x21, 0, ""),
        (0x12, 0x22, 1, desc),
        (0x13, 0x23, i64::MAX, desc),
    ] {
        let mut ids = FixedIds::new([id; 16], [event; 16]);
        let value = create_product(
            name.into(),
            description.into(),
            price,
            &mut ids,
            &FixedClock,
            &mut store,
        )
        .unwrap();
        assert_eq!(value.id, [id; 16]);
        assert_eq!(value.name, name);
        assert_eq!(value.description, description);
        assert_eq!(value.suggested_price_cents, price);
        assert!(value.is_active);
        assert_eq!(ids.calls, 2);
    }
    drop(store);
    let c = open_database(&p).unwrap();
    health(&c);
    let (after, products) = state(&c);
    assert_eq!(after.0, baseline.0 .0);
    assert_eq!(after.1, baseline.0 .1);
    assert_eq!(after.3, baseline.0 .3);
    assert_eq!(
        products,
        [(0x11, 0, ""), (0x12, 1, desc), (0x13, i64::MAX, desc)].map(
            |(id, price, description)| vec![
                Value::Blob(vec![id; 16]),
                Value::Text(name.into()),
                Value::Text(description.into()),
                Value::Integer(price),
                Value::Integer(1)
            ]
        )
    );
    let mut expected = vec![audit(0x11, 0x21), audit(0x12, 0x22), audit(0x13, 0x23)];
    expected.extend(baseline.0 .2);
    assert_eq!(after.2, expected);
}

#[test]
fn prd005_invalid_direct_products_leave_adapter_usable_and_database_unchanged() {
    let (_d, p) = setup();
    let before = state(&open_database(&p).unwrap());
    let mut store = AdapterSqliteProductStore::new(open_database(&p).unwrap());
    for (name, price, active) in [
        ("", 0, true),
        (" \t\r\n", 0, true),
        ("\u{2003}\u{00a0}", 0, true),
        ("Válido", -1, true),
        ("Válido", i64::MIN, true),
        ("Inativo", 0, false),
    ] {
        let error = store
            .persist_product_with_audit(&product(0x55, name, price, active), [0x66; 16], 1234)
            .unwrap_err();
        assert_eq!(format!("{error:?}"), "ProductStoreError");
        assert_eq!(error.to_string(), "product could not be stored");
        opaque(&error);
        let c = open_database(&p).unwrap();
        health(&c);
        assert_eq!(state(&c), before);
    }
    store
        .persist_product_with_audit(&product(0x55, "Válido", 0, true), [0x66; 16], 1234)
        .unwrap();
    drop(store);
    let c = open_database(&p).unwrap();
    health(&c);
    let after = state(&c);
    assert_eq!(after.1.len(), 1);
    assert_eq!(after.1[0][3], Value::Integer(0));
    assert_eq!(after.0 .0, before.0 .0);
    assert_eq!(after.0 .1, before.0 .1);
    assert_eq!(after.0 .3, before.0 .3);
    assert_eq!(after.0 .2.len(), before.0 .2.len() + 1);
}

#[test]
fn prd005_database_faults_rollback_poison_adapter_and_allow_fresh_recovery() {
    for fault in [
        "product_duplicate",
        "audit_duplicate",
        "audit_abort",
        "product_ignore",
        "audit_ignore",
    ] {
        let (_d, p) = setup();
        let mut store = AdapterSqliteProductStore::new(open_database(&p).unwrap());
        create_product(
            "Seed".into(),
            "".into(),
            0,
            &mut FixedIds::new([0x11; 16], [0x21; 16]),
            &FixedClock,
            &mut store,
        )
        .unwrap();
        drop(store);
        let c = open_database(&p).unwrap();
        match fault {
            "audit_abort"=>c.execute_batch("CREATE TRIGGER prd005_fault BEFORE INSERT ON main.audit_event BEGIN SELECT RAISE(ABORT,'PRIVATE_SYNTHETIC'); END;").unwrap(),
            "product_ignore"=>c.execute_batch("CREATE TRIGGER prd005_fault BEFORE INSERT ON main.products BEGIN SELECT RAISE(IGNORE); END;").unwrap(),
            "audit_ignore"=>c.execute_batch("CREATE TRIGGER prd005_fault BEFORE INSERT ON main.audit_event BEGIN SELECT RAISE(IGNORE); END;").unwrap(),_=>()
        }
        let before = state(&c);
        drop(c);
        let mut store = AdapterSqliteProductStore::new(open_database(&p).unwrap());
        let id = if fault == "product_duplicate" {
            0x11
        } else {
            0x12
        };
        let event = if fault == "audit_duplicate" {
            0x21
        } else {
            0x22
        };
        let error = create_product(
            "Rejected".into(),
            "PRIVATE_DESCRIPTION".into(),
            1,
            &mut FixedIds::new([id; 16], [event; 16]),
            &FixedClock,
            &mut store,
        )
        .err()
        .unwrap();
        assert_eq!(error, CreateProductError::StoreFailed);
        opaque(&error);
        let c = open_database(&p).unwrap();
        health(&c);
        assert_eq!(state(&c), before);
        c.execute_batch("DROP TRIGGER IF EXISTS main.prd005_fault;")
            .unwrap();
        drop(c);
        let error = create_product(
            "Valid retry".into(),
            "".into(),
            1,
            &mut FixedIds::new([0x33; 16], [0x44; 16]),
            &FixedClock,
            &mut store,
        )
        .err()
        .expect("poisoned adapter reused");
        assert_eq!(error, CreateProductError::StoreFailed);
        assert_eq!(state(&open_database(&p).unwrap()), before);
        drop(store);
        let mut fresh = AdapterSqliteProductStore::new(open_database(&p).unwrap());
        create_product(
            "Recovered".into(),
            "".into(),
            0,
            &mut FixedIds::new([0x33; 16], [0x44; 16]),
            &FixedClock,
            &mut fresh,
        )
        .unwrap();
        drop(fresh);
        let c = open_database(&p).unwrap();
        health(&c);
        let after = state(&c);
        assert_eq!(after.1.len(), before.1.len() + 1);
        assert_eq!(after.0 .2.len(), before.0 .2.len() + 1);
        assert_eq!(after.0 .0, before.0 .0);
        assert_eq!(after.0 .1, before.0 .1);
        assert_eq!(after.0 .3, before.0 .3);
    }
}

#[test]
fn prd005_invalid_audit_metadata_and_readonly_connection_rollback_without_leaking_causes() {
    for fault in ["zero_product", "zero_event", "negative_time", "query_only"] {
        let (_d, p) = setup();
        let c = open_database(&p).unwrap();
        let before = state(&c);
        if fault == "query_only" {
            c.pragma_update(None, "query_only", true).unwrap();
        }
        let mut store = AdapterSqliteProductStore::new(c);
        let value = product(
            if fault == "zero_product" { 0 } else { 0x55 },
            "PRIVATE_NAME",
            0,
            true,
        );
        let event = if fault == "zero_event" {
            [0; 16]
        } else {
            [0x66; 16]
        };
        let at = if fault == "negative_time" { -1 } else { 1234 };
        let e = store
            .persist_product_with_audit(&value, event, at)
            .unwrap_err();
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
