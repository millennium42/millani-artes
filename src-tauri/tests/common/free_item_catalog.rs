use super::entry_persistence::{FixedClock, FixedIds};
use super::TempDirectory;
use millani_artes::application::create_product::create_product;
use millani_artes::application::deactivate_product::deactivate_product;
use millani_artes::database::{
    execute_versioned_migration, open_database, CUSTOMERS_MIGRATION_SQL,
    MOVEMENT_CATEGORIES_MIGRATION_SQL, PRODUCTS_MIGRATION_SQL, RECURRENCES_MIGRATION_SQL,
};
use millani_artes::domain::{create_free_item, DiscountInput, DomainError, SaleItem};
use millani_artes::infrastructure::products::AdapterSqliteProductStore;
use rusqlite::{types::Value, Connection};
use std::path::{Path, PathBuf};

type Rows = Vec<Vec<Value>>;
type State = (Rows, Vec<(String, Rows)>);
const DESCRIPTION: &str = "  Serviço 'Á' \"livre\"\r\n\0  ";

fn add_product(path: &Path, id: u8, event: u8, name: &str) {
    let mut store = AdapterSqliteProductStore::new(open_database(path).unwrap());
    create_product(
        name.into(),
        DESCRIPTION.into(),
        1500,
        &mut FixedIds::new([id; 16], [event; 16]),
        &FixedClock,
        &mut store,
    )
    .unwrap();
}

fn setup(seeded: bool) -> (TempDirectory, PathBuf) {
    let (directory, path, _) = super::entry_persistence::setup();
    let mut c = open_database(&path).unwrap();
    for (version, sql) in [
        (6, MOVEMENT_CATEGORIES_MIGRATION_SQL),
        (7, RECURRENCES_MIGRATION_SQL),
        (8, PRODUCTS_MIGRATION_SQL),
        (9, CUSTOMERS_MIGRATION_SQL),
    ] {
        c = execute_versioned_migration(c, version, sql).unwrap();
    }
    drop(c);
    if seeded {
        add_product(&path, 0x21, 0x31, DESCRIPTION);
        add_product(&path, 0x22, 0x32, "Outro produto sintético");
        let mut store = AdapterSqliteProductStore::new(open_database(&path).unwrap());
        deactivate_product(
            [0x22; 16],
            &mut FixedIds::new([0x33; 16], [0x34; 16]),
            &FixedClock,
            &mut store,
        )
        .unwrap();
    }
    (directory, path)
}

fn rows(c: &Connection, sql: &str) -> Rows {
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

fn state(c: &Connection) -> State {
    let tables = rows(c, "SELECT name FROM main.sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name");
    let all = tables
        .into_iter()
        .map(|row| {
            let Value::Text(name) = &row[0] else {
                panic!("SAL006_TABLE_NAME_INVALID")
            };
            let query = format!("SELECT * FROM main.\"{}\"", name.replace('"', "\"\""));
            let count = c.prepare(&query).unwrap().column_count();
            let order = (1..=count)
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(",");
            (name.clone(), rows(c, &format!("{query} ORDER BY {order}")))
        })
        .collect();
    (rows(c, "SELECT id,name,description,suggested_price_cents,is_active FROM main.products ORDER BY id"), all)
}

fn assert_unchanged(path: &Path, before: &State) {
    let observer = open_database(path).unwrap();
    assert!(observer.is_autocommit());
    assert_eq!(
        super::entry_persistence::snapshot(&observer).3,
        [1, 2, 3, 4, 5, 6, 7, 8, 9]
    );
    assert_eq!(
        rows(&observer, "PRAGMA main.integrity_check"),
        vec![vec![Value::Text("ok".into())]]
    );
    assert!(rows(&observer, "PRAGMA main.foreign_key_check").is_empty());
    let after = state(&observer);
    assert_eq!(
        after.0.len(),
        before.0.len(),
        "SAL006_CATALOG_COUNT_CHANGED"
    );
    assert_eq!(after.0, before.0, "SAL006_CATALOG_ROWS_CHANGED");
    assert_eq!(after.1, before.1, "SAL006_DATABASE_CHANGED");
}

fn check_free(item: &SaleItem, id: [u8; 16], description: &str, amounts: [i64; 4]) {
    assert_eq!(item.id, id);
    assert_eq!(item.product_id, None);
    assert_eq!(item.description, description);
    assert_eq!(
        [
            item.quantity,
            item.unit_price_cents,
            item.discount_cents,
            item.total_cents
        ],
        amounts
    );
}

#[test]
fn sal006_valid_free_items_leave_catalog_and_database_unchanged_after_reopen() {
    for seeded in [false, true] {
        let (_directory, path) = setup(seeded);
        let before = state(&open_database(&path).unwrap());
        assert_eq!(before.0.len(), if seeded { 2 } else { 0 });
        for (id, quantity, price, input, discount, total) in [
            (0x61, 1, 1000, DiscountInput::AmountCents(0), 0, 1000),
            (0x21, 2, 1500, DiscountInput::AmountCents(300), 300, 2700),
            (0x22, 1, 5, DiscountInput::PercentageBasisPoints(1000), 1, 4),
            (
                0x62,
                2,
                i64::MAX,
                DiscountInput::PercentageBasisPoints(5000),
                i64::MAX,
                i64::MAX,
            ),
            (0x63, 1, 0, DiscountInput::AmountCents(0), 0, 0),
        ] {
            let item =
                create_free_item([id; 16], DESCRIPTION.into(), quantity, price, input).unwrap();
            check_free(
                &item,
                [id; 16],
                DESCRIPTION,
                [quantity, price, discount, total],
            );
            // SAL006_CATALOG_ORACLE_MUTATION_POINT
            assert_unchanged(&path, &before);
        }
        assert_unchanged(&path, &before);
    }
}

#[test]
fn sal006_invalid_free_items_leave_catalog_and_database_unchanged_after_reopen() {
    for seeded in [false, true] {
        let (_directory, path) = setup(seeded);
        let before = state(&open_database(&path).unwrap());
        assert_eq!(before.0.len(), if seeded { 2 } else { 0 });
        for (description, quantity, price, input) in [
            ("", 1, 1000, DiscountInput::AmountCents(0)),
            (
                " \t\u{2003}\u{00a0}",
                1,
                1000,
                DiscountInput::AmountCents(0),
            ),
            (DESCRIPTION, 0, 1000, DiscountInput::AmountCents(0)),
            (DESCRIPTION, -1, 1000, DiscountInput::AmountCents(0)),
            (DESCRIPTION, 1, -1, DiscountInput::AmountCents(0)),
            (DESCRIPTION, 1, 1000, DiscountInput::AmountCents(-1)),
            (DESCRIPTION, 1, 1000, DiscountInput::AmountCents(1001)),
            (
                DESCRIPTION,
                1,
                1000,
                DiscountInput::PercentageBasisPoints(10001),
            ),
            (DESCRIPTION, 2, i64::MAX, DiscountInput::AmountCents(0)),
        ] {
            assert_eq!(
                create_free_item([0x21; 16], description.into(), quantity, price, input).err(),
                Some(DomainError::InvalidInput)
            );
            assert_unchanged(&path, &before);
        }
        assert_unchanged(&path, &before);
    }
}
