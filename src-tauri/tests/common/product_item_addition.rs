use super::entry_persistence::{FixedClock, FixedIds};
use super::TempDirectory;
use millani_artes::application::add_free_item::{add_free_item, FreeItemInput};
use millani_artes::application::add_product_item::{
    add_product_item, read_sale, ProductItemInput, SaleError,
};
use millani_artes::application::create_product::create_product;
use millani_artes::database::*;
use millani_artes::domain::{DiscountInput, Sale, SaleStatus};
use millani_artes::infrastructure::products::AdapterSqliteProductStore;
use millani_artes::infrastructure::sales::AdapterSqliteSaleStore;
use rusqlite::{params, types::Value, Connection};
use std::path::PathBuf;

fn setup() -> (TempDirectory, PathBuf) {
    let d = TempDirectory::new();
    let p = d.path().join("sale.sqlite");
    let mut c = execute_versioned_migration(open_database(&p).unwrap(), 1, "").unwrap();
    for (v, sql) in [
        (2, AUDIT_EVENT_MIGRATION_SQL),
        (3, ACCOUNTS_MIGRATION_SQL),
        (4, CATEGORIES_MIGRATION_SQL),
        (5, MOVEMENTS_MIGRATION_SQL),
        (6, MOVEMENT_CATEGORIES_MIGRATION_SQL),
        (7, RECURRENCES_MIGRATION_SQL),
        (8, PRODUCTS_MIGRATION_SQL),
        (9, CUSTOMERS_MIGRATION_SQL),
        (10, SALES_MIGRATION_SQL),
        (11, SALE_ITEMS_MIGRATION_SQL),
    ] {
        c = execute_versioned_migration(c, v, sql).unwrap();
    }
    let mut products = AdapterSqliteProductStore::new(c);
    for (id, description, price) in [(0x11, "  Descrição\0 🧵\r\n  ", 1500), (0x12, "", 0)] {
        create_product(
            "Produto".into(),
            description.into(),
            price,
            &mut FixedIds::new([id; 16], [id + 0x10; 16]),
            &FixedClock,
            &mut products,
        )
        .unwrap();
    }
    (d, p)
}
fn input(
    sale: Option<u8>,
    product: u8,
    q: i64,
    p: i64,
    discount: DiscountInput,
) -> ProductItemInput {
    ProductItemInput {
        sale_id: sale.map(|v| [v; 16]),
        product_id: [product; 16],
        quantity: q,
        unit_price_cents: p,
        discount,
    }
}
struct Ids {
    next: u8,
}
impl millani_artes::domain::ports::UuidGenerator for Ids {
    type Error = ();
    fn generate(&mut self) -> Result<[u8; 16], ()> {
        let id = self.next;
        self.next += 1;
        Ok([id; 16])
    }
}
fn rows(c: &Connection, sql: &str) -> Vec<Vec<Value>> {
    let mut s = c.prepare(sql).unwrap();
    let n = s.column_count();
    s.query_map([], |r| {
        (0..n).map(|i| r.get(i)).collect::<rusqlite::Result<_>>()
    })
    .unwrap()
    .collect::<rusqlite::Result<_>>()
    .unwrap()
}
fn state(c: &Connection) -> Vec<Vec<Vec<Value>>> {
    [
        "schema_migration",
        "accounts",
        "categories",
        "movements",
        "audit_event",
        "recurrences",
        "products",
        "customers",
        "sales",
        "sale_items",
    ]
    .iter()
    .map(|t| rows(c, &format!("SELECT * FROM main.{t} ORDER BY 1")))
    .collect()
}
fn health(c: &Connection) {
    assert!(c.is_autocommit());
    assert_eq!(schema_version(c, 11).unwrap(), 11);
    assert_eq!(
        rows(c, "PRAGMA main.integrity_check"),
        vec![vec![Value::Text("ok".into())]]
    );
    assert!(rows(c, "PRAGMA main.foreign_key_check").is_empty());
}
fn load(p: &std::path::Path, id: u8) -> Sale {
    let mut s = AdapterSqliteSaleStore::new(open_database(p).unwrap());
    read_sale(&mut s, [id; 16]).unwrap().unwrap()
}
fn add(
    s: &mut AdapterSqliteSaleStore,
    sale: Option<u8>,
    seed: u8,
) -> millani_artes::application::add_product_item::AddedProductItem {
    add_product_item(
        input(sale, 0x11, 2, 1500, DiscountInput::AmountCents(0)),
        &mut Ids { next: seed },
        &FixedClock,
        s,
    )
    .unwrap()
}

fn free_input(
    sale: Option<u8>,
    description: &str,
    quantity: i64,
    price: i64,
    discount: DiscountInput,
) -> FreeItemInput {
    FreeItemInput {
        sale_id: sale.map(|id| [id; 16]),
        description: description.into(),
        quantity,
        unit_price_cents: price,
        discount,
    }
}

fn add_free(
    store: &mut AdapterSqliteSaleStore,
    sale: Option<u8>,
    seed: u8,
    description: &str,
    quantity: i64,
    price: i64,
    discount: DiscountInput,
) -> Result<millani_artes::application::add_free_item::AddedFreeItem, SaleError> {
    add_free_item(
        free_input(sale, description, quantity, price, discount),
        &mut Ids { next: seed },
        &FixedClock,
        store,
    )
}

#[test]
fn sal012_production_create_append_order_audit_and_reopen_without_cash_effects() {
    let (_d, p) = setup();
    let before = state(&open_database(&p).unwrap());
    let mut s = AdapterSqliteSaleStore::new(open_database(&p).unwrap());
    let a = add(&mut s, None, 0x31);
    assert_eq!(a.sale_id, [0x31; 16]);
    assert_eq!(a.item.id, [0x32; 16]);
    assert_eq!(a.item.description, "  Descrição\0 🧵\r\n  ");
    assert_eq!(a.item.total_cents, 3000);
    let b = add(&mut s, Some(0x31), 0x34);
    assert_eq!(b.item.id, [0x34; 16]);
    add(&mut s, None, 0x40);
    drop(s);
    let sale = load(&p, 0x31);
    assert_eq!(sale.id, [0x31; 16]);
    assert!(matches!(sale.status, SaleStatus::Open));
    assert_eq!(
        sale.items.iter().map(|v| v.id).collect::<Vec<_>>(),
        vec![[0x32; 16], [0x34; 16]]
    );
    for i in sale.items {
        assert_eq!(i.product_id, Some([0x11; 16]));
        assert_eq!(i.quantity, 2);
        assert_eq!(i.unit_price_cents, 1500);
        assert_eq!(i.discount_cents, 0);
        assert_eq!(i.total_cents, 3000);
    }
    assert_eq!(load(&p, 0x40).items.len(), 1);
    let c = open_database(&p).unwrap();
    health(&c);
    let after = state(&c);
    for i in [0, 1, 2, 3, 5, 6, 7] {
        assert_eq!(before[i], after[i]);
    }
    assert_eq!(after[4].len(), before[4].len() + 3, "AUDIT_COUNT");
    for (event, item) in [(0x33, 0x32), (0x35, 0x34), (0x42, 0x41)] {
        let audit=rows(&c,&format!("SELECT occurred_at_ms,operation,entity FROM main.audit_event WHERE event_id=x'{}' AND entity_id=x'{}'",format!("{event:02x}").repeat(16),format!("{item:02x}").repeat(16)));
        assert_eq!(
            audit,
            vec![vec![
                Value::Integer(1234),
                Value::Text("sale_item_add".into()),
                Value::Text("sale_item".into())
            ]]
        );
    }
    assert_eq!(rows(&c,"SELECT position FROM main.sale_items WHERE sale_id=x'31313131313131313131313131313131' ORDER BY position"),vec![vec![Value::Integer(0)],vec![Value::Integer(1)]]);
}

#[test]
fn sal012_selected_price_zero_percentage_rounding_and_wide_money_roundtrip() {
    let (_d, p) = setup();
    let mut s = AdapterSqliteSaleStore::new(open_database(&p).unwrap());
    for (seed, product, q, price, discount, applied, total) in [
        (0x30, 0x12, 1, 0, DiscountInput::AmountCents(0), 0, 0),
        (
            0x40,
            0x11,
            1,
            5,
            DiscountInput::PercentageBasisPoints(1000),
            1,
            4,
        ),
        (
            0x50,
            0x11,
            2,
            i64::MAX,
            DiscountInput::AmountCents(i64::MAX),
            i64::MAX,
            i64::MAX,
        ),
        (
            0x60,
            0x11,
            2,
            i64::MAX,
            DiscountInput::PercentageBasisPoints(5000),
            i64::MAX,
            i64::MAX,
        ),
    ] {
        let r = add_product_item(
            input(None, product, q, price, discount),
            &mut Ids { next: seed },
            &FixedClock,
            &mut s,
        )
        .unwrap();
        assert_eq!(r.item.discount_cents, applied);
        assert_eq!(r.item.total_cents, total);
        let sale = load(&p, seed);
        assert_eq!(sale.items.len(), 1);
        assert_eq!(sale.items[0].unit_price_cents, price);
        assert_eq!(sale.items[0].total_cents, total);
        assert_eq!(
            sale.items[0].description,
            if product == 0x12 {
                ""
            } else {
                "  Descrição\0 🧵\r\n  "
            }
        );
    }
    health(&open_database(&p).unwrap());
}

#[test]
fn sal012_business_rejections_have_no_effect_and_adapter_remains_usable() {
    let (_d, p) = setup();
    let c = open_database(&p).unwrap();
    c.execute(
        "UPDATE main.products SET is_active=0 WHERE id=?1",
        params![[0x12u8; 16].as_slice()],
    )
    .unwrap();
    drop(c);
    let mut s = AdapterSqliteSaleStore::new(open_database(&p).unwrap());
    add(&mut s, None, 0x30);
    for (i, expected) in [
        (
            input(Some(0xFF), 0x11, 1, 1, DiscountInput::AmountCents(0)),
            SaleError::SaleNotFound,
        ),
        (
            input(None, 0xFF, 1, 1, DiscountInput::AmountCents(0)),
            SaleError::ProductNotFound,
        ),
        (
            input(None, 0x12, 1, 1, DiscountInput::AmountCents(0)),
            SaleError::InvalidItem,
        ),
        (
            input(None, 0x11, 0, 1, DiscountInput::AmountCents(0)),
            SaleError::InvalidItem,
        ),
        (
            input(None, 0x11, 1, -1, DiscountInput::AmountCents(0)),
            SaleError::InvalidItem,
        ),
        (
            input(None, 0x11, 1, 1, DiscountInput::AmountCents(-1)),
            SaleError::InvalidItem,
        ),
        (
            input(None, 0x11, 1, 1, DiscountInput::AmountCents(2)),
            SaleError::InvalidItem,
        ),
        (
            input(None, 0x11, 1, 1, DiscountInput::PercentageBasisPoints(-1)),
            SaleError::InvalidItem,
        ),
        (
            input(
                None,
                0x11,
                1,
                1,
                DiscountInput::PercentageBasisPoints(10001),
            ),
            SaleError::InvalidItem,
        ),
        (
            input(None, 0x11, 2, i64::MAX, DiscountInput::AmountCents(0)),
            SaleError::InvalidItem,
        ),
    ] {
        let before = state(&open_database(&p).unwrap());
        assert_eq!(
            add_product_item(i, &mut Ids { next: 0x50 }, &FixedClock, &mut s).err(),
            Some(expected)
        );
        assert_eq!(state(&open_database(&p).unwrap()), before);
    }
    add(&mut s, Some(0x30), 0x60);
    assert_eq!(load(&p, 0x30).items.len(), 2);
    assert!(read_sale(&mut s, [0xFF; 16]).unwrap().is_none());
}

#[test]
fn sal012_finalized_and_exhausted_position_reject_without_overwrite() {
    for finalized in [false, true] {
        let (_d, p) = setup();
        let mut s = AdapterSqliteSaleStore::new(open_database(&p).unwrap());
        add(&mut s, None, 0x30);
        let c = open_database(&p).unwrap();
        if finalized {
            c.execute("UPDATE main.sales SET status='finalized'", [])
                .unwrap();
        } else {
            c.execute("UPDATE main.sale_items SET position=?1", [i64::MAX])
                .unwrap();
        }
        let before = state(&c);
        drop(c);
        assert_eq!(
            add_product_item(
                input(Some(0x30), 0x11, 1, 1, DiscountInput::AmountCents(0)),
                &mut Ids { next: 0x50 },
                &FixedClock,
                &mut s
            )
            .err(),
            Some(if finalized {
                SaleError::SaleNotOpen
            } else {
                SaleError::PositionExhausted
            })
        );
        assert_eq!(state(&open_database(&p).unwrap()), before);
        assert_eq!(load(&p, 0x30).items.len(), 1);
        if finalized {
            assert!(matches!(load(&p, 0x30).status, SaleStatus::Finalized));
        }
    }
}

#[test]
fn sal012_atomic_faults_poison_adapter_and_fresh_store_recovers() {
    for (table, op, action) in [
        ("sales", "INSERT", "ABORT"),
        ("sales", "INSERT", "IGNORE"),
        ("sale_items", "INSERT", "ABORT"),
        ("sale_items", "INSERT", "IGNORE"),
        ("audit_event", "INSERT", "ABORT"),
        ("audit_event", "INSERT", "IGNORE"),
    ] {
        let (_d, p) = setup();
        let c = open_database(&p).unwrap();
        c.execute_batch(&format!("CREATE TRIGGER sal012_fault BEFORE {op} ON main.{table} BEGIN SELECT RAISE({action}{}); END;",if action=="ABORT"{",'PRIVATE'"}else{""})).unwrap();
        let before = state(&c);
        let mut s = AdapterSqliteSaleStore::new(c);
        assert_eq!(
            add_product_item(
                input(None, 0x11, 2, 1500, DiscountInput::AmountCents(0)),
                &mut Ids { next: 0x30 },
                &FixedClock,
                &mut s
            )
            .err(),
            Some(SaleError::StoreFailed)
        );
        let c = open_database(&p).unwrap();
        health(&c);
        assert_eq!(state(&c), before);
        c.execute_batch("DROP TRIGGER main.sal012_fault").unwrap();
        drop(c);
        assert_eq!(
            add_product_item(
                input(None, 0x11, 1, 1, DiscountInput::AmountCents(0)),
                &mut Ids { next: 0x40 },
                &FixedClock,
                &mut s
            )
            .err(),
            Some(SaleError::StoreFailed)
        );
        assert_eq!(
            read_sale(&mut s, [0x30; 16]).err(),
            Some(SaleError::StoreFailed)
        );
        drop(s);
        let mut fresh = AdapterSqliteSaleStore::new(open_database(&p).unwrap());
        add(&mut fresh, None, 0x50);
        assert_eq!(load(&p, 0x50).items.len(), 1);
    }
}

#[test]
fn sal012_collisions_metadata_query_only_and_lock_leave_all_tables_intact() {
    for fault in [
        "sale_collision",
        "item_collision",
        "audit_collision",
        "zero_item",
        "zero_event",
        "query_only",
        "locked",
    ] {
        let (_d, p) = setup();
        let mut s = AdapterSqliteSaleStore::new(open_database(&p).unwrap());
        add(&mut s, None, 0x30);
        drop(s);
        let c = open_database(&p).unwrap();
        let before = state(&c);
        let lock = if fault == "locked" {
            let l = open_database(&p).unwrap();
            l.execute_batch("BEGIN IMMEDIATE").unwrap();
            Some(l)
        } else {
            None
        };
        if fault == "query_only" {
            c.pragma_update(None, "query_only", true).unwrap();
        }
        c.busy_timeout(std::time::Duration::ZERO).unwrap();
        let mut s = AdapterSqliteSaleStore::new(c);
        let seed = match fault {
            "sale_collision" => 0x30,
            "item_collision" => 0x31,
            "audit_collision" => 0x32,
            _ => 0x60,
        };
        let sale = if matches!(fault, "sale_collision" | "query_only" | "locked") {
            None
        } else {
            Some(0x30)
        };
        struct FaultIds {
            values: std::vec::IntoIter<[u8; 16]>,
        }
        impl millani_artes::domain::ports::UuidGenerator for FaultIds {
            type Error = ();
            fn generate(&mut self) -> Result<[u8; 16], ()> {
                Ok(self.values.next().unwrap())
            }
        }
        let mut ids = FaultIds {
            values: if fault == "zero_item" {
                vec![[0; 16], [0x70; 16]]
            } else if fault == "zero_event" {
                vec![[0x70; 16], [0; 16]]
            } else if fault == "audit_collision" {
                vec![[0x70; 16], [0x32; 16]]
            } else {
                (seed..seed + 3).map(|v| [v; 16]).collect()
            }
            .into_iter(),
        };
        assert_eq!(
            add_product_item(
                input(sale, 0x11, 1, 1, DiscountInput::AmountCents(0)),
                &mut ids,
                &FixedClock,
                &mut s
            )
            .err(),
            Some(SaleError::StoreFailed),
            "{fault}"
        );
        drop(lock);
        drop(s);
        let c = open_database(&p).unwrap();
        health(&c);
        assert_eq!(state(&c), before);
    }
}

#[test]
fn sal012_main_qualification_and_snapshot_read_do_not_consult_live_product() {
    let (_d, p) = setup();
    let c = open_database(&p).unwrap();
    c.execute_batch("CREATE TEMP TABLE sales(id BLOB,status TEXT); CREATE TEMP TABLE products(id BLOB); CREATE TEMP TABLE sale_items(id BLOB); CREATE TEMP TABLE audit_event(event_id BLOB);").unwrap();
    let mut s = AdapterSqliteSaleStore::new(c);
    add(&mut s, None, 0x30);
    assert_eq!(
        read_sale(&mut s, [0x30; 16]).unwrap().unwrap().items[0].total_cents,
        3000
    );
    drop(s);
    // Fault fixture removes the catalog table after persistence; reads use only snapshot columns.
    let c = Connection::open(&p).unwrap();
    c.execute_batch("PRAGMA foreign_keys=OFF; DROP TABLE main.products;")
        .unwrap();
    drop(c);
    let sale = load(&p, 0x30);
    assert_eq!(sale.items[0].description, "  Descrição\0 🧵\r\n  ");
    assert_eq!(sale.items[0].unit_price_cents, 1500);
}

#[test]
fn sal012_read_rejects_corrupt_rows_without_partial_result() {
    for fault in [
        "status",
        "item_id",
        "product_id",
        "quantity",
        "money",
        "total",
        "position",
        "missing_column",
    ] {
        let (_d, p) = setup();
        let mut s = AdapterSqliteSaleStore::new(open_database(&p).unwrap());
        add(&mut s, None, 0x30);
        add(&mut s, Some(0x30), 0x40);
        drop(s);
        let c = open_database(&p).unwrap();
        c.execute_batch("PRAGMA ignore_check_constraints=ON; PRAGMA foreign_keys=OFF;")
            .unwrap();
        let sql = match fault {
            "status" => "UPDATE main.sales SET status='PRIVATE'",
            "item_id" => "UPDATE main.sale_items SET id=x'01' WHERE position=1",
            "product_id" => "UPDATE main.sale_items SET product_id=x'01' WHERE position=1",
            "quantity" => "UPDATE main.sale_items SET quantity=0 WHERE position=1",
            "money" => "UPDATE main.sale_items SET unit_price_cents=-1 WHERE position=1",
            "total" => "UPDATE main.sale_items SET total_cents=2999 WHERE position=1",
            "position" => "UPDATE main.sale_items SET position=-1 WHERE position=1",
            _ => "ALTER TABLE main.sale_items RENAME COLUMN description TO broken",
        };
        c.execute_batch(sql).unwrap();
        drop(c);
        let mut s = AdapterSqliteSaleStore::new(open_database(&p).unwrap());
        let e = read_sale(&mut s, [0x30; 16]).err().unwrap();
        assert_eq!(e, SaleError::StoreFailed);
        assert!(!format!("{e:?} {e}").contains("PRIVATE"));
        assert!(std::error::Error::source(&e).is_none());
    }
}

#[test]
fn sal012_position_gaps_and_nullable_product_snapshots_are_read_in_order() {
    let (_d, p) = setup();
    let mut s = AdapterSqliteSaleStore::new(open_database(&p).unwrap());
    add(&mut s, None, 0x30);
    let c = open_database(&p).unwrap();
    c.execute("UPDATE main.sale_items SET position=5,product_id=NULL", [])
        .unwrap();
    drop(c);
    add(&mut s, Some(0x30), 0x40);
    drop(s);
    let sale = load(&p, 0x30);
    assert_eq!(sale.items.len(), 2);
    assert!(sale.items[0].product_id.is_none());
    assert_eq!(sale.items[1].product_id, Some([0x11; 16]));
    assert_eq!(
        rows(
            &open_database(&p).unwrap(),
            "SELECT position FROM main.sale_items ORDER BY position"
        ),
        vec![vec![Value::Integer(5)], vec![Value::Integer(6)]]
    );
}

#[test]
fn sal012_sql_lookup_failures_rollback_and_read_header_errors_are_opaque() {
    for table in ["sales", "products", "sale_items"] {
        let (_d, p) = setup();
        let c = open_database(&p).unwrap();
        c.execute_batch(&format!("ALTER TABLE main.{table} RENAME TO broken"))
            .unwrap();
        let before = rows(&c, "SELECT * FROM main.audit_event ORDER BY event_id");
        let mut s = AdapterSqliteSaleStore::new(c);
        assert_eq!(
            add_product_item(
                input(Some(0x80), 0x11, 1, 1, DiscountInput::AmountCents(0)),
                &mut Ids { next: 0x50 },
                &FixedClock,
                &mut s
            )
            .err(),
            Some(if table == "sales" {
                SaleError::StoreFailed
            } else {
                SaleError::SaleNotFound
            })
        );
        if table != "sales" {
            assert_eq!(
                add_product_item(
                    input(None, 0x11, 1, 1, DiscountInput::AmountCents(0)),
                    &mut Ids { next: 0x60 },
                    &FixedClock,
                    &mut s
                )
                .err(),
                Some(SaleError::StoreFailed)
            );
        }
        assert_eq!(
            rows(
                &open_database(&p).unwrap(),
                "SELECT * FROM main.audit_event ORDER BY event_id"
            ),
            before
        );
        let mut fresh = AdapterSqliteSaleStore::new(open_database(&p).unwrap());
        if table == "sales" {
            assert_eq!(
                read_sale(&mut fresh, [0x80; 16]).err(),
                Some(SaleError::StoreFailed)
            );
        }
    }
}

#[test]
fn sal013_free_item_persists_snapshot_order_audit_and_no_catalog_or_cash_effects() {
    let (_d, p) = setup();
    let description = "  Serviço 'Á'\0 🧵\r\n  ";
    let before = state(&open_database(&p).unwrap());
    let mut store = AdapterSqliteSaleStore::new(open_database(&p).unwrap());
    let first = add_free(
        &mut store,
        None,
        0x50,
        description,
        2,
        1500,
        DiscountInput::AmountCents(300),
    )
    .unwrap();
    assert_eq!(first.sale_id, [0x50; 16]);
    assert_eq!(first.item.id, [0x51; 16]);
    assert_eq!(first.item.product_id, None);
    assert_eq!(first.item.description, description);
    assert_eq!(first.item.discount_cents, 300);
    assert_eq!(first.item.total_cents, 2700);
    let second = add_free(
        &mut store,
        Some(0x50),
        0x53,
        "Serviço",
        1,
        5,
        DiscountInput::PercentageBasisPoints(1000),
    )
    .unwrap();
    assert_eq!(second.sale_id, [0x50; 16]);
    assert_eq!(second.item.id, [0x53; 16]);
    assert_eq!(second.item.product_id, None);
    assert_eq!(
        [second.item.discount_cents, second.item.total_cents],
        [1, 4]
    );
    drop(store);

    let mut reopened = AdapterSqliteSaleStore::new(open_database(&p).unwrap());
    let sale = read_sale(&mut reopened, [0x50; 16]).unwrap().unwrap();
    assert!(matches!(sale.status, SaleStatus::Open));
    assert_eq!(sale.items.len(), 2);
    assert_eq!(sale.items[0].id, [0x51; 16]);
    assert_eq!(sale.items[0].product_id, None);
    assert_eq!(sale.items[0].description, description);
    assert_eq!(sale.items[0].total_cents, 2700);
    assert_eq!(sale.items[1].id, [0x53; 16]);
    assert_eq!(sale.items[1].product_id, None);
    assert_eq!(sale.items[1].description, "Serviço");
    assert_eq!(sale.items[1].total_cents, 4);

    let c = open_database(&p).unwrap();
    health(&c);
    let after = state(&c);
    for table in [0, 1, 2, 3, 5, 6, 7] {
        assert_eq!(after[table], before[table], "TABLE_{table}_CHANGED");
    }
    assert_eq!(after[4].len(), before[4].len() + 2, "AUDIT_COUNT");
    assert_eq!(after[8].len(), before[8].len() + 1, "SALE_COUNT");
    assert_eq!(after[9].len(), before[9].len() + 2, "SALE_ITEM_COUNT");
    assert_eq!(
        rows(
            &c,
            "SELECT product_id FROM main.sale_items WHERE sale_id=x'50505050505050505050505050505050' ORDER BY position"
        ),
        vec![vec![Value::Null], vec![Value::Null]]
    );
    for (event, item) in [(0x52, 0x51), (0x54, 0x53)] {
        assert_eq!(
            rows(&c, &format!(
                "SELECT occurred_at_ms,operation,entity,entity_id FROM main.audit_event WHERE event_id=x'{}'",
                format!("{event:02x}").repeat(16)
            )),
            vec![vec![
                Value::Integer(1234),
                Value::Text("sale_item_add".into()),
                Value::Text("sale_item".into()),
                Value::Blob(vec![item; 16]),
            ]]
        );
    }
}

#[test]
fn sal013_free_item_zero_ties_and_wide_amounts_roundtrip() {
    let (_d, p) = setup();
    let before = state(&open_database(&p).unwrap());
    let mut store = AdapterSqliteSaleStore::new(open_database(&p).unwrap());
    for (seed, quantity, price, discount, expected_discount, expected_total) in [
        (0x60, 1, 0, DiscountInput::AmountCents(0), 0, 0),
        (0x70, 1, 5, DiscountInput::PercentageBasisPoints(1000), 1, 4),
        (
            0x80,
            2,
            i64::MAX,
            DiscountInput::AmountCents(i64::MAX),
            i64::MAX,
            i64::MAX,
        ),
    ] {
        let added = add_free(&mut store, None, seed, "Livre", quantity, price, discount).unwrap();
        assert_eq!(added.item.product_id, None);
        assert_eq!(added.item.discount_cents, expected_discount);
        assert_eq!(added.item.total_cents, expected_total);
        let sale = read_sale(&mut store, [seed; 16]).unwrap().unwrap();
        assert_eq!(sale.items[0].description, "Livre");
        assert_eq!(sale.items[0].total_cents, expected_total);
    }
    drop(store);
    let c = open_database(&p).unwrap();
    health(&c);
    let after = state(&c);
    assert_eq!(after[6], before[6], "PRODUCT_CATALOG_CHANGED");
    assert_eq!(after[3], before[3], "MOVEMENTS_CHANGED");
    assert_eq!(after[1], before[1], "ACCOUNTS_CHANGED");
}

#[test]
fn sal013_invalid_items_missing_closed_sales_and_max_position_have_no_write() {
    let (_d, p) = setup();
    let mut store = AdapterSqliteSaleStore::new(open_database(&p).unwrap());
    add_free(
        &mut store,
        None,
        0x40,
        "Original",
        1,
        1000,
        DiscountInput::AmountCents(0),
    )
    .unwrap();
    let before = state(&open_database(&p).unwrap());
    for (seed, description, quantity, price, discount) in [
        (0x60, " \t\u{2003}", 1, 1000, DiscountInput::AmountCents(0)),
        (0x63, "Livre", 0, 1000, DiscountInput::AmountCents(0)),
        (0x66, "Livre", 1, -1, DiscountInput::AmountCents(0)),
        (0x69, "Livre", 1, 1000, DiscountInput::AmountCents(1001)),
        (
            0x6c,
            "Livre",
            1,
            1000,
            DiscountInput::PercentageBasisPoints(10001),
        ),
    ] {
        assert_eq!(
            add_free(
                &mut store,
                Some(0x40),
                seed,
                description,
                quantity,
                price,
                discount
            )
            .err(),
            Some(SaleError::InvalidItem)
        );
        assert_eq!(state(&open_database(&p).unwrap()), before);
    }
    assert_eq!(
        add_free(
            &mut store,
            None,
            0x80,
            "",
            1,
            1000,
            DiscountInput::AmountCents(0)
        )
        .err(),
        Some(SaleError::InvalidItem)
    );
    assert_eq!(state(&open_database(&p).unwrap()), before);
    assert_eq!(
        add_free(
            &mut store,
            Some(0xfe),
            0x83,
            "Livre",
            1,
            1000,
            DiscountInput::AmountCents(0)
        )
        .err(),
        Some(SaleError::SaleNotFound)
    );
    assert_eq!(state(&open_database(&p).unwrap()), before);

    let c = open_database(&p).unwrap();
    c.execute(
        "UPDATE main.sales SET status='finalized' WHERE id=?1",
        params![[0x40u8; 16].as_slice()],
    )
    .unwrap();
    drop(c);
    let closed_before = state(&open_database(&p).unwrap());
    assert_eq!(
        add_free(
            &mut store,
            Some(0x40),
            0x86,
            "Livre",
            1,
            1000,
            DiscountInput::AmountCents(0)
        )
        .err(),
        Some(SaleError::SaleNotOpen)
    );
    assert_eq!(state(&open_database(&p).unwrap()), closed_before);
    health(&open_database(&p).unwrap());

    let (_d, p) = setup();
    let mut store = AdapterSqliteSaleStore::new(open_database(&p).unwrap());
    add_free(
        &mut store,
        None,
        0x40,
        "Original",
        1,
        1000,
        DiscountInput::AmountCents(0),
    )
    .unwrap();
    let c = open_database(&p).unwrap();
    c.execute(
        "UPDATE main.sale_items SET position=?1 WHERE sale_id=?2",
        params![i64::MAX, [0x40u8; 16].as_slice()],
    )
    .unwrap();
    drop(c);
    let before = state(&open_database(&p).unwrap());
    assert_eq!(
        add_free(
            &mut store,
            Some(0x40),
            0x90,
            "Outro",
            1,
            1000,
            DiscountInput::AmountCents(0)
        )
        .err(),
        Some(SaleError::PositionExhausted)
    );
    assert_eq!(state(&open_database(&p).unwrap()), before);
}

#[test]
fn sal013_sqlite_item_and_audit_failures_rollback_poison_and_recover() {
    for (trigger, sql) in [
        (
            "fail_free_item",
            "CREATE TRIGGER fail_free_item BEFORE INSERT ON sale_items BEGIN SELECT RAISE(ABORT,'PRIVATE'); END;",
        ),
        (
            "fail_free_audit",
            "CREATE TRIGGER fail_free_audit BEFORE INSERT ON audit_event WHEN NEW.operation='sale_item_add' BEGIN SELECT RAISE(ABORT,'PRIVATE'); END;",
        ),
    ] {
        let (_d, p) = setup();
        let c = open_database(&p).unwrap();
        c.execute_batch(sql).unwrap();
        let before = state(&c);
        let mut store = AdapterSqliteSaleStore::new(c);
        assert_eq!(
            add_free(
                &mut store,
                None,
                0x40,
                "Novo",
                1,
                1000,
                DiscountInput::AmountCents(0)
            )
            .err(),
            Some(SaleError::StoreFailed)
        );
        assert_eq!(
            add_free(
                &mut store,
                None,
                0x50,
                "Poisonado",
                1,
                1000,
                DiscountInput::AmountCents(0)
            )
            .err(),
            Some(SaleError::StoreFailed)
        );
        drop(store);
        let observer = open_database(&p).unwrap();
        health(&observer);
        assert_eq!(state(&observer), before, "{trigger}_PARTIAL_WRITE");
        observer.execute_batch(&format!("DROP TRIGGER main.{trigger}")).unwrap();
        drop(observer);
        let mut fresh = AdapterSqliteSaleStore::new(open_database(&p).unwrap());
        add_free(
            &mut fresh,
            None,
            0x60,
            "Recuperado",
            1,
            1000,
            DiscountInput::AmountCents(0),
        )
        .unwrap();
        drop(fresh);
        assert_eq!(load(&p, 0x60).items[0].description, "Recuperado");
        health(&open_database(&p).unwrap());
    }
}

#[test]
fn sal013_collisions_query_only_and_lock_leave_all_tables_intact() {
    for fault in [
        "sale_collision",
        "item_collision",
        "audit_collision",
        "query_only",
        "locked",
    ] {
        let (_d, p) = setup();
        let mut initial = AdapterSqliteSaleStore::new(open_database(&p).unwrap());
        add_free(
            &mut initial,
            None,
            0x30,
            "Original",
            1,
            1000,
            DiscountInput::AmountCents(0),
        )
        .unwrap();
        drop(initial);

        let c = open_database(&p).unwrap();
        let before = state(&c);
        let lock = if fault == "locked" {
            let l = open_database(&p).unwrap();
            l.execute_batch("BEGIN IMMEDIATE").unwrap();
            Some(l)
        } else {
            None
        };
        if fault == "query_only" {
            c.pragma_update(None, "query_only", true).unwrap();
        }
        c.busy_timeout(std::time::Duration::ZERO).unwrap();
        let mut store = AdapterSqliteSaleStore::new(c);
        struct FaultIds {
            values: std::vec::IntoIter<[u8; 16]>,
        }
        impl millani_artes::domain::ports::UuidGenerator for FaultIds {
            type Error = ();
            fn generate(&mut self) -> Result<[u8; 16], ()> {
                Ok(self.values.next().unwrap())
            }
        }
        let (sale_id, ids) = match fault {
            "sale_collision" => (None, vec![[0x30; 16], [0x50; 16], [0x51; 16]]),
            "item_collision" => (Some([0x30; 16]), vec![[0x31; 16], [0x50; 16]]),
            "audit_collision" => (Some([0x30; 16]), vec![[0x50; 16], [0x32; 16]]),
            "query_only" => (None, vec![[0x60; 16], [0x61; 16], [0x62; 16]]),
            "locked" => (None, vec![[0x70; 16], [0x71; 16], [0x72; 16]]),
            _ => unreachable!(),
        };
        let mut ids = FaultIds {
            values: ids.into_iter(),
        };
        assert_eq!(
            add_free_item(
                free_input(
                    sale_id.map(|id| id[0]),
                    "Livre",
                    1,
                    1000,
                    DiscountInput::AmountCents(0),
                ),
                &mut ids,
                &FixedClock,
                &mut store,
            )
            .err(),
            Some(SaleError::StoreFailed),
            "{fault}"
        );
        drop(lock);
        drop(store);
        let observer = open_database(&p).unwrap();
        health(&observer);
        assert_eq!(state(&observer), before, "{fault}_PARTIAL_WRITE");
    }
}

#[test]
fn sal013_writes_main_snapshot_even_when_temp_tables_shadow_sales_objects() {
    let (_d, p) = setup();
    let c = open_database(&p).unwrap();
    c.execute_batch(
        "CREATE TEMP TABLE sales(id BLOB,status TEXT); CREATE TEMP TABLE products(id BLOB); CREATE TEMP TABLE sale_items(id BLOB); CREATE TEMP TABLE audit_event(event_id BLOB);",
    )
    .unwrap();
    let before = state(&c);
    let mut store = AdapterSqliteSaleStore::new(c);
    add_free(
        &mut store,
        None,
        0x40,
        "Livre",
        1,
        1000,
        DiscountInput::AmountCents(0),
    )
    .unwrap();
    drop(store);
    let sale = load(&p, 0x40);
    assert_eq!(sale.items.len(), 1);
    assert_eq!(sale.items[0].product_id, None);
    assert_eq!(sale.items[0].description, "Livre");
    let c = open_database(&p).unwrap();
    let after = state(&c);
    assert_eq!(after[6], before[6], "PRODUCT_CATALOG_CHANGED");
    assert_eq!(after[4].len(), before[4].len() + 1, "MAIN_AUDIT_MISSING");
    health(&c);
}
