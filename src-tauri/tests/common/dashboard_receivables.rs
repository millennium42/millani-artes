use super::TempDirectory;
use millani_artes::application::list_receivables::{
    list_receivable_due_cents, ListReceivableDueError, ReceivableDueReadStore,
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
use millani_artes::infrastructure::receivable_due::{
    AdapterSqliteReceivableDueReadStore, ReceivableDueReadError,
};
use rusqlite::{params, types::Value, Connection};

const CUSTOMER_ID: [u8; 16] = [0xa1; 16];
const ACCOUNT_ID: [u8; 16] = [0xa2; 16];

fn setup() -> (TempDirectory, std::path::PathBuf) {
    let directory = TempDirectory::new();
    let path = directory.path().join("dsh006.sqlite");
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
    id: u8,
    status: &str,
    item_amounts: &[i64],
    principal_cents: Option<i64>,
) {
    let connection = open_database(path).unwrap();
    connection
        .execute(
            "INSERT INTO main.sales(id,status) VALUES(?1,?2)",
            params![[id; 16].as_slice(), status],
        )
        .unwrap();
    for (index, amount_cents) in item_amounts.iter().enumerate() {
        let item_id = [id.wrapping_add(0x40 + index as u8); 16];
        connection
            .execute(
                "INSERT INTO main.sale_items(id,sale_id,position,product_id,description,quantity,unit_price_cents,discount_cents,total_cents) VALUES(?1,?2,?3,NULL,'synthetic item',1,?4,0,?4)",
                params![item_id.as_slice(), [id; 16].as_slice(), index as i64, amount_cents],
            )
            .unwrap();
    }
    if let Some(principal_cents) = principal_cents {
        connection
            .execute(
                "INSERT INTO main.receivables(id,sale_id,amount_cents,customer_id,due_at_ms) VALUES(?1,?2,?3,?4,0)",
                params![[id.wrapping_add(0x20); 16].as_slice(), [id; 16].as_slice(), principal_cents, CUSTOMER_ID.as_slice()],
            )
            .unwrap();
    }
}

fn add_receipt(path: &std::path::Path, id: u8, sale_id: u8, amount_cents: i64) {
    open_database(path)
        .unwrap()
        .execute(
            "INSERT INTO main.receipts(id,receivable_id,account_id,amount_cents,context) VALUES(?1,?2,?3,?4,'home')",
            params![[id; 16].as_slice(), [sale_id.wrapping_add(0x20); 16].as_slice(), ACCOUNT_ID.as_slice(), amount_cents],
        )
        .unwrap();
}

fn add_return(
    path: &std::path::Path,
    return_id: u8,
    item_id: u8,
    amount_cents: i64,
    debt_reduction_cents: i64,
) {
    open_database(path)
        .unwrap()
        .execute(
            "INSERT INTO main.sale_return_items(id,sale_item_id,quantity,return_amount_cents,debt_reduction_cents) VALUES(?1,?2,1,?3,?4)",
            params![[return_id; 16].as_slice(), [item_id; 16].as_slice(), amount_cents, debt_reduction_cents],
        )
        .unwrap();
}

fn add_refund(path: &std::path::Path, movement_id: u8, return_id: u8, amount_cents: i64) {
    let connection = open_database(path).unwrap();
    connection
        .execute(
            "INSERT INTO main.movements(id,kind,amount_cents,context,account_id) VALUES(?1,'exit',?2,'home',?3)",
            params![[movement_id; 16].as_slice(), amount_cents, ACCOUNT_ID.as_slice()],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO main.sale_return_refunds(movement_id,sale_return_item_id) VALUES(?1,?2)",
            params![[movement_id; 16].as_slice(), [return_id; 16].as_slice()],
        )
        .unwrap();
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
        "receivables",
        "receipts",
        "sale_return_items",
        "movements",
        "sale_return_refunds",
        "sale_cancellations",
    ]
    .into_iter()
    .map(|table| table_rows(connection, table))
    .collect()
}

fn assert_app_error(result: Result<i64, ListReceivableDueError>) {
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("expected a static dashboard error"),
    };
    assert_eq!(format!("{error:?}"), "ListReceivableDueError");
    assert_eq!(format!("{error}"), "receivable total could not be read");
    assert!(std::error::Error::source(&error).is_none());
}

fn assert_store_error(result: Result<Vec<i64>, ReceivableDueReadError>) {
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("expected a static adapter error"),
    };
    assert_eq!(format!("{error:?}"), "ReceivableDueReadError");
    assert_eq!(
        format!("{error}"),
        "receivable due history could not be read"
    );
    assert!(std::error::Error::source(&error).is_none());
}

#[test]
fn dashboard_receivables_projects_unique_history_after_reopen_read_only() {
    let (_directory, path) = setup();
    add_sale(
        &path,
        0x11,
        "finalized",
        &[1_000, 1_000, 3_000],
        Some(5_000),
    );
    add_receipt(&path, 0x71, 0x11, 1_000);
    add_receipt(&path, 0x72, 0x11, 500);
    add_return(&path, 0x81, 0x51, 1_000, 1_000);
    add_return(&path, 0x82, 0x52, 1_000, 1_000);

    add_sale(&path, 0x21, "finalized", &[4_000], Some(4_000));
    add_receipt(&path, 0x73, 0x21, 4_000);
    add_return(&path, 0x83, 0x61, 1_000, 0);
    add_refund(&path, 0x91, 0x83, 1_000);

    add_sale(&path, 0x22, "finalized", &[2_000], Some(2_000));
    add_receipt(&path, 0x74, 0x22, 2_000);
    add_return(&path, 0x84, 0x62, 2_000, 0);
    add_refund(&path, 0x92, 0x84, 2_000);
    open_database(&path)
        .unwrap()
        .execute(
            "INSERT INTO main.sale_cancellations(sale_id,cancelled_at_ms) VALUES(?1,0)",
            params![[0x22_u8; 16].as_slice()],
        )
        .unwrap();

    add_sale(&path, 0x23, "finalized", &[1_000], None);
    add_sale(&path, 0x24, "open", &[1_000], None);

    let reader = open_database(&path).unwrap();
    assert_eq!(schema_version(&reader, 17).unwrap(), 17);
    reader
        .execute_batch(
            "CREATE TEMP TABLE sales AS SELECT * FROM main.sales WHERE 0;
             CREATE TEMP TABLE sale_items AS SELECT * FROM main.sale_items WHERE 0;
             CREATE TEMP TABLE receivables AS SELECT * FROM main.receivables WHERE 0;
             CREATE TEMP TABLE receipts AS SELECT * FROM main.receipts WHERE 0;
             CREATE TEMP TABLE sale_return_items AS SELECT * FROM main.sale_return_items WHERE 0;",
        )
        .unwrap();
    reader.pragma_update(None, "query_only", true).unwrap();
    let before = snapshot(&reader);
    let store = AdapterSqliteReceivableDueReadStore::new(&reader);
    let result = list_receivable_due_cents(&store).unwrap();

    assert_eq!(result, 1_500);
    assert!(reader.is_autocommit());
    assert_eq!(snapshot(&reader), before);
    assert_eq!(
        reader
            .query_row("PRAGMA main.integrity_check", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    assert_eq!(
        reader
            .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |row| {
                row.get::<_, i64>(0)
            })
            .unwrap(),
        0
    );
}

#[test]
fn corrupt_overpayment_and_missing_schema_fail_closed_without_partial_total() {
    let (_directory, path) = setup();
    add_sale(&path, 0x31, "finalized", &[500], Some(500));
    add_receipt(&path, 0x75, 0x31, 500);
    add_receipt(&path, 0x76, 0x31, 1);

    let reader = open_database(&path).unwrap();
    let store = AdapterSqliteReceivableDueReadStore::new(&reader);
    assert_store_error(store.read_remaining_receivable_due_cents());
    assert_app_error(list_receivable_due_cents(&store));

    let connection = Connection::open_in_memory().unwrap();
    let store = AdapterSqliteReceivableDueReadStore::new(&connection);
    assert_store_error(store.read_remaining_receivable_due_cents());
    assert_app_error(list_receivable_due_cents(&store));
}

#[test]
fn sqlite_receipt_sum_overflow_fails_closed_without_partial_total() {
    let (_directory, path) = setup();
    add_sale(&path, 0x32, "finalized", &[i64::MAX], Some(i64::MAX));
    add_receipt(&path, 0x77, 0x32, i64::MAX);
    add_receipt(&path, 0x78, 0x32, 1);

    let reader = open_database(&path).unwrap();
    reader.pragma_update(None, "query_only", true).unwrap();
    let before = snapshot(&reader);
    let store = AdapterSqliteReceivableDueReadStore::new(&reader);
    assert_store_error(store.read_remaining_receivable_due_cents());
    assert_app_error(list_receivable_due_cents(&store));
    assert!(reader.is_autocommit());
    assert_eq!(snapshot(&reader), before);
}
