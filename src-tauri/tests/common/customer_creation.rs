use super::entry_persistence::{FixedClock, FixedIds};
use super::TempDirectory;
use millani_artes::application::create_customer::{
    create_customer, CreateCustomerError, CustomerCreationStore,
};
use millani_artes::database::{
    execute_versioned_migration, open_database, CUSTOMERS_MIGRATION_SQL,
    MOVEMENT_CATEGORIES_MIGRATION_SQL, PRODUCTS_MIGRATION_SQL, RECURRENCES_MIGRATION_SQL,
};
use millani_artes::domain::Customer;
use millani_artes::infrastructure::customers::AdapterSqliteCustomerStore;
use rusqlite::{params, types::Value, Connection};
use std::error::Error;
use std::path::PathBuf;

fn setup() -> (TempDirectory, PathBuf) {
    let (d, p, _) = super::entry_persistence::setup();
    let mut c = open_database(&p).unwrap();
    for (v, sql) in [
        (6, MOVEMENT_CATEGORIES_MIGRATION_SQL),
        (7, RECURRENCES_MIGRATION_SQL),
        (8, PRODUCTS_MIGRATION_SQL),
        (9, CUSTOMERS_MIGRATION_SQL),
    ] {
        c = execute_versioned_migration(c, v, sql).unwrap();
    }
    c.execute(
        "INSERT INTO main.products VALUES(?1,'Produto histórico',?2,0,0)",
        params![[0x80_u8; 16].as_slice(), "  descrição ‘Á’\r\n\0  "],
    )
    .unwrap();
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
type State = (
    Vec<Vec<Vec<Value>>>,
    Vec<Vec<Value>>,
    Vec<Vec<Value>>,
    Vec<Vec<Value>>,
);
fn state(c: &Connection) -> State {
    let others = [
        "accounts",
        "categories",
        "movements",
        "recurrences",
        "recurrence_obligations",
        "products",
    ]
    .iter()
    .map(|t| rows(c, &format!("SELECT * FROM main.{t} ORDER BY 1")))
    .collect();
    (
        others,
        rows(c, "SELECT * FROM main.audit_event ORDER BY event_id"),
        rows(
            c,
            "SELECT version FROM main.schema_migration ORDER BY version",
        ),
        rows(c, "SELECT id,name,phone FROM main.customers ORDER BY id"),
    )
}
fn health(c: &Connection) {
    assert!(c.is_autocommit());
    assert_eq!(
        rows(
            c,
            "SELECT version FROM main.schema_migration ORDER BY version"
        ),
        (1..=9).map(|v| vec![Value::Integer(v)]).collect::<Vec<_>>()
    );
    assert_eq!(
        rows(c, "PRAGMA main.integrity_check"),
        vec![vec![Value::Text("ok".into())]]
    );
    assert!(rows(c, "PRAGMA main.foreign_key_check").is_empty());
}
fn customer(id: u8, name: &str, phone: Option<&str>) -> Customer {
    Customer {
        id: [id; 16],
        name: name.into(),
        phone: phone.map(String::from),
    }
}
fn audit(id: u8, event: u8) -> Vec<Value> {
    vec![
        Value::Blob(vec![event; 16]),
        Value::Integer(1234),
        Value::Text("customer_create".into()),
        Value::Text("customer".into()),
        Value::Blob(vec![id; 16]),
    ]
}
fn opaque(e: &impl Error) {
    assert!(e.source().is_none());
    assert!(!e.to_string().contains("PRIVATE"));
}

#[test]
fn cus005_creation_reuses_adapter_preserves_none_some_and_one_audit_each_after_reopen() {
    let (_d, p) = setup();
    let baseline = state(&open_database(&p).unwrap());
    let mut store = AdapterSqliteCustomerStore::new(open_database(&p).unwrap());
    let name = "  Café '🧵'\0; DROP TABLE customers; --  ";
    let mut expected_customers = Vec::new();
    let mut expected_audits = baseline.1.clone();
    for (id, event, phone) in [
        (0x11, 0x21, None),
        (0x12, 0x22, Some("(11) 91234-5678")),
        (0x13, 0x23, Some("(11) 91234-5678")),
    ] {
        let mut ids = FixedIds::new([id; 16], [event; 16]);
        let value = create_customer(
            name.into(),
            phone.map(String::from),
            &mut ids,
            &FixedClock,
            &mut store,
        )
        .unwrap();
        assert_eq!(value.id, [id; 16]);
        assert_eq!(value.name, name);
        assert_eq!(value.phone.as_deref(), phone);
        assert_eq!(ids.calls, 2);
        expected_customers.push(vec![
            Value::Blob(vec![id; 16]),
            Value::Text(name.into()),
            phone.map(|v| Value::Text(v.into())).unwrap_or(Value::Null),
        ]);
        expected_audits.push(audit(id, event));
        let c = open_database(&p).unwrap();
        health(&c);
        let after = state(&c);
        assert_eq!(after.0, baseline.0);
        assert_eq!(after.2, baseline.2);
        assert_eq!(after.3, expected_customers);
        assert_eq!(after.1.len(), expected_audits.len());
        for row in &expected_audits {
            assert!(after.1.contains(row));
        }
    }
    drop(store);
    let c = open_database(&p).unwrap();
    health(&c);
    let after = state(&c);
    assert_eq!(after.0, baseline.0);
    assert_eq!(after.2, baseline.2);
    assert_eq!(after.3, expected_customers);
    assert_eq!(after.1.len(), expected_audits.len());
    for row in expected_audits {
        assert!(after.1.contains(&row));
    }
}

#[test]
fn cus005_invalid_direct_customers_leave_adapter_usable_and_database_unchanged() {
    let (_d, p) = setup();
    let before = state(&open_database(&p).unwrap());
    let mut store = AdapterSqliteCustomerStore::new(open_database(&p).unwrap());
    for (name, phone) in [
        ("", None),
        (" \t\r\n", None),
        ("\u{2003}\u{00a0}", None),
        ("Válido", Some("")),
        ("Válido", Some("(20) 91234-5678")),
        ("Válido", Some("(11) 81234-5678")),
        ("Válido", Some("(11) 9é34-5678")),
    ] {
        let e = store
            .persist_customer_with_audit(&customer(0x55, name, phone), [0x66; 16], 1234)
            .unwrap_err();
        opaque(&e);
        assert_eq!(
            format!("{e:?}/{e}"),
            "CustomerStoreError/customer could not be stored"
        );
        let c = open_database(&p).unwrap();
        health(&c);
        assert_eq!(state(&c), before);
    }
    store
        .persist_customer_with_audit(&customer(0x55, "Válido", None), [0x66; 16], 1234)
        .unwrap();
    drop(store);
    let c = open_database(&p).unwrap();
    health(&c);
    let after = state(&c);
    assert_eq!(after.0, before.0);
    assert_eq!(after.2, before.2);
    assert_eq!(after.3.len(), 1);
    assert_eq!(after.3[0][2], Value::Null);
    let mut expected = before.1.clone();
    expected.push(audit(0x55, 0x66));
    assert_eq!(after.1.len(), expected.len());
    for row in expected {
        assert!(after.1.contains(&row));
    }
}

#[test]
fn cus005_database_faults_rollback_poison_adapter_and_allow_fresh_recovery() {
    for fault in [
        "customer_duplicate",
        "audit_duplicate",
        "audit_abort",
        "audit_ignore",
        "customer_ignore",
        "customer_abort",
    ] {
        let (_d, p) = setup();
        let mut seed = AdapterSqliteCustomerStore::new(open_database(&p).unwrap());
        create_customer(
            "Seed".into(),
            None,
            &mut FixedIds::new([0x11; 16], [0x21; 16]),
            &FixedClock,
            &mut seed,
        )
        .unwrap();
        drop(seed);
        let c = open_database(&p).unwrap();
        match fault {
            "audit_abort"=>c.execute_batch("CREATE TRIGGER main.cus005_fault BEFORE INSERT ON audit_event BEGIN SELECT RAISE(ABORT,'PRIVATE_SYNTHETIC'); END;").unwrap(),
            "audit_ignore"=>c.execute_batch("CREATE TRIGGER main.cus005_fault BEFORE INSERT ON audit_event BEGIN SELECT RAISE(IGNORE); END;").unwrap(),
            "customer_ignore"=>c.execute_batch("CREATE TRIGGER main.cus005_fault BEFORE INSERT ON customers BEGIN SELECT RAISE(IGNORE); END;").unwrap(),
            "customer_abort"=>c.execute_batch("CREATE TRIGGER main.cus005_fault BEFORE INSERT ON customers BEGIN SELECT RAISE(ABORT,'PRIVATE_SYNTHETIC'); END;").unwrap(),_=>()
        }
        let before = state(&c);
        drop(c);
        let mut store = AdapterSqliteCustomerStore::new(open_database(&p).unwrap());
        let id = if fault == "customer_duplicate" {
            0x11
        } else {
            0x12
        };
        let event = if fault == "audit_duplicate" {
            0x21
        } else {
            0x22
        };
        let e = create_customer(
            "PRIVATE_REJECTED_NAME".into(),
            Some("(11) 91234-5678".into()),
            &mut FixedIds::new([id; 16], [event; 16]),
            &FixedClock,
            &mut store,
        )
        .err()
        .unwrap();
        assert_eq!(e, CreateCustomerError::StoreFailed);
        opaque(&e);
        let c = open_database(&p).unwrap();
        health(&c);
        assert_eq!(state(&c), before);
        c.execute_batch("DROP TRIGGER IF EXISTS main.cus005_fault;")
            .unwrap();
        drop(c);
        let e = create_customer(
            "Valid retry".into(),
            None,
            &mut FixedIds::new([0x33; 16], [0x44; 16]),
            &FixedClock,
            &mut store,
        )
        .err()
        .expect("poisoned adapter reused");
        assert_eq!(e, CreateCustomerError::StoreFailed);
        assert_eq!(state(&open_database(&p).unwrap()), before);
        drop(store);
        let mut fresh = AdapterSqliteCustomerStore::new(open_database(&p).unwrap());
        create_customer(
            "Recovered".into(),
            None,
            &mut FixedIds::new([0x33; 16], [0x44; 16]),
            &FixedClock,
            &mut fresh,
        )
        .unwrap();
        drop(fresh);
        let c = open_database(&p).unwrap();
        health(&c);
        let after = state(&c);
        assert_eq!(after.0, before.0);
        assert_eq!(after.2, before.2);
        assert_eq!(after.3.len(), before.3.len() + 1);
        assert_eq!(after.1.len(), before.1.len() + 1);
        for row in before.1 {
            assert!(after.1.contains(&row));
        }
        assert!(after.1.contains(&audit(0x33, 0x44)));
    }
}

#[test]
fn cus005_invalid_audit_metadata_and_readonly_connection_rollback_without_leaking_causes() {
    for fault in ["zero_customer", "zero_event", "negative_time", "query_only"] {
        let (_d, p) = setup();
        let c = open_database(&p).unwrap();
        let before = state(&c);
        if fault == "query_only" {
            c.pragma_update(None, "query_only", true).unwrap();
        }
        let mut store = AdapterSqliteCustomerStore::new(c);
        let value = customer(
            if fault == "zero_customer" { 0 } else { 0x55 },
            "PRIVATE_NAME",
            Some("(99) 99999-9999"),
        );
        let event = if fault == "zero_event" {
            [0; 16]
        } else {
            [0x66; 16]
        };
        let at = if fault == "negative_time" { -1 } else { 1234 };
        let e = store
            .persist_customer_with_audit(&value, event, at)
            .unwrap_err();
        opaque(&e);
        assert_eq!(
            format!("{e:?}/{e}"),
            "CustomerStoreError/customer could not be stored"
        );
        drop(store);
        let c = open_database(&p).unwrap();
        health(&c);
        assert_eq!(state(&c), before);
    }
}
