use super::entry_persistence::{setup as setup_finance, FixedClock, FixedIds};
use super::TempDirectory;
use millani_artes::application::add_product_item::{
    add_product_item, read_sale, ProductItemInput, SaleError,
};
use millani_artes::application::create_entry::create_entry;
use millani_artes::application::create_product::create_product;
use millani_artes::application::remove_sale_item::{remove_sale_item, RemoveSaleItemInput};
use millani_artes::database::*;
use millani_artes::domain::ports::UuidGenerator;
use millani_artes::domain::{Context, DiscountInput, Sale, SaleStatus};
use millani_artes::infrastructure::movements::AdapterSqliteMovementStore;
use millani_artes::infrastructure::products::AdapterSqliteProductStore;
use millani_artes::infrastructure::sales::AdapterSqliteSaleStore;
use rusqlite::{params, types::Value, Connection};
use std::path::PathBuf;
use std::time::Duration;

struct Sequence(u8);
impl UuidGenerator for Sequence {
    type Error = ();
    fn generate(&mut self) -> Result<[u8; 16], ()> {
        let id = self.0;
        self.0 += 1;
        Ok([id; 16])
    }
}

fn setup() -> (TempDirectory, PathBuf) {
    let (directory, path, account_id) = setup_finance();
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
    let mut movements = AdapterSqliteMovementStore::new(open_database(&path).unwrap());
    create_entry(
        account_id,
        625,
        Context::Home,
        &mut FixedIds::new([0xC3; 16], [0xD4; 16]),
        &FixedClock,
        &mut movements,
    )
    .unwrap();
    drop(movements);
    let connection = open_database(&path).unwrap();
    let mut products = AdapterSqliteProductStore::new(connection);
    create_product(
        "Modelo atual".into(),
        "Descrição do produto".into(),
        1500,
        &mut Sequence(0x11),
        &FixedClock,
        &mut products,
    )
    .unwrap();
    drop(products);
    (directory, path)
}

fn add(
    store: &mut AdapterSqliteSaleStore,
    sale_id: Option<u8>,
    seed: u8,
) -> millani_artes::application::add_product_item::AddedProductItem {
    add_product_item(
        ProductItemInput {
            sale_id: sale_id.map(|id| [id; 16]),
            product_id: [0x11; 16],
            quantity: 1,
            unit_price_cents: 1500,
            discount: DiscountInput::AmountCents(0),
        },
        &mut Sequence(seed),
        &FixedClock,
        store,
    )
    .unwrap()
}

fn remove(
    store: &mut AdapterSqliteSaleStore,
    sale_id: [u8; 16],
    item_id: [u8; 16],
    event_id: u8,
) -> Result<(), SaleError> {
    remove_sale_item(
        RemoveSaleItemInput { sale_id, item_id },
        &mut Sequence(event_id),
        &FixedClock,
        store,
    )
}

fn rows(connection: &Connection, sql: &str) -> Vec<Vec<Value>> {
    let mut statement = connection.prepare(sql).unwrap();
    let columns = statement.column_count();
    statement
        .query_map([], |row| {
            (0..columns)
                .map(|column| row.get(column))
                .collect::<rusqlite::Result<Vec<_>>>()
        })
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap()
}

fn snapshot(connection: &Connection) -> Vec<Vec<Vec<Value>>> {
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
    .map(|table| {
        rows(
            connection,
            &format!("SELECT * FROM main.{table} ORDER BY 1"),
        )
    })
    .collect()
}

fn non_sale_snapshot(connection: &Connection) -> Vec<Vec<Vec<Value>>> {
    [
        "schema_migration",
        "accounts",
        "categories",
        "movements",
        "recurrences",
        "products",
        "customers",
    ]
    .iter()
    .map(|table| {
        rows(
            connection,
            &format!("SELECT * FROM main.{table} ORDER BY 1"),
        )
    })
    .collect()
}

fn item_positions(connection: &Connection, sale_id: [u8; 16]) -> Vec<(i64, Vec<u8>)> {
    let mut statement = connection
        .prepare("SELECT position,id FROM main.sale_items WHERE sale_id=?1 ORDER BY position")
        .unwrap();
    statement
        .query_map(params![sale_id.as_slice()], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap()
}

type AuditRemoval = (Vec<u8>, i64, String, String, Vec<u8>);

fn removal_events(connection: &Connection) -> Vec<AuditRemoval> {
    let mut statement = connection
        .prepare("SELECT event_id,occurred_at_ms,operation,entity,entity_id FROM main.audit_event WHERE operation='sale_item_remove' ORDER BY event_id")
        .unwrap();
    statement
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        })
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap()
}

fn health(connection: &Connection) {
    assert!(connection.is_autocommit());
    assert_eq!(schema_version(connection, 11).unwrap(), 11);
    assert_eq!(
        rows(connection, "PRAGMA main.integrity_check"),
        vec![vec![Value::Text("ok".into())]]
    );
    assert!(rows(connection, "PRAGMA main.foreign_key_check").is_empty());
}

fn load(store: &mut AdapterSqliteSaleStore, sale_id: [u8; 16]) -> Sale {
    read_sale(store, sale_id).unwrap().unwrap()
}

#[test]
fn sal014_removal_is_atomic_preserves_order_and_leaves_open_empty_sale() {
    let (_directory, path) = setup();
    let mut store = AdapterSqliteSaleStore::new(open_database(&path).unwrap());
    let first = add(&mut store, None, 0x31);
    let middle = add(&mut store, Some(0x31), 0x34);
    let last = add(&mut store, Some(0x31), 0x36);
    assert_eq!((first.sale_id, first.item.id), ([0x31; 16], [0x32; 16]));
    assert_eq!(middle.item.id, [0x34; 16]);
    assert_eq!(last.item.id, [0x36; 16]);
    drop(store);

    let observer = open_database(&path).unwrap();
    let unaffected_before = non_sale_snapshot(&observer);
    let audit_before: i64 = observer
        .query_row("SELECT count(*) FROM main.audit_event", [], |row| {
            row.get(0)
        })
        .unwrap();
    let mut store = AdapterSqliteSaleStore::new(open_database(&path).unwrap());
    assert_eq!(
        item_positions(&observer, [0x31; 16]),
        vec![
            (0, [0x32; 16].to_vec()),
            (1, [0x34; 16].to_vec()),
            (2, [0x36; 16].to_vec())
        ]
    );

    assert_eq!(remove(&mut store, [0x31; 16], [0x34; 16], 0x71), Ok(()));
    let sale = load(&mut store, [0x31; 16]);
    assert!(matches!(sale.status, SaleStatus::Open));
    assert_eq!(
        sale.items.iter().map(|item| item.id).collect::<Vec<_>>(),
        vec![[0x32; 16], [0x36; 16]]
    );
    assert_eq!(
        item_positions(&observer, [0x31; 16]),
        vec![(0, [0x32; 16].to_vec()), (2, [0x36; 16].to_vec())]
    );

    drop(store);
    let mut store = AdapterSqliteSaleStore::new(open_database(&path).unwrap());
    let reopened = load(&mut store, [0x31; 16]);
    assert_eq!(
        reopened
            .items
            .iter()
            .map(|item| item.id)
            .collect::<Vec<_>>(),
        vec![[0x32; 16], [0x36; 16]]
    );
    let appended = add(&mut store, Some(0x31), 0x38);
    assert_eq!(appended.item.id, [0x38; 16]);
    assert_eq!(
        item_positions(&observer, [0x31; 16]),
        vec![
            (0, [0x32; 16].to_vec()),
            (2, [0x36; 16].to_vec()),
            (3, [0x38; 16].to_vec())
        ]
    );

    assert_eq!(remove(&mut store, [0x31; 16], [0x32; 16], 0x72), Ok(()));
    assert_eq!(
        load(&mut store, [0x31; 16])
            .items
            .iter()
            .map(|item| item.id)
            .collect::<Vec<_>>(),
        vec![[0x36; 16], [0x38; 16]]
    );
    assert_eq!(remove(&mut store, [0x31; 16], [0x38; 16], 0x73), Ok(()));
    assert_eq!(
        load(&mut store, [0x31; 16])
            .items
            .iter()
            .map(|item| item.id)
            .collect::<Vec<_>>(),
        vec![[0x36; 16]]
    );
    assert_eq!(remove(&mut store, [0x31; 16], [0x36; 16], 0x74), Ok(()));
    let empty = load(&mut store, [0x31; 16]);
    assert!(matches!(empty.status, SaleStatus::Open));
    assert!(empty.items.is_empty());
    assert_eq!(
        observer
            .query_row(
                "SELECT status FROM main.sales WHERE id=?1",
                params![[0x31_u8; 16].as_slice()],
                |row| row.get::<_, String>(0),
            )
            .unwrap(),
        "open"
    );
    assert_eq!(non_sale_snapshot(&observer), unaffected_before);
    assert_eq!(
        removal_events(&observer),
        vec![
            (
                [0x71; 16].to_vec(),
                1234,
                "sale_item_remove".into(),
                "sale_item".into(),
                [0x34; 16].to_vec()
            ),
            (
                [0x72; 16].to_vec(),
                1234,
                "sale_item_remove".into(),
                "sale_item".into(),
                [0x32; 16].to_vec()
            ),
            (
                [0x73; 16].to_vec(),
                1234,
                "sale_item_remove".into(),
                "sale_item".into(),
                [0x38; 16].to_vec()
            ),
            (
                [0x74; 16].to_vec(),
                1234,
                "sale_item_remove".into(),
                "sale_item".into(),
                [0x36; 16].to_vec()
            ),
        ]
    );
    let audit_after: i64 = observer
        .query_row("SELECT count(*) FROM main.audit_event", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(audit_after, audit_before + 5);
    assert_eq!(
        observer
            .query_row("SELECT count(*) FROM main.accounts", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        observer
            .query_row("SELECT count(*) FROM main.movements", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
    health(&observer);
}

#[test]
fn sal014_rejections_do_not_write_and_keep_the_adapter_usable() {
    let (_directory, path) = setup();
    let mut creator = AdapterSqliteSaleStore::new(open_database(&path).unwrap());
    let first = add(&mut creator, None, 0x31);
    let second = add(&mut creator, None, 0x41);
    let finalized = add(&mut creator, None, 0x51);
    open_database(&path)
        .unwrap()
        .execute(
            "UPDATE main.sales SET status='finalized' WHERE id=?1",
            params![[0x51_u8; 16].as_slice()],
        )
        .unwrap();
    drop(creator);

    let mut store = AdapterSqliteSaleStore::new(open_database(&path).unwrap());
    let observer = open_database(&path).unwrap();
    for (sale_id, item_id, event_id, expected) in [
        ([0x70; 16], first.item.id, 0x81, SaleError::SaleNotFound),
        ([0x31; 16], [0x7F; 16], 0x82, SaleError::SaleItemNotFound),
        ([0x41; 16], first.item.id, 0x83, SaleError::SaleItemNotFound),
        ([0x51; 16], finalized.item.id, 0x84, SaleError::SaleNotOpen),
    ] {
        let before = snapshot(&observer);
        assert_eq!(
            remove(&mut store, sale_id, item_id, event_id),
            Err(expected)
        );
        assert_eq!(snapshot(&observer), before);
        assert!(read_sale(&mut store, [0x31; 16]).unwrap().is_some());
    }
    assert_eq!(
        remove(&mut store, first.sale_id, first.item.id, 0x85),
        Ok(())
    );
    assert!(load(&mut store, first.sale_id).items.is_empty());
    assert_eq!(load(&mut store, second.sale_id).items[0].id, second.item.id);
    assert_eq!(removal_events(&observer).len(), 1);
    health(&observer);
}

#[test]
fn sal014_sql_and_audit_failures_rollback_poison_and_allow_fresh_recovery() {
    for case in [
        "delete-abort",
        "delete-ignore",
        "audit-duplicate",
        "audit-ignore",
    ] {
        let (_directory, path) = setup();
        let mut creator = AdapterSqliteSaleStore::new(open_database(&path).unwrap());
        let sale = add(&mut creator, None, 0x31);
        drop(creator);
        let preparation = open_database(&path).unwrap();
        match case {
            "delete-abort" => preparation
                .execute_batch("CREATE TRIGGER block_sale_item_delete BEFORE DELETE ON sale_items BEGIN SELECT RAISE(ABORT,'injected'); END;")
                .unwrap(),
            "delete-ignore" => preparation
                .execute_batch("CREATE TRIGGER ignore_sale_item_delete BEFORE DELETE ON sale_items BEGIN SELECT RAISE(IGNORE); END;")
                .unwrap(),
            "audit-duplicate" => {
                preparation
                    .execute(
                        "INSERT INTO main.audit_event VALUES(?1,0,'seed_event','sale_item',?2)",
                        params![[0x91_u8; 16].as_slice(), sale.item.id.as_slice()],
                    )
                    .unwrap();
            }
            "audit-ignore" => preparation
                .execute_batch("CREATE TRIGGER ignore_sale_item_audit BEFORE INSERT ON audit_event BEGIN SELECT RAISE(IGNORE); END;")
                .unwrap(),
            _ => unreachable!(),
        }
        drop(preparation);

        let observer = open_database(&path).unwrap();
        let before = snapshot(&observer);
        let event_id = if case == "audit-duplicate" {
            0x91
        } else {
            0x92
        };
        let mut store = AdapterSqliteSaleStore::new(open_database(&path).unwrap());
        assert_eq!(
            remove(&mut store, sale.sale_id, sale.item.id, event_id),
            Err(SaleError::StoreFailed),
            "{case}"
        );
        assert_eq!(snapshot(&observer), before, "{case} partially committed");
        assert_eq!(
            remove(&mut store, sale.sale_id, sale.item.id, 0xA0),
            Err(SaleError::StoreFailed),
            "{case} reused a poisoned removal connection"
        );
        assert_eq!(
            read_sale(&mut store, sale.sale_id).err(),
            Some(SaleError::StoreFailed),
            "{case} reused its failed connection"
        );
        drop(store);
        drop(observer);

        if case != "audit-duplicate" {
            let repair = open_database(&path).unwrap();
            let trigger = match case {
                "delete-abort" => "block_sale_item_delete",
                "delete-ignore" => "ignore_sale_item_delete",
                _ => "ignore_sale_item_audit",
            };
            repair
                .execute_batch(&format!("DROP TRIGGER main.{trigger}"))
                .unwrap();
        }
        let mut fresh = AdapterSqliteSaleStore::new(open_database(&path).unwrap());
        assert_eq!(
            remove(&mut fresh, sale.sale_id, sale.item.id, 0xA1),
            Ok(()),
            "{case} fresh adapter did not recover"
        );
        assert!(load(&mut fresh, sale.sale_id).items.is_empty());
    }
}

#[test]
fn sal014_query_only_and_writer_lock_fail_without_partial_writes() {
    let (_directory, path) = setup();
    let mut creator = AdapterSqliteSaleStore::new(open_database(&path).unwrap());
    let sale = add(&mut creator, None, 0x31);
    drop(creator);
    let observer = open_database(&path).unwrap();
    let before = snapshot(&observer);

    let query_only_connection = open_database(&path).unwrap();
    query_only_connection
        .pragma_update(None, "query_only", true)
        .unwrap();
    let mut query_only = AdapterSqliteSaleStore::new(query_only_connection);
    assert_eq!(
        remove(&mut query_only, sale.sale_id, sale.item.id, 0x92),
        Err(SaleError::StoreFailed)
    );
    assert_eq!(snapshot(&observer), before);
    assert_eq!(
        remove(&mut query_only, sale.sale_id, sale.item.id, 0xA0),
        Err(SaleError::StoreFailed)
    );
    assert_eq!(
        read_sale(&mut query_only, sale.sale_id).err(),
        Some(SaleError::StoreFailed)
    );
    drop(query_only);

    let lock = open_database(&path).unwrap();
    lock.execute_batch("BEGIN IMMEDIATE").unwrap();
    let locked_connection = open_database(&path).unwrap();
    locked_connection.busy_timeout(Duration::ZERO).unwrap();
    let mut locked = AdapterSqliteSaleStore::new(locked_connection);
    assert_eq!(
        remove(&mut locked, sale.sale_id, sale.item.id, 0x93),
        Err(SaleError::StoreFailed)
    );
    assert_eq!(snapshot(&observer), before);
    assert_eq!(
        remove(&mut locked, sale.sale_id, sale.item.id, 0xA0),
        Err(SaleError::StoreFailed)
    );
    assert_eq!(
        read_sale(&mut locked, sale.sale_id).err(),
        Some(SaleError::StoreFailed)
    );
    drop(locked);
    lock.execute_batch("ROLLBACK").unwrap();
    drop(lock);

    let mut fresh = AdapterSqliteSaleStore::new(open_database(&path).unwrap());
    assert_eq!(remove(&mut fresh, sale.sale_id, sale.item.id, 0x94), Ok(()));
    assert!(load(&mut fresh, sale.sale_id).items.is_empty());
    health(&observer);
}

#[test]
fn sal014_uses_main_tables_when_temp_names_shadow_sales_tables() {
    let (_directory, path) = setup();
    let mut creator = AdapterSqliteSaleStore::new(open_database(&path).unwrap());
    let sale = add(&mut creator, None, 0x31);
    drop(creator);

    let connection = open_database(&path).unwrap();
    connection
        .execute_batch(
            "CREATE TEMP TABLE sales(id BLOB,status TEXT);
             CREATE TEMP TABLE sale_items(id BLOB,sale_id BLOB);
             CREATE TEMP TABLE audit_event(event_id BLOB,occurred_at_ms INTEGER,operation TEXT,entity TEXT,entity_id BLOB);",
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO temp.sales VALUES(?1,'finalized')",
            params![sale.sale_id.as_slice()],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO temp.sale_items VALUES(?1,?2)",
            params![sale.item.id.as_slice(), sale.sale_id.as_slice()],
        )
        .unwrap();
    let mut store = AdapterSqliteSaleStore::new(connection);
    assert_eq!(remove(&mut store, sale.sale_id, sale.item.id, 0x95), Ok(()));
    let remaining = load(&mut store, sale.sale_id);
    assert!(matches!(remaining.status, SaleStatus::Open));
    assert!(remaining.items.is_empty());
    assert_eq!(removal_events(&open_database(&path).unwrap()).len(), 1);
}
