use super::entry_persistence::{snapshot, FixedClock, FixedIds, Snapshot};
use super::TempDirectory;
use millani_artes::application::create_product::create_product;
use millani_artes::application::edit_product::{
    edit_product, EditProductError, ProductEditingStore,
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
    for (id, event, name) in [(0x11, 0x21, "Primeiro"), (0x12, 0x22, "Segundo")] {
        create_product(
            name.into(),
            "Original".into(),
            7,
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
        Value::Text("product_edit".into()),
        Value::Text("product".into()),
        Value::Blob(vec![id; 16]),
    ]
}
fn opaque(e: &impl Error) {
    assert!(e.source().is_none());
    assert!(!e.to_string().contains("PRIVATE"));
}

#[test]
fn prd006_edit_preserves_identity_state_and_other_product_with_exact_fields_and_audits_after_reopen(
) {
    let (_d, p) = setup();
    let baseline = state(&open_database(&p).unwrap());
    let mut store = AdapterSqliteProductStore::new(open_database(&p).unwrap());
    let name = "  Café '🧵'\0; DROP TABLE products; --  ";
    let description = "  Descrição\r\n‘livre’\0  ";
    let mut expected = baseline.1.clone();
    let mut events = vec![];
    for (id, index, active) in [(0x11_u8, 0, true), (0x12, 1, false)] {
        for (event, price, desc) in [
            (1_u8, 0, ""),
            (2, 1, description),
            (3, i64::MAX, description),
        ] {
            let event = event + if active { 0 } else { 3 };
            let mut ids = FixedIds::new([event; 16], [0xf1; 16]);
            let value = edit_product(
                [id; 16],
                name.into(),
                desc.into(),
                price,
                &mut ids,
                &FixedClock,
                &mut store,
            )
            .unwrap();
            assert_eq!(value.id, [id; 16]);
            assert_eq!(value.name, name);
            assert_eq!(value.description, desc);
            assert_eq!(value.suggested_price_cents, price);
            assert_eq!(value.is_active, active);
            assert_eq!(ids.calls, 1);
            expected[index] = vec![
                Value::Blob(vec![id; 16]),
                Value::Text(name.into()),
                Value::Text(desc.into()),
                Value::Integer(price),
                Value::Integer(i64::from(active)),
            ];
            let observed = state(&open_database(&p).unwrap());
            assert_eq!(observed.1, expected);
            events.push(audit(id, event));
        }
    }
    drop(store);
    let c = open_database(&p).unwrap();
    health(&c);
    let after = state(&c);
    assert_eq!(after.1, expected);
    assert_eq!(after.0 .0, baseline.0 .0);
    assert_eq!(after.0 .1, baseline.0 .1);
    assert_eq!(after.0 .3, baseline.0 .3);
    events.extend(baseline.0 .2);
    assert_eq!(after.0 .2, events);
}

#[test]
fn prd006_invalid_input_and_not_found_leave_state_unchanged_and_adapter_usable() {
    let (_d, p) = setup();
    let before = state(&open_database(&p).unwrap());
    let mut store = AdapterSqliteProductStore::new(open_database(&p).unwrap());
    for (name, price) in [
        ("", 0),
        (" \t\r\n", 0),
        ("\u{2003}\u{00a0}", 0),
        ("Válido", -1),
        ("Válido", i64::MIN),
    ] {
        let e = store
            .edit_product_with_audit([0x11; 16], name, "", price, [1; 16], 1234)
            .err()
            .unwrap();
        assert_eq!(
            format!("{e:?}/{e}"),
            "ProductStoreError/product could not be stored"
        );
        opaque(&e);
        assert_eq!(state(&open_database(&p).unwrap()), before);
    }
    let e = edit_product(
        [0xff; 16],
        "Missing".into(),
        "".into(),
        0,
        &mut FixedIds::new([1; 16], [2; 16]),
        &FixedClock,
        &mut store,
    )
    .err()
    .unwrap();
    assert_eq!(e, EditProductError::NotFound);
    assert_eq!(state(&open_database(&p).unwrap()), before);
    edit_product(
        [0x11; 16],
        "Após ausência".into(),
        "".into(),
        0,
        &mut FixedIds::new([1; 16], [2; 16]),
        &FixedClock,
        &mut store,
    )
    .unwrap();
    drop(store);
    let c = open_database(&p).unwrap();
    health(&c);
    let after = state(&c);
    assert_eq!(after.1[1], before.1[1]);
    assert_eq!(after.0 .2.len(), before.0 .2.len() + 1);
}

#[test]
fn prd006_transaction_faults_rollback_poison_adapter_and_allow_fresh_recovery() {
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
            "audit_abort"=>c.execute_batch("CREATE TRIGGER prd006_fault BEFORE INSERT ON main.audit_event BEGIN SELECT RAISE(ABORT,'PRIVATE'); END;").unwrap(),
            "audit_ignore"=>c.execute_batch("CREATE TRIGGER prd006_fault BEFORE INSERT ON main.audit_event BEGIN SELECT RAISE(IGNORE); END;").unwrap(),
            "update_ignore"=>c.execute_batch("CREATE TRIGGER prd006_fault BEFORE UPDATE ON main.products BEGIN SELECT RAISE(IGNORE); END;").unwrap(),
            "update_abort"=>c.execute_batch("CREATE TRIGGER prd006_fault BEFORE UPDATE ON main.products BEGIN SELECT RAISE(ABORT,'PRIVATE'); END;").unwrap(),_=>()
        }
        let before = state(&c);
        let mut store = AdapterSqliteProductStore::new(c);
        let event = if fault == "duplicate_audit" { 0x21 } else { 1 };
        let e = edit_product(
            [0x11; 16],
            "Rejected".into(),
            "PRIVATE_DESCRIPTION".into(),
            1,
            &mut FixedIds::new([event; 16], [2; 16]),
            &FixedClock,
            &mut store,
        )
        .err()
        .unwrap();
        assert_eq!(e, EditProductError::StoreFailed);
        opaque(&e);
        let c = open_database(&p).unwrap();
        health(&c);
        assert_eq!(state(&c), before);
        c.execute_batch("DROP TRIGGER IF EXISTS main.prd006_fault;")
            .unwrap();
        drop(c);
        let e = edit_product(
            [0x11; 16],
            "Valid retry".into(),
            "".into(),
            0,
            &mut FixedIds::new([3; 16], [4; 16]),
            &FixedClock,
            &mut store,
        )
        .err()
        .expect("poisoned adapter reused");
        assert_eq!(e, EditProductError::StoreFailed);
        assert_eq!(state(&open_database(&p).unwrap()), before);
        drop(store);
        let mut fresh = AdapterSqliteProductStore::new(open_database(&p).unwrap());
        edit_product(
            [0x11; 16],
            "Recovered".into(),
            "".into(),
            0,
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
        assert_eq!(after.1[0][1], Value::Text("Recovered".into()));
        assert_eq!(after.0 .2.len(), before.0 .2.len() + 1);
    }
}

#[test]
fn prd006_invalid_audit_metadata_and_query_only_rollback_without_leaking_causes() {
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
            .edit_product_with_audit(
                [0x11; 16],
                "PRIVATE_NAME",
                "PRIVATE_DESCRIPTION",
                0,
                event,
                at,
            )
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
fn prd006_select_error_is_opaque_and_preserves_data_then_fresh_adapter_recovers() {
    let (_d, p) = setup();
    let c = open_database(&p).unwrap();
    let before = state(&c);
    let old_schema=rows(&c,"SELECT type,name,tbl_name,sql FROM main.sqlite_schema WHERE name <> 'products' ORDER BY type,name");
    let columns = rows(&c, "PRAGMA main.table_xinfo('products')");
    c.execute_batch("ALTER TABLE main.products RENAME TO prd006_products_backup;")
        .unwrap();
    let mut store = AdapterSqliteProductStore::new(c);
    let e = store
        .edit_product_with_audit([0x11; 16], "PRIVATE_NAME", "", 0, [1; 16], 1234)
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
        rows(&c, "SELECT * FROM main.prd006_products_backup ORDER BY id"),
        before.1
    );
    c.execute_batch("ALTER TABLE main.prd006_products_backup RENAME TO products;")
        .unwrap();
    assert_eq!(rows(&c,"SELECT type,name,tbl_name,sql FROM main.sqlite_schema WHERE name <> 'products' ORDER BY type,name"),old_schema);
    assert_eq!(rows(&c, "PRAGMA main.table_xinfo('products')"), columns);
    assert_eq!(state(&c), before);
    let mut fresh = AdapterSqliteProductStore::new(c);
    edit_product(
        [0x11; 16],
        "Recovered".into(),
        "".into(),
        0,
        &mut FixedIds::new([1; 16], [2; 16]),
        &FixedClock,
        &mut fresh,
    )
    .unwrap();
    drop(fresh);
    health(&open_database(&p).unwrap());
}
