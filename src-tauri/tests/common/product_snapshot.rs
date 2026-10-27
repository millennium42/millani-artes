use super::entry_persistence::{snapshot, FixedClock, FixedIds};
use super::TempDirectory;
use millani_artes::application::add_product_item::{add_product_item, read_sale, ProductItemInput};
use millani_artes::application::create_product::create_product;
use millani_artes::application::deactivate_product::deactivate_product;
use millani_artes::application::edit_product::edit_product;
use millani_artes::database::*;
use millani_artes::domain::ports::UuidGenerator;
use millani_artes::domain::{DiscountInput, Sale, SaleItem, SaleStatus};
use millani_artes::infrastructure::products::AdapterSqliteProductStore;
use millani_artes::infrastructure::sales::AdapterSqliteSaleStore;
use rusqlite::params;
use std::path::{Path, PathBuf};

fn setup() -> (TempDirectory, PathBuf) {
    let (directory, path, _) = super::entry_persistence::setup();
    let mut connection = open_database(&path).unwrap();
    for (version, sql) in [
        (6, MOVEMENT_CATEGORIES_MIGRATION_SQL),
        (7, RECURRENCES_MIGRATION_SQL),
        (8, PRODUCTS_MIGRATION_SQL),
        (9, CUSTOMERS_MIGRATION_SQL),
        (10, SALES_MIGRATION_SQL),
        (11, SALE_ITEMS_MIGRATION_SQL),
    ] {
        connection = execute_versioned_migration(connection, version, sql).unwrap();
    }
    drop(connection);
    (directory, path)
}

struct Sequence(u8);
impl UuidGenerator for Sequence {
    type Error = ();
    fn generate(&mut self) -> Result<[u8; 16], Self::Error> {
        let id = self.0;
        self.0 += 1;
        Ok([id; 16])
    }
}

type ItemSnapshot = ([u8; 16], Option<[u8; 16]>, String, i64, i64, i64, i64);
fn assert_item(item: &SaleItem, expected: &ItemSnapshot) {
    assert_eq!(
        (
            item.id,
            item.product_id,
            item.description.clone(),
            item.quantity,
            item.unit_price_cents,
            item.discount_cents,
            item.total_cents
        ),
        *expected,
        "SAL008_SNAPSHOT_CHANGED"
    );
}
fn assert_sale(sale: Sale, expected: &ItemSnapshot) {
    assert_eq!(sale.id, [0x30; 16]);
    assert!(matches!(sale.status, SaleStatus::Open));
    assert_eq!(sale.items.len(), 1);
    assert_item(&sale.items[0], expected);
}
fn load(store: &mut AdapterSqliteSaleStore) -> Sale {
    read_sale(store, [0x30; 16])
        .expect("SAL008_SNAPSHOT_READ")
        .expect("SAL008_SALE_MISSING")
}
fn assert_catalog(path: &Path, description: &str, price: i64, active: bool) {
    let connection = open_database(path).unwrap();
    let actual: (String, i64, bool) = connection
        .query_row(
            "SELECT description,suggested_price_cents,is_active FROM main.products WHERE id=?1",
            params![[0x11_u8; 16].as_slice()],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(
        actual,
        (description.into(), price, active),
        "SAL008_CATALOG_NOT_CHANGED"
    );
}
fn assert_health(path: &Path) {
    let connection = open_database(path).unwrap();
    assert!(connection.is_autocommit());
    assert_eq!(schema_version(&connection, 11).unwrap(), 11);
    assert_eq!(
        connection
            .query_row("PRAGMA main.integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    let mut statement = connection.prepare("PRAGMA main.foreign_key_check").unwrap();
    assert!(statement.query([]).unwrap().next().unwrap().is_none());
}

#[test]
fn sal008_persisted_snapshots_survive_product_changes_and_reopen() {
    for (description, quantity, price, discount, applied, total) in [
        ("A", 2, 1500, DiscountInput::AmountCents(0), 0, 3000),
        (
            "  Descrição\0 🧵\r\n  ",
            1,
            1250,
            DiscountInput::PercentageBasisPoints(1000),
            125,
            1125,
        ),
        ("", 1, 0, DiscountInput::AmountCents(0), 0, 0),
        (
            "wide",
            2,
            i64::MAX,
            DiscountInput::AmountCents(i64::MAX),
            i64::MAX,
            i64::MAX,
        ),
    ] {
        let (_directory, path) = setup();
        let before = snapshot(&open_database(&path).unwrap());
        let mut products = AdapterSqliteProductStore::new(open_database(&path).unwrap());
        let created = create_product(
            "Template".into(),
            description.into(),
            1500,
            &mut FixedIds::new([0x11; 16], [0x21; 16]),
            &FixedClock,
            &mut products,
        )
        .unwrap();
        assert_eq!(created.id, [0x11; 16]);
        assert_eq!(created.description, description);
        assert_eq!(created.suggested_price_cents, 1500);
        assert!(created.is_active);
        let mut sales = AdapterSqliteSaleStore::new(open_database(&path).unwrap());
        let added = add_product_item(
            ProductItemInput {
                sale_id: None,
                product_id: [0x11; 16],
                quantity,
                unit_price_cents: price,
                discount,
            },
            &mut Sequence(0x30),
            &FixedClock,
            &mut sales,
        )
        .unwrap();
        let expected: ItemSnapshot = (
            [0x31; 16],
            Some([0x11; 16]),
            description.into(),
            quantity,
            price,
            applied,
            total,
        );
        assert_eq!(added.sale_id, [0x30; 16]);
        assert_item(&added.item, &expected);
        assert_sale(load(&mut sales), &expected);

        let edited = edit_product(
            [0x11; 16],
            "Produto B".into(),
            "B".into(),
            1700,
            &mut FixedIds::new([0x41; 16], [0xF1; 16]),
            &FixedClock,
            &mut products,
        )
        .unwrap();
        assert_eq!(edited.description, "B");
        assert_eq!(edited.suggested_price_cents, 1700);
        assert!(edited.is_active);
        assert_catalog(&path, "B", 1700, true);
        assert_sale(load(&mut sales), &expected);

        let inactive = deactivate_product(
            [0x11; 16],
            &mut FixedIds::new([0x42; 16], [0xF2; 16]),
            &FixedClock,
            &mut products,
        )
        .unwrap();
        assert_eq!(inactive.description, "B");
        assert_eq!(inactive.suggested_price_cents, 1700);
        assert!(!inactive.is_active);
        assert_catalog(&path, "B", 1700, false);
        assert_sale(load(&mut sales), &expected);
        assert_item(&added.item, &expected);
        drop(products);
        drop(sales);

        let mut fresh = AdapterSqliteSaleStore::new(open_database(&path).unwrap());
        assert_sale(load(&mut fresh), &expected);
        assert_catalog(&path, "B", 1700, false);
        assert_health(&path);
        let after = snapshot(&open_database(&path).unwrap());
        assert_eq!(after.0, before.0);
        assert_eq!(after.1, before.1);
        assert_eq!(after.3, before.3);
        assert_eq!(after.2.len(), before.2.len() + 4);
        drop(fresh);
    }
}
