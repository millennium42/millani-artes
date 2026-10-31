use super::TempDirectory;
use millani_artes::application::{
    list_accounts::list_accounts,
    list_movements::list_movements,
    record_receivable_receipt::{
        record_receivable_receipt, ReceivableReceiptRecordingStore, ReceivableReceiptWrite,
        RecordReceivableReceiptError,
    },
};
use millani_artes::database::{
    execute_versioned_migration, execute_write_transaction, open_database, schema_version,
    MigrationError, TransactionError, ACCOUNTS_MIGRATION_SQL, AUDIT_EVENT_MIGRATION_SQL,
    CATEGORIES_MIGRATION_SQL, CUSTOMERS_MIGRATION_SQL, MOVEMENTS_MIGRATION_SQL,
    MOVEMENT_CATEGORIES_MIGRATION_SQL, PRODUCTS_MIGRATION_SQL, RECEIPTS_MIGRATION_SQL,
    RECEIPTS_MIGRATION_VERSION, RECEIVABLES_MIGRATION_SQL, RECURRENCES_MIGRATION_SQL,
    SALES_MIGRATION_SQL, SALE_ITEMS_MIGRATION_SQL,
};
use millani_artes::domain::ports::{Clock, UuidGenerator};
use millani_artes::domain::{
    Context, Movement, MovementKind, Receivable, ReceivableReceipt, SalePending,
};
use millani_artes::infrastructure::receivables::{
    persist_receivable_receipt_in_transaction, AdapterSqliteReceivableReceiptStore,
    ReceivableReceiptStoreError,
};
use millani_artes::infrastructure::{
    accounts::AdapterSqliteAccountStore, movements::AdapterSqliteMovementStore,
};
use rusqlite::{params, types::Value, Connection};
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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

struct ReceiptIds {
    values: [[u8; 16]; 8],
    calls: usize,
}

impl ReceiptIds {
    fn new() -> Self {
        Self {
            values: [
                [0x31; 16], [0x32; 16], [0x33; 16], [0x34; 16], [0x35; 16], [0x36; 16], [0x37; 16],
                [0x38; 16],
            ],
            calls: 0,
        }
    }
}

impl UuidGenerator for ReceiptIds {
    type Error = &'static str;

    fn generate(&mut self) -> Result<[u8; 16], Self::Error> {
        let id = self
            .values
            .get(self.calls)
            .copied()
            .ok_or("UUID_SOURCE_EXHAUSTED")?;
        self.calls += 1;
        Ok(id)
    }
}

struct ReceiptClock(SystemTime);

impl Clock for ReceiptClock {
    fn now(&self) -> SystemTime {
        self.0
    }
}

#[test]
fn rcv007_records_receipt_entry_and_both_audits_together() {
    let (_directory, path) = setup("rcv007-success.sqlite", 3000);
    let mut ids = ReceiptIds::new();
    let clock = ReceiptClock(UNIX_EPOCH + Duration::from_millis(1_800_000_000_099));
    let mut store = AdapterSqliteReceivableReceiptStore::new(
        open_database(&path).expect("DATABASE_OPEN_FAILED"),
    );
    let movement = record_receivable_receipt(
        RECEIVABLE_ID,
        receipt(ACCOUNT_B, 1000, Context::Factory),
        &mut ids,
        &clock,
        &mut store,
    )
    .expect("RECEIPT_ENTRY_RECORD_FAILED");

    assert_eq!(movement.id, [0x33; 16]);
    assert_eq!(movement.amount_cents, 1000);
    assert!(matches!(
        &movement.kind,
        MovementKind::Entry { account_id } if *account_id == ACCOUNT_B
    ));
    assert!(matches!(&movement.context, Context::Factory));
    assert_eq!(ids.calls, 4);

    let reopened = open_database(&path).expect("RCV007_REOPEN_FAILED");
    assert_eq!(
        rows(
            &reopened,
            "SELECT id,receivable_id,account_id,amount_cents,context FROM main.receipts"
        ),
        vec![vec![
            Value::Blob(vec![0x31; 16]),
            Value::Blob(RECEIVABLE_ID.to_vec()),
            Value::Blob(ACCOUNT_B.to_vec()),
            Value::Integer(1000),
            Value::Text("factory".into()),
        ]]
    );
    assert_eq!(
        rows(
            &reopened,
            "SELECT id,kind,amount_cents,context,account_id FROM main.movements"
        ),
        vec![vec![
            Value::Blob(vec![0x33; 16]),
            Value::Text("entry".into()),
            Value::Integer(1000),
            Value::Text("factory".into()),
            Value::Blob(ACCOUNT_B.to_vec()),
        ]]
    );
    assert_eq!(
        rows(&reopened, "SELECT event_id,occurred_at_ms,operation,entity,entity_id FROM main.audit_event ORDER BY event_id"),
        vec![
            vec![
                Value::Blob(vec![0x32; 16]),
                Value::Integer(1_800_000_000_099),
                Value::Text("receivable_receipt_create".into()),
                Value::Text("receipt".into()),
                Value::Blob(vec![0x31; 16]),
            ],
            vec![
                Value::Blob(vec![0x34; 16]),
                Value::Integer(1_800_000_000_099),
                Value::Text("movement_entry_create".into()),
                Value::Text("movement".into()),
                Value::Blob(vec![0x33; 16]),
            ],
        ]
    );
}

#[test]
fn rcv008_receipt_increases_only_selected_account_balance_after_reopen() {
    let (_directory, path) = setup("rcv008-account-balance.sqlite", 3000);
    let before_accounts = list_accounts(&AdapterSqliteAccountStore::new(
        open_database(&path).expect("RCV008_BASELINE_DATABASE_OPEN_FAILED"),
    ))
    .expect("RCV008_BASELINE_ACCOUNTS_READ_FAILED");
    let before_connection = open_database(&path).expect("RCV008_BASELINE_MOVEMENTS_OPEN_FAILED");
    before_connection
        .pragma_update(None, "query_only", true)
        .expect("RCV008_BASELINE_QUERY_ONLY_FAILED");
    let before_movements = list_movements(&AdapterSqliteMovementStore::new(before_connection))
        .expect("RCV008_BASELINE_MOVEMENTS_READ_FAILED");
    assert!(before_movements.is_empty());

    let before_a = before_accounts
        .iter()
        .find(|account| account.id == ACCOUNT_A)
        .expect("RCV008_ACCOUNT_A_MISSING")
        .balance_cents(&before_movements)
        .expect("RCV008_ACCOUNT_A_BASELINE_BALANCE_FAILED");
    let before_b = before_accounts
        .iter()
        .find(|account| account.id == ACCOUNT_B)
        .expect("RCV008_ACCOUNT_B_MISSING")
        .balance_cents(&before_movements)
        .expect("RCV008_ACCOUNT_B_BASELINE_BALANCE_FAILED");
    assert_eq!(before_a, 100);
    assert_eq!(before_b, 200);

    let mut ids = ReceiptIds::new();
    let clock = ReceiptClock(UNIX_EPOCH + Duration::from_millis(1_800_000_000_099));
    let mut store = AdapterSqliteReceivableReceiptStore::new(
        open_database(&path).expect("RCV008_DATABASE_OPEN_FAILED"),
    );
    let recorded = record_receivable_receipt(
        RECEIVABLE_ID,
        receipt(ACCOUNT_B, 1000, Context::Factory),
        &mut ids,
        &clock,
        &mut store,
    )
    .expect("RCV008_RECEIPT_RECORD_FAILED");
    drop(store);

    let after_accounts = list_accounts(&AdapterSqliteAccountStore::new(
        open_database(&path).expect("RCV008_REOPEN_ACCOUNTS_FAILED"),
    ))
    .expect("RCV008_REOPEN_ACCOUNTS_READ_FAILED");
    let after_connection = open_database(&path).expect("RCV008_REOPEN_MOVEMENTS_FAILED");
    after_connection
        .pragma_update(None, "query_only", true)
        .expect("RCV008_REOPEN_QUERY_ONLY_FAILED");
    let after_movements = list_movements(&AdapterSqliteMovementStore::new(after_connection))
        .expect("RCV008_REOPEN_MOVEMENTS_READ_FAILED");

    assert_eq!(after_movements.len(), 1);
    assert_eq!(after_movements[0].id, recorded.id);
    assert_eq!(after_movements[0].amount_cents, 1000);
    assert!(matches!(
        &after_movements[0].kind,
        MovementKind::Entry { account_id } if *account_id == ACCOUNT_B
    ));
    assert!(matches!(&after_movements[0].context, Context::Factory));

    let after_a = after_accounts
        .iter()
        .find(|account| account.id == ACCOUNT_A)
        .expect("RCV008_REOPEN_ACCOUNT_A_MISSING")
        .balance_cents(&after_movements)
        .expect("RCV008_ACCOUNT_A_FINAL_BALANCE_FAILED");
    let after_b = after_accounts
        .iter()
        .find(|account| account.id == ACCOUNT_B)
        .expect("RCV008_REOPEN_ACCOUNT_B_MISSING")
        .balance_cents(&after_movements)
        .expect("RCV008_ACCOUNT_B_FINAL_BALANCE_FAILED");

    assert_eq!(after_a, before_a);
    assert_eq!(after_b - before_b, 1000);
    assert_eq!(after_b, 1200);
}

#[test]
fn rcv007_receipt_insert_failure_rolls_back_without_partial_writes() {
    let (_directory, path) = setup("rcv007-receipt-failure.sqlite", 3000);
    let connection = open_database(&path).expect("DATABASE_OPEN_FAILED");
    connection
        .execute_batch(
            "CREATE TRIGGER main.rcv007_fail_receipt BEFORE INSERT ON main.receipts BEGIN SELECT RAISE(ABORT, 'SYNTHETIC_RCV007_RECEIPT_FAILURE'); END;",
        )
        .expect("RCV007_TRIGGER_CREATE_FAILED");
    drop(connection);

    let mut ids = ReceiptIds::new();
    let clock = ReceiptClock(UNIX_EPOCH + Duration::from_millis(1_800_000_000_100));
    let mut store = AdapterSqliteReceivableReceiptStore::new(
        open_database(&path).expect("DATABASE_OPEN_FAILED"),
    );
    let error = record_receivable_receipt(
        RECEIVABLE_ID,
        receipt(ACCOUNT_A, 1000, Context::Home),
        &mut ids,
        &clock,
        &mut store,
    )
    .err()
    .expect("receipt insert failure must cancel the operation");
    assert_eq!(error, RecordReceivableReceiptError::StoreFailed);
    assert!(!error.to_string().contains("SYNTHETIC_RCV007"));

    let reopened = open_database(&path).expect("RCV007_RECEIPT_FAILURE_REOPEN_FAILED");
    assert!(rows(&reopened, "SELECT * FROM main.receipts").is_empty());
    assert!(rows(&reopened, "SELECT * FROM main.movements").is_empty());
    assert!(rows(&reopened, "SELECT * FROM main.audit_event").is_empty());
}

#[test]
fn rcv007_entry_audit_failure_rolls_back_receipt_entry_and_events() {
    let (_directory, path) = setup("rcv007-audit-failure.sqlite", 3000);
    let connection = open_database(&path).expect("DATABASE_OPEN_FAILED");
    connection
        .execute_batch(
            "CREATE TRIGGER main.rcv007_fail_entry_audit BEFORE INSERT ON main.audit_event WHEN NEW.operation='movement_entry_create' BEGIN SELECT RAISE(ABORT, 'SYNTHETIC_RCV007_AUDIT_FAILURE'); END;",
        )
        .expect("RCV007_TRIGGER_CREATE_FAILED");
    drop(connection);

    let mut ids = ReceiptIds::new();
    let clock = ReceiptClock(UNIX_EPOCH + Duration::from_millis(1_800_000_000_100));
    let mut store = AdapterSqliteReceivableReceiptStore::new(
        open_database(&path).expect("DATABASE_OPEN_FAILED"),
    );
    let error = record_receivable_receipt(
        RECEIVABLE_ID,
        receipt(ACCOUNT_A, 1000, Context::Home),
        &mut ids,
        &clock,
        &mut store,
    )
    .err()
    .expect("entry audit failure must cancel the operation");
    assert_eq!(error, RecordReceivableReceiptError::StoreFailed);
    assert!(!error.to_string().contains("SYNTHETIC_RCV007"));

    let reopened = open_database(&path).expect("RCV007_FAILURE_REOPEN_FAILED");
    assert!(rows(&reopened, "SELECT * FROM main.receipts").is_empty());
    assert!(rows(&reopened, "SELECT * FROM main.movements").is_empty());
    assert!(rows(&reopened, "SELECT * FROM main.audit_event").is_empty());
}

#[test]
fn rcv009_rejects_over_remaining_due_after_partial_receipt_without_new_writes() {
    let (_directory, path) = setup("rcv009-overdue.sqlite", 3000);
    let mut ids = ReceiptIds::new();
    let clock = ReceiptClock(UNIX_EPOCH + Duration::from_millis(1_800_000_000_111));
    let mut store = AdapterSqliteReceivableReceiptStore::new(
        open_database(&path).expect("DATABASE_OPEN_FAILED"),
    );

    record_receivable_receipt(
        RECEIVABLE_ID,
        receipt(ACCOUNT_B, 1000, Context::Factory),
        &mut ids,
        &clock,
        &mut store,
    )
    .expect("RCV009_INITIAL_PARTIAL_RECEIPT_FAILED");

    let before_connection = open_database(&path).expect("RCV009_BEFORE_REOPEN_FAILED");
    let before_receipts = rows(
        &before_connection,
        "SELECT * FROM main.receipts ORDER BY id",
    );
    let before_movements = rows(
        &before_connection,
        "SELECT * FROM main.movements ORDER BY id",
    );
    let before_audits = rows(
        &before_connection,
        "SELECT * FROM main.audit_event ORDER BY event_id",
    );
    let principal_cents = before_connection
        .query_row(
            "SELECT amount_cents FROM main.receivables WHERE id = ?1",
            params![RECEIVABLE_ID],
            |row| row.get::<_, i64>(0),
        )
        .expect("RCV009_PRINCIPAL_READ_FAILED");
    assert_eq!(principal_cents, 3000);
    assert_eq!(before_receipts.len(), 1);
    assert_eq!(before_receipts[0][3], Value::Integer(1000));
    assert_eq!(principal_cents - 1000, 2000);
    assert_eq!(before_movements.len(), 1);
    assert_eq!(before_audits.len(), 2);
    drop(before_connection);

    let error = record_receivable_receipt(
        RECEIVABLE_ID,
        receipt(ACCOUNT_A, 2001, Context::Factory),
        &mut ids,
        &clock,
        &mut store,
    )
    .err()
    .expect("RCV009_OVERDUE_RECEIPT_WAS_ACCEPTED");
    assert_eq!(error, RecordReceivableReceiptError::AmountExceedsDue);

    let reopened = open_database(&path).expect("RCV009_AFTER_REOPEN_FAILED");
    assert_eq!(
        rows(&reopened, "SELECT * FROM main.receipts ORDER BY id",),
        before_receipts
    );
    assert_eq!(
        rows(&reopened, "SELECT * FROM main.movements ORDER BY id",),
        before_movements
    );
    assert_eq!(
        rows(
            &reopened,
            "SELECT * FROM main.audit_event ORDER BY event_id",
        ),
        before_audits
    );
}

#[test]
fn rcv010_full_receipt_closes_projected_due_after_reopen() {
    let (_directory, path) = setup("rcv010-full-receipt.sqlite", 3000);
    let mut ids = ReceiptIds::new();
    let clock = ReceiptClock(UNIX_EPOCH + Duration::from_millis(1_800_000_000_111));
    let receivable = Receivable {
        id: RECEIVABLE_ID,
        sale_id: SALE_ID,
        pending: SalePending {
            amount_cents: 3000,
            customer_id: CUSTOMER_ID,
            due_at_ms: 1_800_000_000_000,
        },
    };
    let mut store = AdapterSqliteReceivableReceiptStore::new(
        open_database(&path).expect("DATABASE_OPEN_FAILED"),
    );

    record_receivable_receipt(
        RECEIVABLE_ID,
        receipt(ACCOUNT_B, 1000, Context::Factory),
        &mut ids,
        &clock,
        &mut store,
    )
    .expect("RCV010_PARTIAL_RECEIPT_FAILED");

    let partial = open_database(&path).expect("RCV010_PARTIAL_REOPEN_FAILED");
    let (principal_cents, applied_cents) = partial
        .query_row(
            "SELECT receivable.amount_cents, COALESCE(SUM(receipt.amount_cents), 0) FROM main.receivables AS receivable LEFT JOIN main.receipts AS receipt ON receipt.receivable_id = receivable.id WHERE receivable.id = ?1 GROUP BY receivable.id, receivable.amount_cents",
            [RECEIVABLE_ID.as_slice()],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
        )
        .expect("RCV010_PARTIAL_DUE_READ_FAILED");
    assert_eq!(principal_cents, 3000);
    assert_eq!(applied_cents, 1000);
    assert_eq!(receivable.remaining_due_cents(applied_cents), Ok(2000));
    drop(partial);

    record_receivable_receipt(
        RECEIVABLE_ID,
        receipt(ACCOUNT_B, 2000, Context::Factory),
        &mut ids,
        &clock,
        &mut store,
    )
    .expect("RCV010_FINAL_RECEIPT_FAILED");
    drop(store);

    let reopened = open_database(&path).expect("RCV010_FINAL_REOPEN_FAILED");
    let (principal_cents, applied_cents) = reopened
        .query_row(
            "SELECT receivable.amount_cents, COALESCE(SUM(receipt.amount_cents), 0) FROM main.receivables AS receivable LEFT JOIN main.receipts AS receipt ON receipt.receivable_id = receivable.id WHERE receivable.id = ?1 GROUP BY receivable.id, receivable.amount_cents",
            [RECEIVABLE_ID.as_slice()],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
        )
        .expect("RCV010_FINAL_DUE_READ_FAILED");

    assert_eq!(principal_cents, 3000);
    assert_eq!(applied_cents, 3000);
    assert_eq!(receivable.remaining_due_cents(applied_cents), Ok(0));
    assert_eq!(
        rows(
            &reopened,
            "SELECT amount_cents FROM main.receipts ORDER BY amount_cents",
        ),
        vec![vec![Value::Integer(1000)], vec![Value::Integer(2000)],]
    );
    assert_eq!(
        rows(
            &reopened,
            "SELECT amount_cents FROM main.movements ORDER BY amount_cents",
        ),
        vec![vec![Value::Integer(1000)], vec![Value::Integer(2000)],]
    );
    assert_eq!(
        rows(&reopened, "SELECT event_id FROM main.audit_event").len(),
        4
    );
}

#[test]
fn rcv007_missing_or_excess_receivable_creates_no_receipt_entry_or_audit() {
    for (name, receivable_id, amount_cents, expected_error) in [
        (
            "rcv007-missing.sqlite",
            [0xee; 16],
            1000,
            RecordReceivableReceiptError::ReceivableNotFound,
        ),
        (
            "rcv007-excess.sqlite",
            RECEIVABLE_ID,
            3001,
            RecordReceivableReceiptError::AmountExceedsDue,
        ),
    ] {
        let (_directory, path) = setup(name, 3000);
        let mut ids = ReceiptIds::new();
        let clock = ReceiptClock(UNIX_EPOCH + Duration::from_millis(1_800_000_000_101));
        let mut store = AdapterSqliteReceivableReceiptStore::new(
            open_database(&path).expect("DATABASE_OPEN_FAILED"),
        );
        let error = record_receivable_receipt(
            receivable_id,
            receipt(ACCOUNT_A, amount_cents, Context::Home),
            &mut ids,
            &clock,
            &mut store,
        )
        .err()
        .expect("invalid receivable operation must fail");
        assert_eq!(error, expected_error);

        let reopened = open_database(&path).expect("RCV007_REJECTION_REOPEN_FAILED");
        assert!(rows(&reopened, "SELECT * FROM main.receipts").is_empty());
        assert!(rows(&reopened, "SELECT * FROM main.movements").is_empty());
        assert!(rows(&reopened, "SELECT * FROM main.audit_event").is_empty());
    }
}

#[test]
fn rcv007_rejects_inconsistent_writes_and_nonpositive_amount_before_writes() {
    let cases = [
        (
            "rcv007-nonpositive-amount.sqlite",
            receipt(ACCOUNT_A, 0, Context::Home),
            MovementKind::Entry {
                account_id: ACCOUNT_A,
            },
            0,
            Context::Home,
        ),
        (
            "rcv007-account-mismatch.sqlite",
            receipt(ACCOUNT_A, 1000, Context::Home),
            MovementKind::Entry {
                account_id: ACCOUNT_B,
            },
            1000,
            Context::Home,
        ),
        (
            "rcv007-amount-mismatch.sqlite",
            receipt(ACCOUNT_A, 1000, Context::Home),
            MovementKind::Entry {
                account_id: ACCOUNT_A,
            },
            1001,
            Context::Home,
        ),
        (
            "rcv007-context-mismatch.sqlite",
            receipt(ACCOUNT_A, 1000, Context::Factory),
            MovementKind::Entry {
                account_id: ACCOUNT_A,
            },
            1000,
            Context::Home,
        ),
        (
            "rcv007-kind-mismatch.sqlite",
            receipt(ACCOUNT_A, 1000, Context::Home),
            MovementKind::Exit {
                account_id: ACCOUNT_A,
            },
            1000,
            Context::Home,
        ),
    ];

    for (name, receipt, kind, amount_cents, context) in cases {
        let (_directory, path) = setup(name, 3000);
        let movement = Movement {
            id: [0x33; 16],
            kind,
            amount_cents,
            context,
        };
        let mut store = AdapterSqliteReceivableReceiptStore::new(
            open_database(&path).expect("DATABASE_OPEN_FAILED"),
        );
        for _ in 0..2 {
            let error = store
                .persist_receipt_and_entry(ReceivableReceiptWrite {
                    receivable_id: RECEIVABLE_ID,
                    receipt_id: [0x31; 16],
                    receipt: &receipt,
                    receipt_event_id: [0x32; 16],
                    movement: &movement,
                    movement_event_id: [0x34; 16],
                    occurred_at_ms: 1_800_000_000_102,
                })
                .expect_err("receipt and entry must match");
            assert_eq!(error, RecordReceivableReceiptError::InvalidInput);
        }

        drop(store);
        let reopened = open_database(&path).expect("RCV007_MISMATCH_REOPEN_FAILED");
        assert!(rows(&reopened, "SELECT * FROM main.receipts").is_empty());
        assert!(rows(&reopened, "SELECT * FROM main.movements").is_empty());
        assert!(rows(&reopened, "SELECT * FROM main.audit_event").is_empty());
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
