use super::TempDirectory;
use millani_artes::application::{
    calculate_context_result::calculate_context_result,
    calculate_home_expenses::calculate_home_expenses, list_account_balances::list_account_balances,
    list_accounts::list_accounts, list_month_sales::list_month_sale_cents,
    list_movements::list_movements, list_payables::list_payables_cents,
    list_receivables::list_receivable_due_cents, list_today_entries::list_today_entry_cents,
    list_today_exits::list_today_exit_cents,
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
use millani_artes::domain::ports::Clock;
use millani_artes::domain::{total_balance_cents, Context};
use millani_artes::infrastructure::accounts::AdapterSqliteAccountStore;
use millani_artes::infrastructure::movements::AdapterSqliteMovementStore;
use millani_artes::infrastructure::payable_due::AdapterSqlitePayableObligationsReadStore;
use millani_artes::infrastructure::receivable_due::AdapterSqliteReceivableDueReadStore;
use millani_artes::infrastructure::sales::AdapterSqliteSaleStore;
use rusqlite::types::Value;
use rusqlite::{Connection, ErrorCode};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const NOW_MS: i64 = 1_704_078_600_000;
const ACCOUNT_ID: [u8; 16] = [0x01; 16];
const HOME_ENTRY_ID: [u8; 16] = [0x02; 16];
const HOME_EXIT_ID: [u8; 16] = [0x03; 16];
const FACTORY_EXIT_ID: [u8; 16] = [0x04; 16];
const CUSTOMER_ID: [u8; 16] = [0x20; 16];
const SALE_ID: [u8; 16] = [0x30; 16];
const SALE_ITEM_ID: [u8; 16] = [0x31; 16];
const RECEIVABLE_ID: [u8; 16] = [0x40; 16];
const RECURRENCE_ID: [u8; 16] = [0x50; 16];
const OBLIGATION_ID: [u8; 16] = [0x51; 16];

type DatabaseSnapshot = (
    Vec<(String, String, String, Option<String>)>,
    Vec<(String, Vec<Vec<Value>>)>,
);

struct FixedClock;

impl Clock for FixedClock {
    fn now(&self) -> SystemTime {
        UNIX_EPOCH + Duration::from_millis(NOW_MS as u64)
    }
}

fn setup() -> (TempDirectory, std::path::PathBuf) {
    let directory = TempDirectory::new();
    let path = directory.path().join("dsh010.sqlite");
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
            "INSERT INTO main.accounts(id,name,account_type,initial_balance_cents) VALUES(?1,'synthetic','cash',10000)",
            [ACCOUNT_ID.as_slice()],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO main.customers(id,name,phone) VALUES(?1,'synthetic customer',NULL)",
            [CUSTOMER_ID.as_slice()],
        )
        .unwrap();
    for (movement_id, kind, amount, context) in [
        (HOME_ENTRY_ID, "entry", 500_i64, "home"),
        (HOME_EXIT_ID, "exit", 200_i64, "home"),
        (FACTORY_EXIT_ID, "exit", 75_i64, "factory"),
    ] {
        connection
            .execute(
                "INSERT INTO main.movements(id,kind,amount_cents,context,account_id) VALUES(?1,?2,?3,?4,?5)",
                rusqlite::params![movement_id.as_slice(), kind, amount, context, ACCOUNT_ID.as_slice()],
            )
            .unwrap();
    }
    for (event_id, movement_id, operation) in [
        ([0x60_u8; 16], HOME_ENTRY_ID, "movement_entry_create"),
        ([0x61_u8; 16], HOME_EXIT_ID, "movement_exit_create"),
        ([0x62_u8; 16], FACTORY_EXIT_ID, "movement_exit_create"),
    ] {
        connection
            .execute(
                "INSERT INTO main.audit_event(event_id,occurred_at_ms,operation,entity,entity_id) VALUES(?1,?2,?3,'movement',?4)",
                rusqlite::params![event_id.as_slice(), NOW_MS, operation, movement_id.as_slice()],
            )
            .unwrap();
    }
    connection
        .execute(
            "INSERT INTO main.sales(id,status) VALUES(?1,'finalized')",
            [SALE_ID.as_slice()],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO main.sale_items(id,sale_id,position,product_id,description,quantity,unit_price_cents,discount_cents,total_cents) VALUES(?1,?2,0,NULL,'synthetic item',1,3000,0,3000)",
            rusqlite::params![SALE_ITEM_ID.as_slice(), SALE_ID.as_slice()],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO main.audit_event(event_id,occurred_at_ms,operation,entity,entity_id) VALUES(?1,?2,'sale_finalize','sale',?3)",
            rusqlite::params![[0x63_u8; 16].as_slice(), NOW_MS, SALE_ID.as_slice()],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO main.receivables(id,sale_id,amount_cents,customer_id,due_at_ms) VALUES(?1,?2,1200,?3,?4)",
            rusqlite::params![RECEIVABLE_ID.as_slice(), SALE_ID.as_slice(), CUSTOMER_ID.as_slice(), NOW_MS],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO main.recurrences(id,name,kind,amount_cents,context,due_day) VALUES(?1,'synthetic variable','variable',NULL,'home',1)",
            [RECURRENCE_ID.as_slice()],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO main.recurrence_obligations(id,recurrence_id,period_year,period_month,due_at_ms,state,amount_cents,movement_id,context) VALUES(?1,?2,2024,1,?3,'payable',4000,NULL,'home')",
            rusqlite::params![OBLIGATION_ID.as_slice(), RECURRENCE_ID.as_slice(), NOW_MS],
        )
        .unwrap();
    drop(connection);
    (directory, path)
}

fn query_only_connection(path: &std::path::Path) -> Connection {
    let connection = open_database(path).unwrap();
    connection.pragma_update(None, "query_only", true).unwrap();
    let query_only: i64 = connection
        .pragma_query_value(None, "query_only", |row| row.get(0))
        .unwrap();
    assert_eq!(query_only, 1);
    connection
}

fn snapshot(connection: &Connection) -> DatabaseSnapshot {
    let schema = {
        let mut statement = connection
            .prepare("SELECT type,name,tbl_name,sql FROM main.sqlite_schema WHERE name NOT GLOB 'sqlite_*' ORDER BY type,name")
            .unwrap();
        statement
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap()
    };
    let tables: Vec<String> = {
        let mut statement = connection
            .prepare("SELECT name FROM main.sqlite_schema WHERE type='table' AND name NOT GLOB 'sqlite_*' ORDER BY name")
            .unwrap();
        statement
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    let rows = tables
        .into_iter()
        .map(|table| {
            let quoted = table.replace('"', "\"\"");
            let statement = connection
                .prepare(&format!("SELECT * FROM main.\"{quoted}\""))
                .unwrap();
            let columns = statement.column_count();
            let order = (1..=columns)
                .map(|column| column.to_string())
                .collect::<Vec<_>>()
                .join(",");
            drop(statement);
            let mut statement = connection
                .prepare(&format!("SELECT * FROM main.\"{quoted}\" ORDER BY {order}"))
                .unwrap();
            let columns = statement.column_count();
            let values = statement
                .query_map([], |row| {
                    (0..columns)
                        .map(|column| row.get::<_, Value>(column))
                        .collect::<rusqlite::Result<Vec<_>>>()
                })
                .unwrap()
                .collect::<rusqlite::Result<Vec<_>>>()
                .unwrap();
            (table, values)
        })
        .collect();
    (schema, rows)
}

fn assert_healthy(connection: &Connection) {
    assert!(connection.is_autocommit());
    assert_eq!(schema_version(connection, 17).unwrap(), 17);
    let integrity: String = connection
        .query_row("PRAGMA main.integrity_check", [], |row| row.get(0))
        .unwrap();
    assert_eq!(integrity, "ok");
    let foreign_key_violations: i64 = connection
        .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(foreign_key_violations, 0);
}

#[test]
fn all_dashboard_projections_read_without_changing_the_database() {
    let (_directory, path) = setup();
    let before_connection = open_database(&path).unwrap();
    before_connection
        .execute_batch(
            "CREATE TABLE main.sqliteX_shadow (id INTEGER PRIMARY KEY, value TEXT NOT NULL); INSERT INTO main.sqliteX_shadow VALUES(1, 'sentinel');",
        )
        .unwrap();
    assert_healthy(&before_connection);
    let before = snapshot(&before_connection);
    assert!(before
        .0
        .iter()
        .any(|(_, name, _, _)| name == "sqliteX_shadow"));
    assert!(before
        .1
        .iter()
        .any(|(name, rows)| name == "sqliteX_shadow" && rows.len() == 1));
    drop(before_connection);

    let guard = query_only_connection(&path);
    let blocked_write = guard.execute(
        "UPDATE main.accounts SET initial_balance_cents=initial_balance_cents+1 WHERE id=?1",
        [ACCOUNT_ID.as_slice()],
    );
    assert!(matches!(
        blocked_write,
        Err(rusqlite::Error::SqliteFailure(ref error, _)) if error.code == ErrorCode::ReadOnly
    ));
    drop(guard);

    let accounts = list_accounts(&AdapterSqliteAccountStore::new(query_only_connection(
        &path,
    )))
    .expect("DSH001_ACCOUNT_READ_FAILED");
    let movements = list_movements(&AdapterSqliteMovementStore::new(query_only_connection(
        &path,
    )))
    .expect("DSH001_MOVEMENT_READ_FAILED");
    assert_eq!(total_balance_cents(&accounts, &movements), Ok(10_225));

    let balances = list_account_balances(
        &AdapterSqliteAccountStore::new(query_only_connection(&path)),
        &AdapterSqliteMovementStore::new(query_only_connection(&path)),
    )
    .expect("DSH002_BALANCE_READ_FAILED");
    assert_eq!(balances.len(), 1);
    assert_eq!(balances[0].balance_cents, 10_225);

    let movement_reader = AdapterSqliteMovementStore::new(query_only_connection(&path));
    assert_eq!(
        list_today_entry_cents(&FixedClock, &movement_reader).unwrap(),
        500
    );
    assert_eq!(
        list_today_exit_cents(&FixedClock, &movement_reader).unwrap(),
        275
    );
    assert_eq!(
        calculate_context_result(&Context::Factory, &movement_reader).unwrap(),
        -75
    );
    assert_eq!(calculate_home_expenses(&movement_reader).unwrap(), 200);

    let month_sales = list_month_sale_cents(
        &FixedClock,
        &AdapterSqliteSaleStore::new(query_only_connection(&path)),
    )
    .expect("DSH005_SALES_READ_FAILED");
    assert_eq!(month_sales, 3_000);

    let obligations_reader = query_only_connection(&path);
    assert_eq!(
        list_receivable_due_cents(&AdapterSqliteReceivableDueReadStore::new(
            &obligations_reader
        ))
        .unwrap(),
        1_200
    );
    assert_eq!(
        list_payables_cents(&AdapterSqlitePayableObligationsReadStore::new(
            &obligations_reader
        ))
        .unwrap(),
        4_000
    );
    drop(obligations_reader);

    let after_connection = open_database(&path).unwrap();
    assert_healthy(&after_connection);
    assert_eq!(snapshot(&after_connection), before);
}
