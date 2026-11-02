use super::TempDirectory;
use millani_artes::application::record_sale_return::{
    record_sale_return, RecordSaleReturnError, RecordSaleReturnWrite, SaleReturnRecordingStore,
};
use millani_artes::database::{
    execute_versioned_migration, open_database, ACCOUNTS_MIGRATION_SQL, AUDIT_EVENT_MIGRATION_SQL,
    CATEGORIES_MIGRATION_SQL, CUSTOMERS_MIGRATION_SQL, MOVEMENTS_MIGRATION_SQL,
    MOVEMENT_CATEGORIES_MIGRATION_SQL, PRODUCTS_MIGRATION_SQL, RECEIPTS_MIGRATION_SQL,
    RECEIVABLES_MIGRATION_SQL, RECURRENCES_MIGRATION_SQL, SALES_MIGRATION_SQL,
    SALE_ITEMS_MIGRATION_SQL, SALE_RETURN_ITEMS_MIGRATION_SQL, SALE_RETURN_ITEMS_MIGRATION_VERSION,
};
use millani_artes::domain::ports::{Clock, UuidGenerator};
use millani_artes::infrastructure::sale_returns::AdapterSqliteSaleReturnStore;
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const SALE_ID: [u8; 16] = [0x31; 16];
const SALE_ITEM_ID: [u8; 16] = [0x32; 16];
const ACCOUNT_ID: [u8; 16] = [0x33; 16];
const OCCURRED_AT_MS: u64 = 1_800_000_000_123;

struct Sequence(u8);

impl UuidGenerator for Sequence {
    type Error = ();

    fn generate(&mut self) -> Result<[u8; 16], Self::Error> {
        let current = self.0;
        self.0 = self.0.checked_add(1).ok_or(())?;
        Ok([current; 16])
    }
}

struct FixedClock;

impl Clock for FixedClock {
    fn now(&self) -> SystemTime {
        UNIX_EPOCH + Duration::from_millis(OCCURRED_AT_MS)
    }
}

fn setup(name: &str, sold_quantity: i64) -> (TempDirectory, PathBuf) {
    let directory = TempDirectory::new();
    let path = directory.path().join(name);
    let mut connection = execute_versioned_migration(open_database(&path).unwrap(), 1, "").unwrap();
    for (version, sql) in [
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
        (12, RECEIVABLES_MIGRATION_SQL),
        (13, RECEIPTS_MIGRATION_SQL),
        (
            SALE_RETURN_ITEMS_MIGRATION_VERSION,
            SALE_RETURN_ITEMS_MIGRATION_SQL,
        ),
    ] {
        connection = execute_versioned_migration(connection, version, sql).unwrap();
    }
    connection
        .execute(
            "INSERT INTO main.accounts(id,name,account_type,initial_balance_cents) VALUES(?1,'Conta sintética','cash',900)",
            [ACCOUNT_ID.as_slice()],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO main.sales(id,status) VALUES(?1,'finalized')",
            [SALE_ID.as_slice()],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO main.sale_items(id,sale_id,position,product_id,description,quantity,unit_price_cents,discount_cents,total_cents) VALUES(?1,?2,0,NULL,'Item sintético',?3,100,0,?3*100)",
            params![SALE_ITEM_ID.as_slice(), SALE_ID.as_slice(), sold_quantity],
        )
        .unwrap();
    drop(connection);
    (directory, path)
}

fn record(
    path: &Path,
    item_id: [u8; 16],
    quantity: i64,
    seed: u8,
) -> Result<millani_artes::domain::SaleReturnItem, RecordSaleReturnError> {
    let mut store = AdapterSqliteSaleReturnStore::new(open_database(path).unwrap());
    record_sale_return(
        item_id,
        quantity,
        &mut Sequence(seed),
        &FixedClock,
        &mut store,
    )
}

fn count(connection: &Connection, table: &str) -> i64 {
    connection
        .query_row(&format!("SELECT count(*) FROM main.{table}"), [], |row| {
            row.get(0)
        })
        .unwrap()
}

fn audit_count(connection: &Connection) -> i64 {
    connection
        .query_row(
            "SELECT count(*) FROM main.audit_event WHERE operation='sale_return_item_create'",
            [],
            |row| row.get(0),
        )
        .unwrap()
}

fn matching_audit_count(connection: &Connection) -> i64 {
    connection
        .query_row(
            "SELECT count(*) FROM main.audit_event AS event JOIN main.sale_return_items AS returned ON event.entity_id=returned.id WHERE event.operation='sale_return_item_create' AND event.entity='sale_return_item'",
            [],
            |row| row.get(0),
        )
        .unwrap()
}

#[test]
fn ret005_persists_successive_partial_returns_and_rejects_past_the_remaining_quantity() {
    let (_directory, path) = setup("ret005-partials.sqlite", 3);

    for seed in [0x41, 0x51, 0x61] {
        let returned = record(&path, SALE_ITEM_ID, 1, seed).unwrap();
        assert_eq!(returned.sale_item_id, SALE_ITEM_ID);
        assert_eq!(returned.quantity, 1);
    }

    let reopened = open_database(&path).unwrap();
    assert_eq!(count(&reopened, "sale_return_items"), 3);
    assert_eq!(audit_count(&reopened), 3);
    assert_eq!(matching_audit_count(&reopened), 3);
    let account_before: (Vec<u8>, String, String, i64) = reopened
        .query_row(
            "SELECT id,name,account_type,initial_balance_cents FROM main.accounts",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap();
    assert_eq!(account_before.3, 900);
    assert_eq!(count(&reopened, "movements"), 0);
    assert_eq!(count(&reopened, "receivables"), 0);
    assert_eq!(count(&reopened, "receipts"), 0);
    drop(reopened);

    assert!(matches!(
        record(&path, SALE_ITEM_ID, 1, 0x71),
        Err(RecordSaleReturnError::QuantityExceedsAvailable)
    ));
    let reopened = open_database(&path).unwrap();
    assert_eq!(count(&reopened, "sale_return_items"), 3);
    assert_eq!(audit_count(&reopened), 3);
    assert_eq!(matching_audit_count(&reopened), 3);
    assert_eq!(
        reopened
            .query_row(
                "SELECT id,name,account_type,initial_balance_cents FROM main.accounts",
                [],
                |row| Ok((
                    row.get::<_, Vec<u8>>(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?
                )),
            )
            .unwrap(),
        account_before
    );
    assert_eq!(count(&reopened, "movements"), 0);
    assert_eq!(count(&reopened, "receivables"), 0);
    assert_eq!(count(&reopened, "receipts"), 0);
}

#[test]
fn ret005_rejects_nonpositive_quantity_and_unknown_item_without_writes() {
    let (_directory, path) = setup("ret005-invalid.sqlite", 2);

    assert!(matches!(
        record(&path, SALE_ITEM_ID, 0, 0x41),
        Err(RecordSaleReturnError::InvalidInput)
    ));
    assert!(matches!(
        record(&path, [0xff; 16], 1, 0x51),
        Err(RecordSaleReturnError::SaleItemNotFound)
    ));

    let reopened = open_database(&path).unwrap();
    assert_eq!(count(&reopened, "sale_return_items"), 0);
    assert_eq!(audit_count(&reopened), 0);
}

#[test]
fn ret005_store_rejects_nonpositive_write_without_consuming_its_connection() {
    let (_directory, path) = setup("ret005-store-invalid.sqlite", 2);
    let mut store = AdapterSqliteSaleReturnStore::new(open_database(&path).unwrap());

    assert!(matches!(
        store.persist_return_with_audit(RecordSaleReturnWrite {
            return_id: [0x41; 16],
            sale_item_id: SALE_ITEM_ID,
            quantity: 0,
            audit_event_id: [0x42; 16],
            occurred_at_ms: OCCURRED_AT_MS as i64,
        }),
        Err(RecordSaleReturnError::InvalidInput)
    ));
    assert!(record_sale_return(
        SALE_ITEM_ID,
        1,
        &mut Sequence(0x51),
        &FixedClock,
        &mut store,
    )
    .is_ok());

    let reopened = open_database(&path).unwrap();
    assert_eq!(count(&reopened, "sale_return_items"), 1);
    assert_eq!(audit_count(&reopened), 1);
}

#[test]
fn ret005_rolls_back_return_when_audit_event_insert_fails() {
    let (_directory, path) = setup("ret005-audit-failure.sqlite", 2);
    let connection = open_database(&path).unwrap();
    connection
        .execute_batch(
            "CREATE TRIGGER main.reject_sale_return_audit BEFORE INSERT ON audit_event WHEN NEW.operation='sale_return_item_create' BEGIN SELECT RAISE(ABORT,'synthetic audit failure'); END;",
        )
        .unwrap();
    drop(connection);

    assert!(matches!(
        record(&path, SALE_ITEM_ID, 1, 0x41),
        Err(RecordSaleReturnError::StoreFailed)
    ));

    let reopened = open_database(&path).unwrap();
    assert_eq!(count(&reopened, "sale_return_items"), 0);
    assert_eq!(audit_count(&reopened), 0);
}

#[test]
fn ret005_rejects_silently_ignored_return_insert_without_audit() {
    let (_directory, path) = setup("ret005-ignored-return.sqlite", 2);
    let connection = open_database(&path).unwrap();
    connection
        .execute_batch(
            "CREATE TRIGGER main.ignore_sale_return BEFORE INSERT ON sale_return_items BEGIN SELECT RAISE(IGNORE); END;",
        )
        .unwrap();
    drop(connection);

    assert!(matches!(
        record(&path, SALE_ITEM_ID, 1, 0x41),
        Err(RecordSaleReturnError::StoreFailed)
    ));

    let reopened = open_database(&path).unwrap();
    assert_eq!(count(&reopened, "sale_return_items"), 0);
    assert_eq!(audit_count(&reopened), 0);
}

#[test]
fn ret005_concurrent_writers_do_not_return_more_than_the_last_available_unit() {
    let (_directory, path) = setup("ret005-concurrent.sqlite", 1);
    let barrier = Arc::new(Barrier::new(3));
    let workers = [0x41, 0x51].map(|seed| {
        let path = path.clone();
        let barrier = Arc::clone(&barrier);
        thread::spawn(move || {
            let mut store = AdapterSqliteSaleReturnStore::new(open_database(&path).unwrap());
            let mut ids = Sequence(seed);
            barrier.wait();
            record_sale_return(SALE_ITEM_ID, 1, &mut ids, &FixedClock, &mut store)
        })
    });
    barrier.wait();
    let results: Vec<_> = workers
        .into_iter()
        .map(|worker| worker.join().unwrap())
        .collect();
    assert!(results.iter().any(Result::is_ok));

    let reopened = open_database(&path).unwrap();
    assert_eq!(count(&reopened, "sale_return_items"), 1);
    assert_eq!(audit_count(&reopened), 1);
    assert_eq!(matching_audit_count(&reopened), 1);
}
