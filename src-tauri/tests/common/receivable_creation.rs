use super::TempDirectory;
use millani_artes::database::{
    execute_versioned_migration, execute_write_transaction, open_database, schema_version,
    TransactionError, ACCOUNTS_MIGRATION_SQL, AUDIT_EVENT_MIGRATION_SQL, CATEGORIES_MIGRATION_SQL,
    CUSTOMERS_MIGRATION_SQL, MOVEMENTS_MIGRATION_SQL, MOVEMENT_CATEGORIES_MIGRATION_SQL,
    PRODUCTS_MIGRATION_SQL, RECEIVABLES_MIGRATION_SQL, RECURRENCES_MIGRATION_SQL,
    SALES_MIGRATION_SQL, SALE_ITEMS_MIGRATION_SQL,
};
use millani_artes::domain::{Receivable, SalePending};
use millani_artes::infrastructure::receivables::{
    persist_receivable_in_transaction, ReceivableStoreError,
};
use rusqlite::{params, types::Value, Connection};
use std::path::Path;

fn migrate(path: &Path) -> Connection {
    let mut connection =
        execute_versioned_migration(open_database(path).expect("DATABASE_OPEN_FAILED"), 1, "")
            .expect("MIGRATION_1_FAILED");
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
    ] {
        connection = execute_versioned_migration(connection, version, sql)
            .unwrap_or_else(|_| panic!("MIGRATION_{version}_FAILED"));
    }
    assert_eq!(schema_version(&connection, 12).unwrap(), 12);
    connection
}

fn setup(name: &str) -> (TempDirectory, std::path::PathBuf) {
    let directory = TempDirectory::new();
    let path = directory.path().join(name);
    drop(migrate(&path));
    (directory, path)
}

fn insert_customer(connection: &Connection, id: [u8; 16]) {
    connection
        .execute(
            "INSERT INTO main.customers (id, name, phone) VALUES (?1, 'Synthetic customer', NULL)",
            [id.as_slice()],
        )
        .expect("CUSTOMER_INSERT_FAILED");
}

fn insert_sale(connection: &Connection, id: [u8; 16], status: &str) {
    connection
        .execute(
            "INSERT INTO main.sales (id, status) VALUES (?1, ?2)",
            params![id.as_slice(), status],
        )
        .expect("SALE_INSERT_FAILED");
}

fn receivable(
    id: [u8; 16],
    sale_id: [u8; 16],
    amount_cents: i64,
    customer_id: [u8; 16],
    due_at_ms: i64,
) -> Receivable {
    Receivable {
        id,
        sale_id,
        pending: SalePending {
            amount_cents,
            customer_id,
            due_at_ms,
        },
    }
}

fn rows(connection: &Connection, sql: &str) -> Vec<Vec<Value>> {
    let mut statement = connection.prepare(sql).expect("QUERY_PREPARE_FAILED");
    let columns = statement.column_count();
    statement
        .query_map([], |row| {
            (0..columns)
                .map(|index| row.get(index))
                .collect::<rusqlite::Result<Vec<Value>>>()
        })
        .expect("QUERY_FAILED")
        .collect::<rusqlite::Result<_>>()
        .expect("QUERY_ROWS_FAILED")
}

fn attempt(
    path: &Path,
    value: &Receivable,
    event_id: [u8; 16],
) -> Result<(), TransactionError<ReceivableStoreError>> {
    execute_write_transaction(
        open_database(path).expect("DATABASE_OPEN_FAILED"),
        |transaction| {
            persist_receivable_in_transaction(transaction, value, event_id, 1_765_000_000_123)
        },
    )
    .map(|_| ())
}

#[test]
fn rcv003_persists_exact_receivable_and_audit_with_finalization_in_one_transaction() {
    let (_directory, path) = setup("rcv003-success.sqlite");
    let customer_id = [0x31; 16];
    let sale_id = [0x42; 16];
    let receivable_id = [0x53; 16];
    let event_id = [0x64; 16];
    let value = receivable(receivable_id, sale_id, i64::MAX, customer_id, i64::MIN);
    let connection = open_database(&path).expect("DATABASE_OPEN_FAILED");
    insert_customer(&connection, customer_id);
    insert_sale(&connection, sale_id, "open");

    let (connection, ()) = execute_write_transaction(connection, |transaction| {
        assert!(!transaction.is_autocommit());
        let updated = transaction
            .execute(
                "UPDATE main.sales SET status = 'finalized' WHERE id = ?1",
                [sale_id.as_slice()],
            )
            .map_err(|_| ReceivableStoreError::StoreFailed)?;
        if updated != 1 {
            return Err(ReceivableStoreError::StoreFailed);
        }
        persist_receivable_in_transaction(transaction, &value, event_id, 1_765_000_000_123)
    })
    .expect("RCV003_WRITE_TRANSACTION_FAILED");
    assert!(connection.is_autocommit());
    assert_eq!(
        rows(&connection, "SELECT count(*) FROM main.movements"),
        vec![vec![Value::Integer(0)]]
    );
    assert_eq!(
        rows(&connection, "SELECT count(*) FROM main.accounts"),
        vec![vec![Value::Integer(0)]]
    );
    drop(connection);

    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert_eq!(schema_version(&reopened, 12).unwrap(), 12);
    assert_eq!(
        rows(&reopened, "PRAGMA main.integrity_check"),
        vec![vec![Value::Text("ok".into())]]
    );
    assert!(rows(&reopened, "PRAGMA main.foreign_key_check").is_empty());
    assert_eq!(
        rows(
            &reopened,
            "SELECT id, sale_id, amount_cents, customer_id, due_at_ms FROM main.receivables"
        ),
        vec![vec![
            Value::Blob(receivable_id.to_vec()),
            Value::Blob(sale_id.to_vec()),
            Value::Integer(i64::MAX),
            Value::Blob(customer_id.to_vec()),
            Value::Integer(i64::MIN),
        ]]
    );
    assert_eq!(
        rows(
            &reopened,
            "SELECT event_id, occurred_at_ms, operation, entity, entity_id FROM main.audit_event"
        ),
        vec![vec![
            Value::Blob(event_id.to_vec()),
            Value::Integer(1_765_000_000_123),
            Value::Text("receivable_create".into()),
            Value::Text("receivable".into()),
            Value::Blob(receivable_id.to_vec()),
        ]]
    );
    assert_eq!(
        rows(&reopened, "SELECT status FROM main.sales"),
        vec![vec![Value::Text("finalized".into())]]
    );
    assert_eq!(
        rows(&reopened, "SELECT count(*) FROM main.movements"),
        vec![vec![Value::Integer(0)]]
    );
    assert_eq!(
        rows(&reopened, "SELECT count(*) FROM main.accounts"),
        vec![vec![Value::Integer(0)]]
    );
}

#[test]
fn rcv003_non_positive_principal_is_rejected_before_any_schema_read() {
    let directory = TempDirectory::new();
    for (index, amount_cents) in [0, -1].into_iter().enumerate() {
        let path = directory
            .path()
            .join(format!("rcv003-invalid-{index}.sqlite"));
        let value = receivable([0x71; 16], [0x72; 16], amount_cents, [0x73; 16], 0);
        let result = execute_write_transaction(
            open_database(&path).expect("DATABASE_OPEN_FAILED"),
            |transaction| {
                persist_receivable_in_transaction(
                    transaction,
                    &value,
                    [0x74; 16],
                    1_765_000_000_123,
                )
            },
        )
        .map(|_| ());
        assert!(matches!(
            result,
            Err(TransactionError::OperationFailed(
                ReceivableStoreError::InvalidAmount
            ))
        ));
        let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
        assert_eq!(
            rows(
                &reopened,
                "SELECT count(*) FROM main.sqlite_schema WHERE name IN ('sales', 'receivables')"
            ),
            vec![vec![Value::Integer(0)]]
        );
    }
}

#[test]
fn rcv003_missing_open_and_invalid_statuses_are_rejected_without_mutation() {
    for (index, status) in [None, Some("open"), Some("corrupt")]
        .into_iter()
        .enumerate()
    {
        let (_directory, path) = setup(&format!("rcv003-sale-state-{index}.sqlite"));
        let connection = open_database(&path).expect("DATABASE_OPEN_FAILED");
        let customer_id = [0x81; 16];
        let sale_id = [0x82 + index as u8; 16];
        insert_customer(&connection, customer_id);
        if let Some(status) = status {
            if status == "corrupt" {
                connection
                    .pragma_update(None, "ignore_check_constraints", true)
                    .expect("CHECK_CONSTRAINT_BYPASS_FAILED");
            }
            insert_sale(&connection, sale_id, status);
            if status == "corrupt" {
                connection
                    .pragma_update(None, "ignore_check_constraints", false)
                    .expect("CHECK_CONSTRAINT_RESTORE_FAILED");
            }
        }
        let value = receivable([0x83; 16], sale_id, 100, customer_id, 200);
        let result = attempt(&path, &value, [0x84; 16]);
        let expected_error = if status == Some("corrupt") {
            ReceivableStoreError::StoreFailed
        } else {
            ReceivableStoreError::SaleNotFinalized
        };
        assert!(matches!(
            (result, expected_error),
            (
                Err(TransactionError::OperationFailed(
                    ReceivableStoreError::SaleNotFinalized
                )),
                ReceivableStoreError::SaleNotFinalized
            ) | (
                Err(TransactionError::OperationFailed(
                    ReceivableStoreError::StoreFailed
                )),
                ReceivableStoreError::StoreFailed
            )
        ));
        let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
        let expected_sales = match status {
            Some(status) => vec![vec![
                Value::Blob(sale_id.to_vec()),
                Value::Text(status.into()),
            ]],
            None => Vec::new(),
        };
        assert_eq!(
            rows(&reopened, "SELECT id, status FROM main.sales ORDER BY id"),
            expected_sales
        );
        assert!(rows(&reopened, "SELECT * FROM main.receivables").is_empty());
        assert!(rows(&reopened, "SELECT * FROM main.audit_event").is_empty());
    }
}

#[test]
fn rcv003_foreign_key_and_unique_failures_do_not_leave_partial_writes() {
    let (_directory, path) = setup("rcv003-constraints.sqlite");
    let connection = open_database(&path).expect("DATABASE_OPEN_FAILED");
    let customer_id = [0x91; 16];
    let first_sale_id = [0x92; 16];
    let second_sale_id = [0x93; 16];
    insert_customer(&connection, customer_id);
    insert_sale(&connection, first_sale_id, "finalized");
    insert_sale(&connection, second_sale_id, "finalized");
    drop(connection);

    let original = receivable([0x94; 16], first_sale_id, 100, customer_id, 200);
    assert!(attempt(&path, &original, [0x95; 16]).is_ok());
    let before_receivables = rows(
        &open_database(&path).unwrap(),
        "SELECT id, sale_id, amount_cents, customer_id, due_at_ms FROM main.receivables",
    );
    let before_audit = rows(
        &open_database(&path).unwrap(),
        "SELECT event_id, occurred_at_ms, operation, entity, entity_id FROM main.audit_event",
    );

    let duplicate = receivable([0x96; 16], first_sale_id, 300, customer_id, 400);
    assert!(matches!(
        attempt(&path, &duplicate, [0x97; 16]),
        Err(TransactionError::OperationFailed(
            ReceivableStoreError::StoreFailed
        ))
    ));
    let missing_customer = receivable([0x98; 16], second_sale_id, 500, [0x99; 16], 600);
    assert!(matches!(
        attempt(&path, &missing_customer, [0x9a; 16]),
        Err(TransactionError::OperationFailed(
            ReceivableStoreError::StoreFailed
        ))
    ));

    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert_eq!(
        rows(
            &reopened,
            "SELECT id, sale_id, amount_cents, customer_id, due_at_ms FROM main.receivables"
        ),
        before_receivables
    );
    assert_eq!(
        rows(
            &reopened,
            "SELECT event_id, occurred_at_ms, operation, entity, entity_id FROM main.audit_event"
        ),
        before_audit
    );
    assert!(!format!(
        "{:?}{}",
        ReceivableStoreError::StoreFailed,
        ReceivableStoreError::StoreFailed
    )
    .contains("PRIVATE_SQL_MARKER"));
}

#[test]
fn rcv003_audit_failure_propagation_rolls_back_sale_receivable_and_audit() {
    let (_directory, path) = setup("rcv003-audit-rollback.sqlite");
    let connection = open_database(&path).expect("DATABASE_OPEN_FAILED");
    let customer_id = [0xa1; 16];
    let sale_id = [0xa2; 16];
    insert_customer(&connection, customer_id);
    insert_sale(&connection, sale_id, "open");
    connection
        .execute_batch(
            "CREATE TRIGGER main.rcv003_audit_fail BEFORE INSERT ON main.audit_event WHEN NEW.operation = 'receivable_create' BEGIN SELECT RAISE(ABORT, 'PRIVATE_SQL_MARKER'); END;",
        )
        .expect("AUDIT_FAILURE_TRIGGER_CREATE_FAILED");
    let value = receivable([0xa3; 16], sale_id, 700, customer_id, 800);

    let result = execute_write_transaction(connection, |transaction| {
        let updated = transaction
            .execute(
                "UPDATE main.sales SET status = 'finalized' WHERE id = ?1",
                [sale_id.as_slice()],
            )
            .map_err(|_| ReceivableStoreError::StoreFailed)?;
        if updated != 1 {
            return Err(ReceivableStoreError::StoreFailed);
        }
        persist_receivable_in_transaction(transaction, &value, [0xa4; 16], 1_765_000_000_123)
    });
    assert!(matches!(
        result,
        Err(TransactionError::OperationFailed(
            ReceivableStoreError::StoreFailed
        ))
    ));

    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert_eq!(
        rows(&reopened, "SELECT status FROM main.sales"),
        vec![vec![Value::Text("open".into())]]
    );
    assert!(rows(&reopened, "SELECT * FROM main.receivables").is_empty());
    assert!(rows(&reopened, "SELECT * FROM main.audit_event").is_empty());
    assert!(!format!(
        "{:?}{}",
        ReceivableStoreError::StoreFailed,
        ReceivableStoreError::StoreFailed
    )
    .contains("PRIVATE_SQL_MARKER"));
}

#[test]
fn rcv003_main_schema_writes_survive_temp_name_shadowing() {
    let (_directory, path) = setup("rcv003-temp-shadow.sqlite");
    let connection = open_database(&path).expect("DATABASE_OPEN_FAILED");
    let customer_id = [0xb1; 16];
    let sale_id = [0xb2; 16];
    let value = receivable([0xb3; 16], sale_id, 900, customer_id, 1_000);
    insert_customer(&connection, customer_id);
    insert_sale(&connection, sale_id, "open");
    connection
        .execute_batch(
            "CREATE TEMP TABLE sales (id BLOB, status TEXT); CREATE TEMP TABLE receivables (id BLOB, sale_id BLOB, amount_cents INTEGER, customer_id BLOB, due_at_ms INTEGER); CREATE TEMP TABLE audit_event (event_id BLOB, occurred_at_ms INTEGER, operation TEXT, entity TEXT, entity_id BLOB);",
        )
        .expect("TEMP_SHADOW_TABLES_CREATE_FAILED");
    connection
        .execute(
            "INSERT INTO temp.sales (id, status) VALUES (?1, 'open')",
            [sale_id.as_slice()],
        )
        .expect("TEMP_SALE_INSERT_FAILED");

    let (connection, ()) = execute_write_transaction(connection, |transaction| {
        let updated = transaction
            .execute(
                "UPDATE main.sales SET status = 'finalized' WHERE id = ?1",
                [sale_id.as_slice()],
            )
            .map_err(|_| ReceivableStoreError::StoreFailed)?;
        if updated != 1 {
            return Err(ReceivableStoreError::StoreFailed);
        }
        persist_receivable_in_transaction(transaction, &value, [0xb4; 16], 1_765_000_000_123)
    })
    .expect("RCV003_MAIN_SCHEMA_WRITE_FAILED");

    assert_eq!(
        rows(&connection, "SELECT count(*) FROM main.receivables"),
        vec![vec![Value::Integer(1)]]
    );
    assert_eq!(
        rows(&connection, "SELECT count(*) FROM main.audit_event"),
        vec![vec![Value::Integer(1)]]
    );
    assert_eq!(
        rows(&connection, "SELECT count(*) FROM temp.receivables"),
        vec![vec![Value::Integer(0)]]
    );
    assert_eq!(
        rows(&connection, "SELECT count(*) FROM temp.audit_event"),
        vec![vec![Value::Integer(0)]]
    );
    assert_eq!(
        rows(&connection, "SELECT status FROM main.sales"),
        vec![vec![Value::Text("finalized".into())]]
    );
    assert_eq!(
        rows(&connection, "SELECT status FROM temp.sales"),
        vec![vec![Value::Text("open".into())]]
    );
    drop(connection);

    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert_eq!(schema_version(&reopened, 12).unwrap(), 12);
    assert_eq!(
        rows(&reopened, "SELECT count(*) FROM main.receivables"),
        vec![vec![Value::Integer(1)]]
    );
}
