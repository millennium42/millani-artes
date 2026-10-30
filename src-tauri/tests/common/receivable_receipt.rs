use super::TempDirectory;
use millani_artes::database::{
    execute_versioned_migration, execute_write_transaction, open_database, schema_version,
    MigrationError, TransactionError, ACCOUNTS_MIGRATION_SQL, AUDIT_EVENT_MIGRATION_SQL,
    CATEGORIES_MIGRATION_SQL, CUSTOMERS_MIGRATION_SQL, MOVEMENTS_MIGRATION_SQL,
    MOVEMENT_CATEGORIES_MIGRATION_SQL, PRODUCTS_MIGRATION_SQL, RECEIPTS_MIGRATION_SQL,
    RECEIPTS_MIGRATION_VERSION, RECEIVABLES_MIGRATION_SQL, RECURRENCES_MIGRATION_SQL,
    SALES_MIGRATION_SQL, SALE_ITEMS_MIGRATION_SQL,
};
use millani_artes::domain::{Context, ReceivableReceipt};
use millani_artes::infrastructure::receivables::{
    persist_receivable_receipt_in_transaction, ReceivableReceiptStoreError,
};
use rusqlite::{params, types::Value, Connection};
use std::path::Path;

const ACCOUNT_A: [u8; 16] = [1; 16];
const ACCOUNT_B: [u8; 16] = [2; 16];
const CUSTOMER_ID: [u8; 16] = [3; 16];
const SALE_ID: [u8; 16] = [4; 16];
const RECEIVABLE_ID: [u8; 16] = [5; 16];

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

fn migrate_to_v12(path: &Path) -> Connection {
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

fn migrate_to_v13(path: &Path) -> Connection {
    let connection = migrate_to_v12(path);
    apply_receipts_migration(connection)
}

fn apply_receipts_migration(connection: Connection) -> Connection {
    assert_eq!(RECEIPTS_MIGRATION_VERSION, 13);
    execute_versioned_migration(
        connection,
        RECEIPTS_MIGRATION_VERSION,
        RECEIPTS_MIGRATION_SQL,
    )
    .expect("RECEIPTS_MIGRATION_FAILED")
}

fn seed(connection: &Connection, amount_cents: i64) {
    connection
        .execute(
            "INSERT INTO main.accounts (id,name,account_type,initial_balance_cents) VALUES (?1,'Conta sintética A','cash',100)",
            [ACCOUNT_A.as_slice()],
        )
        .expect("ACCOUNT_A_INSERT_FAILED");
    connection
        .execute(
            "INSERT INTO main.accounts (id,name,account_type,initial_balance_cents) VALUES (?1,'Conta sintética B','bank_pix',200)",
            [ACCOUNT_B.as_slice()],
        )
        .expect("ACCOUNT_B_INSERT_FAILED");
    connection
        .execute(
            "INSERT INTO main.customers (id,name,phone) VALUES (?1,'Cliente sintético',NULL)",
            [CUSTOMER_ID.as_slice()],
        )
        .expect("CUSTOMER_INSERT_FAILED");
    connection
        .execute(
            "INSERT INTO main.sales (id,status) VALUES (?1,'finalized')",
            [SALE_ID.as_slice()],
        )
        .expect("SALE_INSERT_FAILED");
    connection
        .execute(
            "INSERT INTO main.receivables (id,sale_id,amount_cents,customer_id,due_at_ms) VALUES (?1,?2,?3,?4,1800000000000)",
            params![
                RECEIVABLE_ID.as_slice(),
                SALE_ID.as_slice(),
                amount_cents,
                CUSTOMER_ID.as_slice()
            ],
        )
        .expect("RECEIVABLE_INSERT_FAILED");
}

fn setup(name: &str, amount_cents: i64) -> (TempDirectory, std::path::PathBuf) {
    let directory = TempDirectory::new();
    let path = directory.path().join(name);
    let connection = migrate_to_v13(&path);
    seed(&connection, amount_cents);
    drop(connection);
    (directory, path)
}

fn receipt(account_id: [u8; 16], amount_cents: i64, context: Context) -> ReceivableReceipt {
    ReceivableReceipt {
        account_id,
        amount_cents,
        context,
    }
}

fn write_receipt(
    path: &Path,
    receivable_id: [u8; 16],
    receipt_id: [u8; 16],
    value: ReceivableReceipt,
    event_id: [u8; 16],
    occurred_at_ms: i64,
) -> Result<(), TransactionError<ReceivableReceiptStoreError>> {
    execute_write_transaction(
        open_database(path).expect("DATABASE_OPEN_FAILED"),
        |transaction| {
            persist_receivable_receipt_in_transaction(
                transaction,
                receivable_id,
                receipt_id,
                &value,
                event_id,
                occurred_at_ms,
            )
        },
    )
    .map(|(connection, ())| drop(connection))
}

#[test]
fn rcv006_empty_chain_and_v12_upgrade_preserve_existing_rows() {
    let directory = TempDirectory::new();
    let empty_path = directory.path().join("empty.sqlite");
    let empty = migrate_to_v13(&empty_path);
    assert_eq!(schema_version(&empty, 13).unwrap(), 13);
    assert!(rows(&empty, "SELECT * FROM main.receipts").is_empty());
    assert_eq!(
        rows(&empty, "PRAGMA main.integrity_check"),
        vec![vec![Value::Text("ok".into())]]
    );
    assert!(rows(&empty, "PRAGMA main.foreign_key_check").is_empty());
    drop(empty);

    let upgrade_path = directory.path().join("upgrade.sqlite");
    let v12 = migrate_to_v12(&upgrade_path);
    seed(&v12, 3000);
    let old_rows: Vec<Vec<Vec<Value>>> = [
        "accounts",
        "customers",
        "sales",
        "receivables",
        "schema_migration",
    ]
    .iter()
    .map(|table| rows(&v12, &format!("SELECT * FROM main.{table} ORDER BY rowid")))
    .collect();
    let upgraded = apply_receipts_migration(v12);
    assert_eq!(schema_version(&upgraded, 13).unwrap(), 13);
    let after: Vec<Vec<Vec<Value>>> = [
        "accounts",
        "customers",
        "sales",
        "receivables",
        "schema_migration",
    ]
    .iter()
    .map(|table| {
        rows(
            &upgraded,
            &format!("SELECT * FROM main.{table} ORDER BY rowid"),
        )
    })
    .collect();
    assert_eq!(after[..4], old_rows[..4]);
    assert_eq!(
        rows(
            &upgraded,
            "SELECT version FROM main.schema_migration ORDER BY version"
        ),
        (1_i64..=13)
            .map(|version| vec![Value::Integer(version)])
            .collect::<Vec<_>>()
    );
    assert_eq!(
        rows(&upgraded, "PRAGMA main.integrity_check"),
        vec![vec![Value::Text("ok".into())]]
    );
    assert!(rows(&upgraded, "PRAGMA main.foreign_key_check").is_empty());
    drop(upgraded);
    let reopened = open_database(&upgrade_path).expect("UPGRADE_REOPEN_FAILED");
    assert_eq!(schema_version(&reopened, 13).unwrap(), 13);
    for (table, expected) in ["accounts", "customers", "sales", "receivables"]
        .iter()
        .zip(old_rows.iter())
    {
        assert_eq!(
            rows(
                &reopened,
                &format!("SELECT * FROM main.{table} ORDER BY rowid")
            ),
            *expected
        );
    }
}

#[test]
fn rcv006_schema_constraints_and_foreign_keys_are_enforced() {
    let directory = TempDirectory::new();
    let path = directory.path().join("constraints.sqlite");
    let connection = migrate_to_v13(&path);
    seed(&connection, 3000);
    let columns: Vec<(String, String, i64, i64)> = {
        let mut statement = connection
            .prepare("PRAGMA main.table_xinfo('receipts')")
            .unwrap();
        statement
            .query_map([], |row| {
                Ok((row.get(1)?, row.get(2)?, row.get(3)?, row.get(5)?))
            })
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(
        columns,
        [
            ("id".into(), "BLOB".into(), 1, 1),
            ("receivable_id".into(), "BLOB".into(), 1, 0),
            ("account_id".into(), "BLOB".into(), 1, 0),
            ("amount_cents".into(), "INTEGER".into(), 1, 0),
            ("context".into(), "TEXT".into(), 1, 0),
        ]
    );
    let table_sql: String = connection
        .query_row(
            "SELECT sql FROM main.sqlite_schema WHERE type='table' AND name='receipts'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(table_sql.ends_with("STRICT"));
    assert_eq!(
        rows(
            &connection,
            "SELECT name FROM main.sqlite_schema WHERE type='index' AND name='receipts_receivable_id_idx'"
        ),
        vec![vec![Value::Text("receipts_receivable_id_idx".into())]]
    );
    let foreign_keys: Vec<(String, String, String, String)> = {
        let mut statement = connection
            .prepare("PRAGMA main.foreign_key_list('receipts')")
            .unwrap();
        statement
            .query_map([], |row| {
                Ok((row.get(2)?, row.get(3)?, row.get(5)?, row.get(6)?))
            })
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(
        foreign_keys,
        [
            (
                "accounts".into(),
                "account_id".into(),
                "NO ACTION".into(),
                "NO ACTION".into()
            ),
            (
                "receivables".into(),
                "receivable_id".into(),
                "NO ACTION".into(),
                "NO ACTION".into()
            ),
        ]
    );

    let valid_id = [6_u8; 16];
    assert_eq!(
        connection.execute(
            "INSERT INTO main.receipts VALUES (?1,?2,?3,?4,?5)",
            params![
                valid_id.as_slice(),
                RECEIVABLE_ID.as_slice(),
                ACCOUNT_A.as_slice(),
                1000_i64,
                "home"
            ],
        ),
        Ok(1)
    );
    let before = rows(&connection, "SELECT * FROM main.receipts");
    let invalid_inserts = [
        connection.execute(
            "INSERT INTO main.receipts VALUES (?1,?2,?3,?4,?5)",
            params![
                [7_u8; 15].as_slice(),
                RECEIVABLE_ID.as_slice(),
                ACCOUNT_A.as_slice(),
                1_i64,
                "home"
            ],
        ),
        connection.execute(
            "INSERT INTO main.receipts VALUES (?1,?2,?3,?4,?5)",
            params![
                [7_u8; 16].as_slice(),
                [8_u8; 15].as_slice(),
                ACCOUNT_A.as_slice(),
                1_i64,
                "home"
            ],
        ),
        connection.execute(
            "INSERT INTO main.receipts VALUES (?1,?2,?3,?4,?5)",
            params![
                [7_u8; 16].as_slice(),
                RECEIVABLE_ID.as_slice(),
                [8_u8; 15].as_slice(),
                1_i64,
                "home"
            ],
        ),
        connection.execute(
            "INSERT INTO main.receipts VALUES (?1,?2,?3,?4,?5)",
            params![
                [7_u8; 16].as_slice(),
                RECEIVABLE_ID.as_slice(),
                ACCOUNT_A.as_slice(),
                0_i64,
                "home"
            ],
        ),
        connection.execute(
            "INSERT INTO main.receipts VALUES (?1,?2,?3,?4,?5)",
            params![
                [7_u8; 16].as_slice(),
                RECEIVABLE_ID.as_slice(),
                ACCOUNT_A.as_slice(),
                -1_i64,
                "home"
            ],
        ),
        connection.execute(
            "INSERT INTO main.receipts VALUES (?1,?2,?3,?4,?5)",
            params![
                [7_u8; 16].as_slice(),
                RECEIVABLE_ID.as_slice(),
                ACCOUNT_A.as_slice(),
                1_i64,
                "Home"
            ],
        ),
        connection.execute(
            "INSERT INTO main.receipts VALUES (?1,?2,?3,?4,?5)",
            params![
                [7_u8; 16].as_slice(),
                [8_u8; 16].as_slice(),
                ACCOUNT_A.as_slice(),
                1_i64,
                "home"
            ],
        ),
        connection.execute(
            "INSERT INTO main.receipts VALUES (?1,?2,?3,?4,?5)",
            params![
                [7_u8; 16].as_slice(),
                RECEIVABLE_ID.as_slice(),
                [8_u8; 16].as_slice(),
                1_i64,
                "home"
            ],
        ),
        connection.execute(
            "INSERT INTO main.receipts VALUES (?1,?2,?3,?4,?5)",
            params![
                [7_u8; 16].as_slice(),
                RECEIVABLE_ID.as_slice(),
                ACCOUNT_A.as_slice(),
                "not-an-integer",
                "home"
            ],
        ),
    ];
    assert!(invalid_inserts.into_iter().all(|result| result.is_err()));
    assert!(connection
        .execute(
            "UPDATE main.receipts SET amount_cents=0 WHERE id=?1",
            [valid_id.as_slice()]
        )
        .is_err());
    assert!(connection
        .execute(
            "UPDATE main.receipts SET context='other' WHERE id=?1",
            [valid_id.as_slice()]
        )
        .is_err());
    assert_eq!(rows(&connection, "SELECT * FROM main.receipts"), before);
    assert!(connection
        .execute(
            "DELETE FROM main.accounts WHERE id=?1",
            [ACCOUNT_A.as_slice()]
        )
        .is_err());
    assert!(connection
        .execute(
            "DELETE FROM main.receivables WHERE id=?1",
            [RECEIVABLE_ID.as_slice()]
        )
        .is_err());
    assert_eq!(rows(&connection, "SELECT * FROM main.receipts"), before);
}

#[test]
fn rcv006_records_receipts_with_audit_and_preserves_account_amount_context() {
    let (_directory, path) = setup("persist.sqlite", 3000);
    let first_id = [11_u8; 16];
    let first_event = [12_u8; 16];
    write_receipt(
        &path,
        RECEIVABLE_ID,
        first_id,
        receipt(ACCOUNT_A, 1000, Context::Home),
        first_event,
        1_800_000_000_001,
    )
    .expect("PARTIAL_RECEIPT_FAILED");
    let second_id = [13_u8; 16];
    let second_event = [14_u8; 16];
    write_receipt(
        &path,
        RECEIVABLE_ID,
        second_id,
        receipt(ACCOUNT_B, 2000, Context::Factory),
        second_event,
        1_800_000_000_002,
    )
    .expect("FULL_RECEIPT_FAILED");

    let reopened = open_database(&path).expect("RECEIPT_REOPEN_FAILED");
    assert_eq!(
        rows(&reopened, "SELECT id,receivable_id,account_id,amount_cents,context FROM main.receipts ORDER BY id"),
        vec![
            vec![Value::Blob(first_id.to_vec()), Value::Blob(RECEIVABLE_ID.to_vec()), Value::Blob(ACCOUNT_A.to_vec()), Value::Integer(1000), Value::Text("home".into())],
            vec![Value::Blob(second_id.to_vec()), Value::Blob(RECEIVABLE_ID.to_vec()), Value::Blob(ACCOUNT_B.to_vec()), Value::Integer(2000), Value::Text("factory".into())],
        ]
    );
    assert_eq!(
        rows(&reopened, "SELECT event_id,occurred_at_ms,operation,entity,entity_id FROM main.audit_event ORDER BY occurred_at_ms"),
        vec![
            vec![Value::Blob(first_event.to_vec()), Value::Integer(1_800_000_000_001), Value::Text("receivable_receipt_create".into()), Value::Text("receipt".into()), Value::Blob(first_id.to_vec())],
            vec![Value::Blob(second_event.to_vec()), Value::Integer(1_800_000_000_002), Value::Text("receivable_receipt_create".into()), Value::Text("receipt".into()), Value::Blob(second_id.to_vec())],
        ]
    );
    assert_eq!(
        rows(&reopened, "SELECT r.amount_cents-SUM(x.amount_cents) FROM main.receivables r JOIN main.receipts x ON x.receivable_id=r.id GROUP BY r.id"),
        vec![vec![Value::Integer(0)]]
    );
    assert_eq!(
        rows(&reopened, "SELECT count(*) FROM main.movements"),
        vec![vec![Value::Integer(0)]]
    );
}

#[test]
fn rcv006_rejects_invalid_missing_and_excess_without_partial_writes() {
    let (_directory, path) = setup("reject.sqlite", 3000);
    let result = write_receipt(
        &path,
        RECEIVABLE_ID,
        [21_u8; 16],
        receipt(ACCOUNT_A, 0, Context::Home),
        [22_u8; 16],
        1_800_000_000_010,
    );
    assert!(matches!(
        result,
        Err(TransactionError::OperationFailed(
            ReceivableReceiptStoreError::InvalidAmount
        ))
    ));
    let result = write_receipt(
        &path,
        [23_u8; 16],
        [24_u8; 16],
        receipt(ACCOUNT_A, 1, Context::Home),
        [25_u8; 16],
        1_800_000_000_011,
    );
    assert!(matches!(
        result,
        Err(TransactionError::OperationFailed(
            ReceivableReceiptStoreError::ReceivableNotFound
        ))
    ));
    let result = write_receipt(
        &path,
        RECEIVABLE_ID,
        [26_u8; 16],
        receipt(ACCOUNT_A, 3001, Context::Home),
        [27_u8; 16],
        1_800_000_000_012,
    );
    assert!(matches!(
        result,
        Err(TransactionError::OperationFailed(
            ReceivableReceiptStoreError::AmountExceedsDue
        ))
    ));
    write_receipt(
        &path,
        RECEIVABLE_ID,
        [28_u8; 16],
        receipt(ACCOUNT_A, 1000, Context::Home),
        [29_u8; 16],
        1_800_000_000_013,
    )
    .expect("VALID_PARTIAL_RECEIPT_FAILED");
    let result = write_receipt(
        &path,
        RECEIVABLE_ID,
        [30_u8; 16],
        receipt(ACCOUNT_B, 2001, Context::Factory),
        [31_u8; 16],
        1_800_000_000_014,
    );
    assert!(matches!(
        result,
        Err(TransactionError::OperationFailed(
            ReceivableReceiptStoreError::AmountExceedsDue
        ))
    ));
    let reopened = open_database(&path).expect("REJECT_REOPEN_FAILED");
    assert_eq!(
        rows(&reopened, "SELECT count(*) FROM main.receipts"),
        vec![vec![Value::Integer(1)]]
    );
    assert_eq!(
        rows(&reopened, "SELECT count(*) FROM main.audit_event"),
        vec![vec![Value::Integer(1)]]
    );
}

#[test]
fn rcv006_audit_failure_rolls_back_and_main_schema_wins_over_temp_shadowing() {
    let (_directory, path) = setup("atomic.sqlite", 3000);
    let connection = open_database(&path).expect("DATABASE_OPEN_FAILED");
    connection.execute_batch(
        "CREATE TRIGGER main.rcv006_fail_audit BEFORE INSERT ON main.audit_event WHEN NEW.operation='receivable_receipt_create' BEGIN SELECT RAISE(ABORT, 'synthetic audit fault'); END;",
    ).expect("AUDIT_TRIGGER_CREATE_FAILED");
    drop(connection);
    let result = write_receipt(
        &path,
        RECEIVABLE_ID,
        [41_u8; 16],
        receipt(ACCOUNT_A, 1000, Context::Home),
        [42_u8; 16],
        1_800_000_000_020,
    );
    assert!(matches!(
        result,
        Err(TransactionError::OperationFailed(
            ReceivableReceiptStoreError::StoreFailed
        ))
    ));
    let reopened = open_database(&path).expect("AUDIT_FAILURE_REOPEN_FAILED");
    assert!(rows(&reopened, "SELECT * FROM main.receipts").is_empty());
    assert!(rows(&reopened, "SELECT * FROM main.audit_event").is_empty());
    drop(reopened);

    let connection = open_database(&path).expect("DATABASE_OPEN_FAILED");
    connection.execute_batch(
        "DROP TRIGGER main.rcv006_fail_audit; CREATE TEMP TABLE receipts (id BLOB, receivable_id BLOB, account_id BLOB, amount_cents INTEGER, context TEXT); CREATE TEMP TABLE receivables (id BLOB, amount_cents INTEGER); CREATE TEMP TABLE audit_event (event_id BLOB, occurred_at_ms INTEGER, operation TEXT, entity TEXT, entity_id BLOB);",
    ).expect("TEMP_SHADOW_TABLES_CREATE_FAILED");
    let (connection, ()) = execute_write_transaction(connection, |transaction| {
        persist_receivable_receipt_in_transaction(
            transaction,
            RECEIVABLE_ID,
            [51_u8; 16],
            &receipt(ACCOUNT_A, 1000, Context::Home),
            [52_u8; 16],
            1_800_000_000_030,
        )
    })
    .expect("SHADOWED_RECEIPT_WRITE_FAILED");
    assert_eq!(
        rows(&connection, "SELECT count(*) FROM main.receipts"),
        vec![vec![Value::Integer(1)]]
    );
    assert_eq!(
        rows(&connection, "SELECT count(*) FROM main.audit_event"),
        vec![vec![Value::Integer(1)]]
    );
    assert_eq!(
        rows(&connection, "SELECT count(*) FROM temp.receipts"),
        vec![vec![Value::Integer(0)]]
    );
    assert_eq!(
        rows(&connection, "SELECT count(*) FROM temp.audit_event"),
        vec![vec![Value::Integer(0)]]
    );
}

#[test]
fn rcv006_ignored_receipt_insert_does_not_create_audit_event() {
    let (_directory, path) = setup("ignored-insert.sqlite", 3000);
    let connection = open_database(&path).expect("DATABASE_OPEN_FAILED");
    connection
        .execute_batch(
            "CREATE TRIGGER main.rcv006_ignore_receipt BEFORE INSERT ON main.receipts BEGIN SELECT RAISE(IGNORE); END;",
        )
        .expect("IGNORE_TRIGGER_CREATE_FAILED");
    drop(connection);

    let result = write_receipt(
        &path,
        RECEIVABLE_ID,
        [61_u8; 16],
        receipt(ACCOUNT_A, 1000, Context::Home),
        [62_u8; 16],
        1_800_000_000_040,
    );
    assert!(matches!(
        result,
        Err(TransactionError::OperationFailed(
            ReceivableReceiptStoreError::StoreFailed
        ))
    ));

    let reopened = open_database(&path).expect("IGNORED_INSERT_REOPEN_FAILED");
    assert!(rows(&reopened, "SELECT * FROM main.receipts").is_empty());
    assert!(rows(&reopened, "SELECT * FROM main.audit_event").is_empty());
}

#[test]
fn rcv006_failed_migration_rolls_back_ddl_and_version_then_retries() {
    let directory = TempDirectory::new();
    let path = directory.path().join("migration-rollback.sqlite");
    let connection = migrate_to_v12(&path);
    seed(&connection, 3000);
    connection.execute_batch(
        "CREATE TRIGGER main.rcv006_fail_ledger BEFORE INSERT ON main.schema_migration WHEN NEW.version=13 BEGIN SELECT RAISE(ABORT, 'synthetic ledger fault'); END;",
    ).expect("LEDGER_TRIGGER_CREATE_FAILED");
    drop(connection);
    let connection = open_database(&path).expect("DATABASE_OPEN_FAILED");
    let failed = execute_versioned_migration(
        connection,
        RECEIPTS_MIGRATION_VERSION,
        RECEIPTS_MIGRATION_SQL,
    );
    assert!(matches!(failed, Err(MigrationError::ApplyFailed)));
    let connection = open_database(&path).expect("MIGRATION_FAILURE_REOPEN_FAILED");
    assert_eq!(schema_version(&connection, 12).unwrap(), 12);
    assert!(rows(
        &connection,
        "SELECT name FROM main.sqlite_schema WHERE type='table' AND name='receipts'"
    )
    .is_empty());
    assert_eq!(
        rows(
            &connection,
            "SELECT id,sale_id,amount_cents,customer_id,due_at_ms FROM main.receivables"
        ),
        vec![vec![
            Value::Blob(RECEIVABLE_ID.to_vec()),
            Value::Blob(SALE_ID.to_vec()),
            Value::Integer(3000),
            Value::Blob(CUSTOMER_ID.to_vec()),
            Value::Integer(1_800_000_000_000)
        ]]
    );
    connection
        .execute_batch("DROP TRIGGER main.rcv006_fail_ledger")
        .expect("LEDGER_TRIGGER_DROP_FAILED");
    drop(connection);
    let connection = open_database(&path).expect("DATABASE_OPEN_FAILED");
    let upgraded = execute_versioned_migration(
        connection,
        RECEIPTS_MIGRATION_VERSION,
        RECEIPTS_MIGRATION_SQL,
    )
    .expect("MIGRATION_RETRY_FAILED");
    assert_eq!(schema_version(&upgraded, 13).unwrap(), 13);
    assert!(rows(&upgraded, "PRAGMA main.foreign_key_check").is_empty());
}
