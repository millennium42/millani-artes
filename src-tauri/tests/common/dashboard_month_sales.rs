use super::TempDirectory;
use millani_artes::application::list_month_sales::{
    list_month_sale_cents, ListMonthSalesError, MonthSaleReadStore,
};
use millani_artes::database::{
    execute_sale_cancellations_migration, execute_sale_return_financials_migration,
    execute_sale_return_refunds_migration, execute_versioned_migration, open_database,
    schema_version, ACCOUNTS_MIGRATION_SQL, AUDIT_EVENT_MIGRATION_SQL, CATEGORIES_MIGRATION_SQL,
    CUSTOMERS_MIGRATION_SQL, MOVEMENTS_MIGRATION_SQL, MOVEMENT_CATEGORIES_MIGRATION_SQL,
    PRODUCTS_MIGRATION_SQL, RECEIPTS_MIGRATION_SQL, RECEIVABLES_MIGRATION_SQL,
    RECURRENCES_MIGRATION_SQL, SALES_MIGRATION_SQL, SALE_ITEMS_MIGRATION_SQL,
    SALE_RETURN_ITEMS_MIGRATION_SQL,
};
use millani_artes::domain::brasilia_month_window_ms;
use millani_artes::domain::ports::Clock;
use millani_artes::infrastructure::sales::{AdapterSqliteSaleStore, MonthSaleReadError};
use rusqlite::{params, types::Value, Connection};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const JANUARY_2024_START_MS: i64 = 1_704_078_000_000;
const ACCOUNT_ID: [u8; 16] = [0x70; 16];
const CUSTOMER_ID: [u8; 16] = [0x71; 16];

struct MillisClock(i64);

impl Clock for MillisClock {
    fn now(&self) -> SystemTime {
        UNIX_EPOCH + Duration::from_millis(self.0 as u64)
    }
}

fn setup() -> (TempDirectory, PathBuf) {
    let directory = TempDirectory::new();
    let path = directory.path().join("dsh005.sqlite");
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
        (14, SALE_RETURN_ITEMS_MIGRATION_SQL),
    ] {
        connection = execute_versioned_migration(connection, version, sql).unwrap();
    }
    connection = execute_sale_return_financials_migration(connection).unwrap();
    connection = execute_sale_return_refunds_migration(connection).unwrap();
    connection = execute_sale_cancellations_migration(connection).unwrap();
    connection
        .execute(
            "INSERT INTO main.accounts(id,name,account_type,initial_balance_cents) VALUES(?1,'synthetic account','cash',0)",
            params![ACCOUNT_ID.as_slice()],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO main.customers(id,name,phone) VALUES(?1,'synthetic customer',NULL)",
            params![CUSTOMER_ID.as_slice()],
        )
        .unwrap();
    drop(connection);
    (directory, path)
}

fn add_sale(
    path: &std::path::Path,
    sale_id: u8,
    status: &str,
    total_cents: i64,
    finalized_at_ms: Option<i64>,
) {
    let connection = open_database(path).unwrap();
    connection
        .execute(
            "INSERT INTO main.sales(id,status) VALUES(?1,?2)",
            params![[sale_id; 16].as_slice(), status],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO main.sale_items(id,sale_id,position,product_id,description,quantity,unit_price_cents,discount_cents,total_cents) VALUES(?1,?2,0,NULL,'synthetic item',1,?3,0,?3)",
            params![[sale_id + 0x20; 16].as_slice(), [sale_id; 16].as_slice(), total_cents],
        )
        .unwrap();
    if let Some(occurred_at_ms) = finalized_at_ms {
        connection
            .execute(
                "INSERT INTO main.audit_event(event_id,occurred_at_ms,operation,entity,entity_id) VALUES(?1,?2,'sale_finalize','sale',?3)",
                params![[sale_id + 0x40; 16].as_slice(), occurred_at_ms, [sale_id; 16].as_slice()],
            )
            .unwrap();
    }
}

fn assert_static_store_error(result: Result<Vec<i64>, MonthSaleReadError>) {
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("expected a static read error"),
    };
    assert_eq!(format!("{error:?}"), "MonthSaleReadError");
    assert_eq!(format!("{error}"), "monthly sale history could not be read");
    assert!(std::error::Error::source(&error).is_none());
}

fn assert_app_error(result: Result<i64, ListMonthSalesError>) {
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("expected a static dashboard error"),
    };
    assert_eq!(format!("{error:?}"), "ListMonthSalesError");
    assert_eq!(format!("{error}"), "month's sales could not be read");
    assert!(std::error::Error::source(&error).is_none());
}

fn table_rows(connection: &Connection, table: &str) -> Vec<Vec<Value>> {
    let mut statement = connection
        .prepare(&format!("SELECT * FROM main.{table} ORDER BY 1"))
        .unwrap();
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
        "customers",
        "sales",
        "sale_items",
        "audit_event",
        "receivables",
        "receipts",
        "sale_return_items",
        "sale_cancellations",
    ]
    .into_iter()
    .map(|table| table_rows(connection, table))
    .collect()
}

fn assert_healthy(connection: &Connection) {
    assert!(connection.is_autocommit());
    assert_eq!(schema_version(connection, 17).unwrap(), 17);
    assert_eq!(
        connection
            .query_row("PRAGMA main.integrity_check", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    let foreign_key_violations: i64 = connection
        .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(foreign_key_violations, 0);
}

#[test]
fn dashboard_month_sales_uses_original_finalization_and_is_read_only_after_reopen() {
    let (_directory, path) = setup();
    let (start_ms, end_ms) = brasilia_month_window_ms(JANUARY_2024_START_MS).unwrap();

    // Fixtures are synthetic persisted sale/audit snapshots, including later return and cancellation markers.
    add_sale(&path, 0x11, "finalized", 100, Some(start_ms));
    add_sale(&path, 0x12, "finalized", 500, Some(end_ms - 1));
    add_sale(&path, 0x13, "finalized", 1_000, Some(start_ms - 1));
    add_sale(&path, 0x14, "finalized", 2_000, Some(end_ms));
    add_sale(&path, 0x15, "open", 3_000, Some(start_ms + 5));
    add_sale(&path, 0x16, "finalized", 4_000, None);
    add_sale(&path, 0x17, "finalized", 700, Some(start_ms + 6));
    add_sale(&path, 0x18, "finalized", 800, Some(start_ms + 7));
    add_sale(&path, 0x19, "finalized", 0, Some(start_ms + 8));

    let connection = open_database(&path).unwrap();
    connection
        .execute(
            "INSERT INTO main.receivables(id,sale_id,amount_cents,customer_id,due_at_ms) VALUES(?1,?2,500,?3,?4)",
            params![[0x90_u8; 16].as_slice(), [0x12_u8; 16].as_slice(), CUSTOMER_ID.as_slice(), end_ms],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO main.receipts(id,receivable_id,account_id,amount_cents,context) VALUES(?1,?2,?3,200,'home')",
            params![[0x91_u8; 16].as_slice(), [0x90_u8; 16].as_slice(), ACCOUNT_ID.as_slice()],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO main.sale_return_items(id,sale_item_id,quantity,return_amount_cents,debt_reduction_cents) VALUES(?1,?2,1,800,0)",
            params![[0x92_u8; 16].as_slice(), [0x38_u8; 16].as_slice()],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO main.sale_cancellations(sale_id,cancelled_at_ms) VALUES(?1,?2)",
            params![[0x17_u8; 16].as_slice(), end_ms + 1],
        )
        .unwrap();
    drop(connection);

    let reader = open_database(&path).unwrap();
    assert_healthy(&reader);
    reader
        .execute_batch(
            "CREATE TEMP TABLE sales(id BLOB,status TEXT);
             CREATE TEMP TABLE sale_items(id BLOB,sale_id BLOB,position INTEGER,total_cents INTEGER);
             CREATE TEMP TABLE audit_event(event_id BLOB,occurred_at_ms INTEGER,operation TEXT,entity TEXT,entity_id BLOB);
             INSERT INTO temp.sales VALUES(X'99999999999999999999999999999999','finalized');
             INSERT INTO temp.sale_items VALUES(X'98989898989898989898989898989898',X'99999999999999999999999999999999',0,999999);
             INSERT INTO temp.audit_event VALUES(X'97979797979797979797979797979797',1704078000000,'sale_finalize','sale',X'99999999999999999999999999999999');",
        )
        .unwrap();
    reader.pragma_update(None, "query_only", true).unwrap();
    let before = snapshot(&reader);
    let store = AdapterSqliteSaleStore::new(reader);
    let result = list_month_sale_cents(&MillisClock(JANUARY_2024_START_MS), &store).unwrap();
    drop(store);

    assert_eq!(result, 2_100);
    assert_eq!(snapshot(&open_database(&path).unwrap()), before);
}

#[test]
fn duplicate_finalization_event_fails_closed_without_partial_total() {
    let (_directory, path) = setup();
    let (start_ms, end_ms) = brasilia_month_window_ms(JANUARY_2024_START_MS).unwrap();
    add_sale(&path, 0x21, "finalized", 100, Some(start_ms));
    add_sale(&path, 0x22, "finalized", 200, Some(start_ms + 1));
    let connection = open_database(&path).unwrap();
    connection
        .execute(
            "INSERT INTO main.audit_event(event_id,occurred_at_ms,operation,entity,entity_id) VALUES(?1,?2,'sale_finalize','sale',?3)",
            params![[0x82_u8; 16].as_slice(), end_ms, [0x21_u8; 16].as_slice()],
        )
        .unwrap();
    drop(connection);

    let store = AdapterSqliteSaleStore::new(open_database(&path).unwrap());
    assert_static_store_error(
        store.read_finalized_sale_item_amounts_created_between(start_ms, end_ms),
    );
    assert_app_error(list_month_sale_cents(
        &MillisClock(JANUARY_2024_START_MS),
        &store,
    ));
}

#[test]
fn duplicate_events_outside_the_projected_month_do_not_change_its_total() {
    let (_directory, path) = setup();
    let (start_ms, end_ms) = brasilia_month_window_ms(JANUARY_2024_START_MS).unwrap();
    add_sale(&path, 0x31, "finalized", 100, Some(start_ms));
    add_sale(&path, 0x32, "finalized", 200, Some(start_ms - 10));
    open_database(&path)
        .unwrap()
        .execute(
            "INSERT INTO main.audit_event(event_id,occurred_at_ms,operation,entity,entity_id) VALUES(?1,?2,'sale_finalize','sale',?3)",
            params![[0x83_u8; 16].as_slice(), start_ms - 9, [0x32_u8; 16].as_slice()],
        )
        .unwrap();

    let store = AdapterSqliteSaleStore::new(open_database(&path).unwrap());
    assert_eq!(
        store
            .read_finalized_sale_item_amounts_created_between(start_ms, end_ms)
            .unwrap(),
        [100]
    );
}

#[test]
fn missing_schema_errors_are_static_at_the_adapter_boundary() {
    let connection = Connection::open_in_memory().unwrap();
    let store = AdapterSqliteSaleStore::new(connection);

    assert_static_store_error(store.read_finalized_sale_item_amounts_created_between(0, 1));
    assert_app_error(list_month_sale_cents(
        &MillisClock(JANUARY_2024_START_MS),
        &store,
    ));
}
