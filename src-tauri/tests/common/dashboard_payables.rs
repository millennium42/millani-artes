use super::TempDirectory;
use millani_artes::application::list_payables::{
    list_payables_cents, ListPayablesError, PayableObligationsReadStore,
};
use millani_artes::database::{
    execute_versioned_migration, open_database, schema_version, ACCOUNTS_MIGRATION_SQL,
    AUDIT_EVENT_MIGRATION_SQL, CATEGORIES_MIGRATION_SQL, MOVEMENTS_MIGRATION_SQL,
    MOVEMENT_CATEGORIES_MIGRATION_SQL, RECURRENCES_MIGRATION_SQL,
};
use millani_artes::infrastructure::payable_due::{
    AdapterSqlitePayableObligationsReadStore, PayableObligationsReadError,
};
use rusqlite::{params, types::Value, Connection};
use std::path::PathBuf;

const HOME_RECURRENCE_ID: [u8; 16] = [0x31; 16];
const FACTORY_RECURRENCE_ID: [u8; 16] = [0x32; 16];
const ACCOUNT_ID: [u8; 16] = [0x33; 16];
const PAID_MOVEMENT_ID: [u8; 16] = [0x34; 16];

struct ObligationFixture<'a> {
    id: u8,
    recurrence_id: &'a [u8; 16],
    month: u8,
    due_at_ms: i64,
    state: &'a str,
    amount_cents: Option<i64>,
    movement_id: Option<&'a [u8]>,
    context: &'a str,
}

fn setup() -> (TempDirectory, PathBuf) {
    let directory = TempDirectory::new();
    let path = directory.path().join("dsh007.sqlite");
    let mut connection = execute_versioned_migration(open_database(&path).unwrap(), 1, "").unwrap();
    for (version, sql) in [
        (2, AUDIT_EVENT_MIGRATION_SQL),
        (3, ACCOUNTS_MIGRATION_SQL),
        (4, CATEGORIES_MIGRATION_SQL),
        (5, MOVEMENTS_MIGRATION_SQL),
        (6, MOVEMENT_CATEGORIES_MIGRATION_SQL),
        (7, RECURRENCES_MIGRATION_SQL),
    ] {
        connection = execute_versioned_migration(connection, version, sql).unwrap();
    }
    connection
        .execute(
            "INSERT INTO main.accounts(id,name,account_type,initial_balance_cents) VALUES(?1,'synthetic','cash',0)",
            params![ACCOUNT_ID.as_slice()],
        )
        .unwrap();
    for (id, context) in [
        (HOME_RECURRENCE_ID, "home"),
        (FACTORY_RECURRENCE_ID, "factory"),
    ] {
        connection
            .execute(
                "INSERT INTO main.recurrences(id,name,kind,amount_cents,context,due_day) VALUES(?1,'synthetic variable','variable',NULL,?2,1)",
                params![id.as_slice(), context],
            )
            .unwrap();
    }
    drop(connection);
    (directory, path)
}

fn add_obligation(connection: &Connection, obligation: ObligationFixture<'_>) {
    connection
        .execute(
            "INSERT INTO main.recurrence_obligations(id,recurrence_id,period_year,period_month,due_at_ms,state,amount_cents,movement_id,context) VALUES(?1,?2,2026,?3,?4,?5,?6,?7,?8)",
            params![
                [obligation.id; 16].as_slice(),
                obligation.recurrence_id.as_slice(),
                obligation.month,
                obligation.due_at_ms,
                obligation.state,
                obligation.amount_cents,
                obligation.movement_id,
                obligation.context
            ],
        )
        .unwrap();
}

fn main_rows(connection: &Connection, table: &str) -> Vec<Vec<Value>> {
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

fn temp_obligation_rows(connection: &Connection) -> Vec<Vec<Value>> {
    let mut statement = connection
        .prepare("SELECT * FROM temp.recurrence_obligations ORDER BY 1")
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
        "audit_event",
        "movements",
        "recurrences",
        "recurrence_obligations",
    ]
    .into_iter()
    .map(|table| main_rows(connection, table))
    .collect()
}

fn assert_dashboard_error(result: Result<i64, ListPayablesError>) {
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("expected a static dashboard error"),
    };
    assert_eq!(format!("{error:?}"), "ListPayablesError");
    assert_eq!(format!("{error}"), "payable total could not be read");
    assert!(std::error::Error::source(&error).is_none());
}

fn assert_store_error(result: Result<Vec<i64>, PayableObligationsReadError>) {
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("expected a static adapter error"),
    };
    assert_eq!(format!("{error:?}"), "PayableObligationsReadError");
    assert_eq!(
        format!("{error}"),
        "payable obligation history could not be read"
    );
    assert!(std::error::Error::source(&error).is_none());
}

#[test]
fn dashboard_payables_projects_persisted_open_states_after_reopen_read_only() {
    let (_directory, path) = setup();
    {
        let connection = open_database(&path).unwrap();
        connection
            .execute(
                "INSERT INTO main.movements(id,kind,amount_cents,context,account_id) VALUES(?1,'exit',900,'factory',?2)",
                params![PAID_MOVEMENT_ID.as_slice(), ACCOUNT_ID.as_slice()],
            )
            .unwrap();
        add_obligation(
            &connection,
            ObligationFixture {
                id: 0x41,
                recurrence_id: &HOME_RECURRENCE_ID,
                month: 1,
                due_at_ms: -1,
                state: "payable",
                amount_cents: Some(2_000),
                movement_id: None,
                context: "home",
            },
        );
        add_obligation(
            &connection,
            ObligationFixture {
                id: 0x42,
                recurrence_id: &FACTORY_RECURRENCE_ID,
                month: 2,
                due_at_ms: -2,
                state: "overdue",
                amount_cents: Some(3_000),
                movement_id: None,
                context: "factory",
            },
        );
        add_obligation(
            &connection,
            ObligationFixture {
                id: 0x43,
                recurrence_id: &HOME_RECURRENCE_ID,
                month: 3,
                due_at_ms: 0,
                state: "awaiting_value",
                amount_cents: None,
                movement_id: None,
                context: "home",
            },
        );
        add_obligation(
            &connection,
            ObligationFixture {
                id: 0x44,
                recurrence_id: &FACTORY_RECURRENCE_ID,
                month: 4,
                due_at_ms: 1,
                state: "paid",
                amount_cents: Some(900),
                movement_id: Some(PAID_MOVEMENT_ID.as_slice()),
                context: "factory",
            },
        );
    }

    let reader = open_database(&path).unwrap();
    assert_eq!(schema_version(&reader, 7).unwrap(), 7);
    reader
        .execute_batch(
            "CREATE TEMP TABLE recurrence_obligations AS SELECT * FROM main.recurrence_obligations WHERE 0;
             INSERT INTO temp.recurrence_obligations(id,recurrence_id,period_year,period_month,due_at_ms,state,amount_cents,movement_id,context)
             VALUES(zeroblob(16),zeroblob(16),2026,5,0,'payable',999999,NULL,'home');",
        )
        .unwrap();
    reader.pragma_update(None, "query_only", true).unwrap();
    let before = snapshot(&reader);
    let shadow_before = temp_obligation_rows(&reader);
    let store = AdapterSqlitePayableObligationsReadStore::new(&reader);

    assert_eq!(list_payables_cents(&store).unwrap(), 5_000);
    assert!(reader.is_autocommit());
    assert_eq!(snapshot(&reader), before);
    assert_eq!(temp_obligation_rows(&reader), shadow_before);
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
fn overflow_and_corrupt_rows_fail_closed_without_partial_total() {
    let (_directory, path) = setup();
    {
        let connection = open_database(&path).unwrap();
        add_obligation(
            &connection,
            ObligationFixture {
                id: 0x51,
                recurrence_id: &HOME_RECURRENCE_ID,
                month: 1,
                due_at_ms: 0,
                state: "payable",
                amount_cents: Some(i64::MAX),
                movement_id: None,
                context: "home",
            },
        );
        add_obligation(
            &connection,
            ObligationFixture {
                id: 0x52,
                recurrence_id: &HOME_RECURRENCE_ID,
                month: 2,
                due_at_ms: 0,
                state: "overdue",
                amount_cents: Some(1),
                movement_id: None,
                context: "home",
            },
        );
    }
    let reader = open_database(&path).unwrap();
    reader.pragma_update(None, "query_only", true).unwrap();
    let before = snapshot(&reader);
    let store = AdapterSqlitePayableObligationsReadStore::new(&reader);
    assert_dashboard_error(list_payables_cents(&store));
    assert!(reader.is_autocommit());
    assert_eq!(snapshot(&reader), before);

    for (id, state, amount, movement_id) in [
        (0x61, "payable", Some(0), None),
        (0x62, "unrecognized", Some(1), None),
        (0x63, "paid", Some(1), Some(vec![0x7f])),
        (0x64, "awaiting_value", Some(1), None),
    ] {
        let (_directory, path) = setup();
        {
            let connection = open_database(&path).unwrap();
            connection
                .pragma_update(None, "ignore_check_constraints", true)
                .unwrap();
            if let Some(movement_id) = movement_id.as_deref() {
                connection
                    .execute(
                        "INSERT INTO main.movements(id,kind,amount_cents,context,account_id) VALUES(?1,'exit',1,'home',?2)",
                        params![movement_id, ACCOUNT_ID.as_slice()],
                    )
                    .unwrap();
            }
            add_obligation(
                &connection,
                ObligationFixture {
                    id,
                    recurrence_id: &HOME_RECURRENCE_ID,
                    month: 1,
                    due_at_ms: 0,
                    state,
                    amount_cents: amount,
                    movement_id: movement_id.as_deref(),
                    context: "home",
                },
            );
        }
        let reader = open_database(&path).unwrap();
        let before = snapshot(&reader);
        let store = AdapterSqlitePayableObligationsReadStore::new(&reader);
        assert_store_error(store.read_open_recurrence_obligation_amounts_cents());
        assert_dashboard_error(list_payables_cents(&store));
        assert!(reader.is_autocommit());
        assert_eq!(snapshot(&reader), before);
    }

    let connection = Connection::open_in_memory().unwrap();
    let store = AdapterSqlitePayableObligationsReadStore::new(&connection);
    assert_store_error(store.read_open_recurrence_obligation_amounts_cents());
    assert_dashboard_error(list_payables_cents(&store));
}
