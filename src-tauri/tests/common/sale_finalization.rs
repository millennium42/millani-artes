use super::TempDirectory;
use millani_artes::application::add_free_item::{add_free_item, FreeItemInput};
use millani_artes::application::add_product_item::SaleError;
use millani_artes::application::finalize_sale::{
    finalize_sale, FinalizeSaleInput, FinalizeSaleWrite, FinalizedSale, PendingSaleTerms,
    SaleFinalizationStore, SalePaymentWrite,
};
use millani_artes::application::list_accounts::list_accounts;
use millani_artes::application::list_movements::list_movements;
use millani_artes::database::{
    execute_versioned_migration, open_database, schema_version, ACCOUNTS_MIGRATION_SQL,
    AUDIT_EVENT_MIGRATION_SQL, CATEGORIES_MIGRATION_SQL, CUSTOMERS_MIGRATION_SQL,
    MOVEMENTS_MIGRATION_SQL, MOVEMENT_CATEGORIES_MIGRATION_SQL, PRODUCTS_MIGRATION_SQL,
    RECEIVABLES_MIGRATION_SQL, RECURRENCES_MIGRATION_SQL, SALES_MIGRATION_SQL,
    SALE_ITEMS_MIGRATION_SQL,
};
use millani_artes::domain::ports::{Clock, UuidGenerator};
use millani_artes::domain::{Context, DiscountInput, Movement, MovementKind, SalePayment};
use millani_artes::infrastructure::{
    accounts::AdapterSqliteAccountStore, movements::AdapterSqliteMovementStore,
    sales::AdapterSqliteSaleStore,
};
use rusqlite::{params, types::Value, Connection};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const ACCOUNT: [u8; 16] = [0x51; 16];
const CUSTOMER: [u8; 16] = [0x61; 16];
const DUE_AT_MS: i64 = 1_800_000_000_000;
const OCCURRED_AT_MS: i64 = 1_765_000_000_123;

struct Sequence(u8);

impl UuidGenerator for Sequence {
    type Error = ();

    fn generate(&mut self) -> Result<[u8; 16], ()> {
        let id = self.0;
        self.0 = self.0.checked_add(1).ok_or(())?;
        Ok([id; 16])
    }
}

struct FixedClock;

impl Clock for FixedClock {
    fn now(&self) -> SystemTime {
        UNIX_EPOCH + Duration::from_millis(OCCURRED_AT_MS as u64)
    }
}

fn setup() -> (TempDirectory, PathBuf) {
    let directory = TempDirectory::new();
    let path = directory.path().join("sal020.sqlite");
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
    ] {
        connection = execute_versioned_migration(connection, version, sql).unwrap();
    }
    drop(connection);
    (directory, path)
}

fn insert_account(connection: &Connection, id: [u8; 16], balance_cents: i64) {
    connection
        .execute(
            "INSERT INTO main.accounts(id,name,account_type,initial_balance_cents) VALUES(?1,'Conta sintética','cash',?2)",
            params![id.as_slice(), balance_cents],
        )
        .unwrap();
}

fn insert_customer(connection: &Connection, id: [u8; 16]) {
    connection
        .execute(
            "INSERT INTO main.customers(id,name,phone) VALUES(?1,'Cliente sintético',NULL)",
            params![id.as_slice()],
        )
        .unwrap();
}

fn add_sale(path: &Path, sale_id: [u8; 16], total_cents: i64, seed: u8) {
    let connection = open_database(path).unwrap();
    let exists: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM main.sales WHERE id=?1)",
            params![sale_id.as_slice()],
            |row| row.get(0),
        )
        .unwrap();
    if !exists {
        connection
            .execute(
                "INSERT INTO main.sales(id,status) VALUES(?1,'open')",
                params![sale_id.as_slice()],
            )
            .unwrap();
    }
    drop(connection);
    let mut store = AdapterSqliteSaleStore::new(open_database(path).unwrap());
    add_free_item(
        FreeItemInput {
            sale_id: Some(sale_id),
            description: "Item sintético SAL-020".into(),
            quantity: 1,
            unit_price_cents: total_cents,
            discount: DiscountInput::AmountCents(0),
        },
        &mut Sequence(seed),
        &FixedClock,
        &mut store,
    )
    .unwrap();
}

fn add_empty_sale(path: &Path, sale_id: [u8; 16], status: &str) {
    open_database(path)
        .unwrap()
        .execute(
            "INSERT INTO main.sales(id,status) VALUES(?1,?2)",
            params![sale_id.as_slice(), status],
        )
        .unwrap();
}

fn payment(account_id: [u8; 16], amount_cents: i64, context: Context) -> SalePayment {
    SalePayment {
        account_id,
        amount_cents,
        context,
    }
}

fn input(
    sale_id: [u8; 16],
    payments: Vec<SalePayment>,
    pending: Option<PendingSaleTerms>,
) -> FinalizeSaleInput {
    FinalizeSaleInput {
        sale_id,
        payments,
        pending,
    }
}

fn run_finalization(
    store: &mut AdapterSqliteSaleStore,
    value: FinalizeSaleInput,
) -> Result<FinalizedSale, SaleError> {
    run_finalization_with_seed(store, value, 0x70)
}

fn run_finalization_with_seed(
    store: &mut AdapterSqliteSaleStore,
    value: FinalizeSaleInput,
    seed: u8,
) -> Result<FinalizedSale, SaleError> {
    finalize_sale(value, &mut Sequence(seed), &FixedClock, store)
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
        "accounts",
        "sales",
        "sale_items",
        "movements",
        "receivables",
        "audit_event",
    ]
    .into_iter()
    .map(|table| {
        rows(
            connection,
            &format!("SELECT * FROM main.{table} ORDER BY 1,2"),
        )
    })
    .collect()
}

fn assert_healthy(connection: &Connection) {
    assert!(connection.is_autocommit());
    assert_eq!(schema_version(connection, 12).unwrap(), 12);
    assert_eq!(
        rows(connection, "PRAGMA main.integrity_check"),
        vec![vec![Value::Text("ok".into())]]
    );
    assert!(rows(connection, "PRAGMA main.foreign_key_check").is_empty());
}

#[test]
fn sal020_finalizes_per_payment_context_atomically_and_uses_main_tables() {
    let (_directory, path) = setup();
    let connection = open_database(&path).unwrap();
    insert_account(&connection, ACCOUNT, 12_500);
    insert_customer(&connection, CUSTOMER);
    drop(connection);
    let sale_id = [0x41; 16];
    add_sale(&path, sale_id, 3000, 0x31);

    let connection = open_database(&path).unwrap();
    connection
        .execute_batch(
            "CREATE TEMP TABLE sales(id BLOB,status TEXT);
             CREATE TEMP TABLE sale_items(id BLOB,sale_id BLOB);
             CREATE TEMP TABLE audit_event(event_id BLOB,occurred_at_ms INTEGER,operation TEXT,entity TEXT,entity_id BLOB);
             INSERT INTO temp.sales VALUES(X'41414141414141414141414141414141','finalized');",
        )
        .unwrap();
    let mut store = AdapterSqliteSaleStore::new(connection);
    let result = run_finalization(
        &mut store,
        input(
            sale_id,
            vec![
                payment(ACCOUNT, 500, Context::Home),
                payment(ACCOUNT, 1000, Context::Factory),
            ],
            Some(PendingSaleTerms {
                customer_id: CUSTOMER,
                due_at_ms: DUE_AT_MS,
            }),
        ),
    )
    .unwrap();
    assert_eq!(
        [result.total_cents, result.paid_cents, result.pending_cents],
        [3000, 1500, 1500]
    );
    drop(store);

    let reopened = open_database(&path).unwrap();
    assert_eq!(
        rows(
            &reopened,
            "SELECT id,kind,amount_cents,context,account_id FROM main.movements ORDER BY rowid"
        ),
        vec![
            vec![
                Value::Blob(vec![0x71; 16]),
                Value::Text("entry".into()),
                Value::Integer(500),
                Value::Text("home".into()),
                Value::Blob(ACCOUNT.to_vec()),
            ],
            vec![
                Value::Blob(vec![0x73; 16]),
                Value::Text("entry".into()),
                Value::Integer(1000),
                Value::Text("factory".into()),
                Value::Blob(ACCOUNT.to_vec()),
            ],
        ]
    );
    assert_eq!(
        rows(&reopened, "SELECT id,status FROM main.sales"),
        vec![vec![
            Value::Blob(sale_id.to_vec()),
            Value::Text("finalized".into())
        ]]
    );
    assert_eq!(
        rows(
            &reopened,
            "SELECT id,sale_id,amount_cents,customer_id,due_at_ms FROM main.receivables"
        ),
        vec![vec![
            Value::Blob(vec![0x75; 16]),
            Value::Blob(sale_id.to_vec()),
            Value::Integer(1500),
            Value::Blob(CUSTOMER.to_vec()),
            Value::Integer(DUE_AT_MS),
        ]]
    );
    assert_eq!(
        rows(
            &reopened,
            "SELECT event_id,occurred_at_ms,operation,entity,entity_id FROM main.audit_event WHERE operation IN ('sale_finalize','movement_entry_create','receivable_create') ORDER BY occurred_at_ms,event_id"
        ),
        vec![
            vec![
                Value::Blob(vec![0x70; 16]),
                Value::Integer(OCCURRED_AT_MS),
                Value::Text("sale_finalize".into()),
                Value::Text("sale".into()),
                Value::Blob(sale_id.to_vec()),
            ],
            vec![
                Value::Blob(vec![0x72; 16]),
                Value::Integer(OCCURRED_AT_MS),
                Value::Text("movement_entry_create".into()),
                Value::Text("movement".into()),
                Value::Blob(vec![0x71; 16]),
            ],
            vec![
                Value::Blob(vec![0x74; 16]),
                Value::Integer(OCCURRED_AT_MS),
                Value::Text("movement_entry_create".into()),
                Value::Text("movement".into()),
                Value::Blob(vec![0x73; 16]),
            ],
            vec![
                Value::Blob(vec![0x76; 16]),
                Value::Integer(OCCURRED_AT_MS),
                Value::Text("receivable_create".into()),
                Value::Text("receivable".into()),
                Value::Blob(vec![0x75; 16]),
            ],
        ]
    );
    let initial_balance: i64 = reopened
        .query_row(
            "SELECT initial_balance_cents FROM main.accounts WHERE id=?1",
            params![ACCOUNT.as_slice()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(initial_balance, 12_500);
    assert_healthy(&reopened);
}

#[test]
fn sal020_payment_shapes_create_only_the_expected_receivables() {
    let (_directory, path) = setup();
    let connection = open_database(&path).unwrap();
    insert_account(&connection, ACCOUNT, 0);
    insert_customer(&connection, CUSTOMER);
    drop(connection);
    let fully_paid_sale = [0x42; 16];
    let zero_sale = [0x43; 16];
    let unpaid_sale = [0x4A; 16];
    add_sale(&path, fully_paid_sale, 3000, 0x31);
    add_sale(&path, zero_sale, 0, 0x41);
    add_sale(&path, unpaid_sale, 2000, 0x51);

    let connection = open_database(&path).unwrap();
    let mut store = AdapterSqliteSaleStore::new(connection);
    let paid = run_finalization(
        &mut store,
        input(
            fully_paid_sale,
            vec![payment(ACCOUNT, 3000, Context::Factory)],
            None,
        ),
    )
    .unwrap();
    assert_eq!(
        [paid.total_cents, paid.paid_cents, paid.pending_cents],
        [3000, 3000, 0]
    );
    let zero =
        run_finalization_with_seed(&mut store, input(zero_sale, Vec::new(), None), 0x80).unwrap();
    assert_eq!(
        [zero.total_cents, zero.paid_cents, zero.pending_cents],
        [0, 0, 0]
    );
    let unpaid = run_finalization_with_seed(
        &mut store,
        input(
            unpaid_sale,
            Vec::new(),
            Some(PendingSaleTerms {
                customer_id: CUSTOMER,
                due_at_ms: DUE_AT_MS,
            }),
        ),
        0x90,
    )
    .unwrap();
    assert_eq!(
        [unpaid.total_cents, unpaid.paid_cents, unpaid.pending_cents],
        [2000, 0, 2000]
    );
    drop(store);

    let reopened = open_database(&path).unwrap();
    assert_eq!(
        rows(
            &reopened,
            "SELECT sale_id,amount_cents,customer_id,due_at_ms FROM main.receivables"
        ),
        vec![vec![
            Value::Blob(unpaid_sale.to_vec()),
            Value::Integer(2000),
            Value::Blob(CUSTOMER.to_vec()),
            Value::Integer(DUE_AT_MS),
        ]]
    );
    assert_eq!(
        rows(&reopened, "SELECT count(*) FROM main.movements"),
        vec![vec![Value::Integer(1)]]
    );
    assert_eq!(
        rows(&reopened, "SELECT id,status FROM main.sales ORDER BY id"),
        vec![
            vec![
                Value::Blob(fully_paid_sale.to_vec()),
                Value::Text("finalized".into())
            ],
            vec![
                Value::Blob(zero_sale.to_vec()),
                Value::Text("finalized".into())
            ],
            vec![
                Value::Blob(unpaid_sale.to_vec()),
                Value::Text("finalized".into())
            ],
        ]
    );
    assert_healthy(&reopened);
}

#[test]
fn sal020_business_rejections_leave_state_unchanged_and_adapter_reusable() {
    let (_directory, path) = setup();
    let connection = open_database(&path).unwrap();
    insert_account(&connection, ACCOUNT, 0);
    insert_customer(&connection, CUSTOMER);
    drop(connection);
    let sale_id = [0x44; 16];
    let finalized_id = [0x45; 16];
    let empty_id = [0x46; 16];
    add_sale(&path, sale_id, 3000, 0x31);
    add_sale(&path, finalized_id, 3000, 0x41);
    add_empty_sale(&path, empty_id, "open");
    open_database(&path)
        .unwrap()
        .execute(
            "UPDATE main.sales SET status='finalized' WHERE id=?1",
            params![finalized_id.as_slice()],
        )
        .unwrap();

    let mut store = AdapterSqliteSaleStore::new(open_database(&path).unwrap());
    let cases = [
        (input([0xFF; 16], Vec::new(), None), SaleError::SaleNotFound),
        (
            input(finalized_id, vec![payment(ACCOUNT, 1, Context::Home)], None),
            SaleError::SaleNotOpen,
        ),
        (input(empty_id, Vec::new(), None), SaleError::EmptySale),
        (
            input(sale_id, vec![payment(ACCOUNT, 3001, Context::Home)], None),
            SaleError::PaymentExceedsTotal,
        ),
        (
            input(sale_id, vec![payment(ACCOUNT, 500, Context::Home)], None),
            SaleError::PendingTermsRequired,
        ),
        (
            input(
                sale_id,
                vec![payment(ACCOUNT, 3000, Context::Home)],
                Some(PendingSaleTerms {
                    customer_id: CUSTOMER,
                    due_at_ms: DUE_AT_MS,
                }),
            ),
            SaleError::PendingTermsUnexpected,
        ),
    ];
    let before = snapshot(&open_database(&path).unwrap());
    for (value, expected) in cases {
        assert_eq!(run_finalization(&mut store, value).err(), Some(expected));
        assert_eq!(snapshot(&open_database(&path).unwrap()), before);
    }
    let result = run_finalization(
        &mut store,
        input(sale_id, vec![payment(ACCOUNT, 3000, Context::Home)], None),
    )
    .unwrap();
    assert_eq!(
        [result.total_cents, result.paid_cents, result.pending_cents],
        [3000, 3000, 0]
    );
    drop(store);
    assert_healthy(&open_database(&path).unwrap());
}

#[test]
fn sal020_sql_failures_roll_back_every_finalization_write() {
    for (case, trigger) in [
        (
            "sale-audit",
            "CREATE TRIGGER main.sal020_failure BEFORE INSERT ON audit_event WHEN NEW.operation='sale_finalize' BEGIN SELECT RAISE(ABORT,'synthetic'); END;",
        ),
        (
            "movement-audit",
            "CREATE TRIGGER main.sal020_failure BEFORE INSERT ON audit_event WHEN NEW.operation='movement_entry_create' BEGIN SELECT RAISE(ABORT,'synthetic'); END;",
        ),
        (
            "receivable-insert",
            "CREATE TRIGGER main.sal020_failure BEFORE INSERT ON receivables BEGIN SELECT RAISE(ABORT,'synthetic'); END;",
        ),
        (
            "receivable-audit",
            "CREATE TRIGGER main.sal020_failure BEFORE INSERT ON audit_event WHEN NEW.operation='receivable_create' BEGIN SELECT RAISE(ABORT,'synthetic'); END;",
        ),
    ] {
        let (_directory, path) = setup();
        let connection = open_database(&path).unwrap();
        insert_account(&connection, ACCOUNT, 0);
        insert_customer(&connection, CUSTOMER);
        drop(connection);
        let sale_id = [0x47; 16];
        add_sale(&path, sale_id, 3000, 0x31);
        let connection = open_database(&path).unwrap();
        connection.execute_batch(trigger).unwrap();
        drop(connection);
        let before = snapshot(&open_database(&path).unwrap());
        let mut store = AdapterSqliteSaleStore::new(open_database(&path).unwrap());
        assert_eq!(
            run_finalization(
                &mut store,
                input(
                    sale_id,
                    vec![
                        payment(ACCOUNT, 500, Context::Home),
                        payment(ACCOUNT, 1000, Context::Factory),
                    ],
                    Some(PendingSaleTerms {
                        customer_id: CUSTOMER,
                        due_at_ms: DUE_AT_MS,
                    }),
                ),
            )
            .err(),
            Some(SaleError::StoreFailed),
            "{case} did not fail with a sanitized store error"
        );
        drop(store);
        let reopened = open_database(&path).unwrap();
        assert_eq!(snapshot(&reopened), before, "{case} partially committed");
        assert_healthy(&reopened);
    }
}

#[test]
fn sal022_deferred_foreign_key_commit_failure_rolls_back_every_finalization_write() {
    let (_directory, path) = setup();
    let connection = open_database(&path).unwrap();
    insert_account(&connection, ACCOUNT, 100);
    insert_customer(&connection, CUSTOMER);
    drop(connection);

    let sale_id = [0x49; 16];
    add_sale(&path, sale_id, 3000, 0x31);

    let connection = open_database(&path).unwrap();
    connection
        .pragma_update(None, "defer_foreign_keys", true)
        .unwrap();
    assert_eq!(
        connection
            .pragma_query_value(None, "defer_foreign_keys", |row| row.get::<_, i64>(0))
            .unwrap(),
        1
    );
    let before = snapshot(&connection);
    let mut store = AdapterSqliteSaleStore::new(connection);
    let error = run_finalization(
        &mut store,
        input(
            sale_id,
            vec![
                payment(ACCOUNT, 500, Context::Home),
                payment([0x52; 16], 1000, Context::Factory),
            ],
            Some(PendingSaleTerms {
                customer_id: CUSTOMER,
                due_at_ms: DUE_AT_MS,
            }),
        ),
    )
    .err()
    .expect("deferred foreign key violation must fail the sale commit");
    assert_eq!(error, SaleError::StoreFailed);
    assert_eq!(format!("{error}"), "sale operation failed");
    assert_eq!(format!("{error:?}"), "StoreFailed");
    assert!(std::error::Error::source(&error).is_none());
    drop(store);

    let reopened = open_database(&path).unwrap();
    assert_eq!(
        snapshot(&reopened),
        before,
        "deferred commit failure partially finalized the sale"
    );
    assert_healthy(&reopened);
}

#[test]
fn sal020_invalid_sale_snapshot_and_missing_foreign_keys_roll_back() {
    let (_directory, path) = setup();
    let connection = open_database(&path).unwrap();
    insert_account(&connection, ACCOUNT, 0);
    insert_customer(&connection, CUSTOMER);
    drop(connection);
    let overflow_sale = [0x48; 16];
    add_sale(&path, overflow_sale, i64::MAX, 0x31);
    {
        let mut store = AdapterSqliteSaleStore::new(open_database(&path).unwrap());
        add_free_item(
            FreeItemInput {
                sale_id: Some(overflow_sale),
                description: "Item sintético de estouro".into(),
                quantity: 1,
                unit_price_cents: 1,
                discount: DiscountInput::AmountCents(0),
            },
            &mut Sequence(0x41),
            &FixedClock,
            &mut store,
        )
        .unwrap();
    }
    let before = snapshot(&open_database(&path).unwrap());
    let mut store = AdapterSqliteSaleStore::new(open_database(&path).unwrap());
    assert_eq!(
        run_finalization(&mut store, input(overflow_sale, Vec::new(), None)).err(),
        Some(SaleError::InvalidSale)
    );
    assert_eq!(snapshot(&open_database(&path).unwrap()), before);
    drop(store);

    let payment_overflow_sale = [0x4B; 16];
    add_sale(&path, payment_overflow_sale, 3000, 0x49);
    let before = snapshot(&open_database(&path).unwrap());
    let mut store = AdapterSqliteSaleStore::new(open_database(&path).unwrap());
    let write = FinalizeSaleWrite {
        sale_id: payment_overflow_sale,
        payments: vec![
            SalePaymentWrite {
                movement: Movement {
                    id: [0xA1; 16],
                    kind: MovementKind::Entry {
                        account_id: ACCOUNT,
                    },
                    amount_cents: i64::MAX,
                    context: Context::Home,
                },
                event_id: [0xA2; 16],
            },
            SalePaymentWrite {
                movement: Movement {
                    id: [0xA3; 16],
                    kind: MovementKind::Entry {
                        account_id: ACCOUNT,
                    },
                    amount_cents: 1,
                    context: Context::Factory,
                },
                event_id: [0xA4; 16],
            },
        ],
        sale_event_id: [0xA5; 16],
        occurred_at_ms: OCCURRED_AT_MS,
        pending: None,
    };
    assert_eq!(
        store.finalize_sale_with_audit(write).err(),
        Some(SaleError::InvalidPayment)
    );
    assert_eq!(snapshot(&open_database(&path).unwrap()), before);
    drop(store);

    let foreign_key_sale = [0x49; 16];
    add_sale(&path, foreign_key_sale, 1000, 0x51);
    let before = snapshot(&open_database(&path).unwrap());
    let mut store = AdapterSqliteSaleStore::new(open_database(&path).unwrap());
    assert_eq!(
        run_finalization(
            &mut store,
            input(
                foreign_key_sale,
                vec![payment([0xFE; 16], 1000, Context::Factory)],
                None,
            ),
        )
        .err(),
        Some(SaleError::StoreFailed)
    );
    drop(store);
    let reopened = open_database(&path).unwrap();
    assert_eq!(snapshot(&reopened), before);
    assert_healthy(&reopened);

    let customer_foreign_key_sale = [0x4C; 16];
    add_sale(&path, customer_foreign_key_sale, 1000, 0x61);
    let before = snapshot(&open_database(&path).unwrap());
    let mut store = AdapterSqliteSaleStore::new(open_database(&path).unwrap());
    assert_eq!(
        run_finalization(
            &mut store,
            input(
                customer_foreign_key_sale,
                Vec::new(),
                Some(PendingSaleTerms {
                    customer_id: [0xFE; 16],
                    due_at_ms: DUE_AT_MS,
                }),
            ),
        )
        .err(),
        Some(SaleError::StoreFailed)
    );
    drop(store);
    let reopened = open_database(&path).unwrap();
    assert_eq!(snapshot(&reopened), before);
    assert_healthy(&reopened);
}

#[test]
fn sal021_pending_receivables_do_not_increase_account_balances() {
    let (_directory, path) = setup();
    let account_a = [0x52; 16];
    let account_b = [0x53; 16];
    let untouched = [0x54; 16];
    let unpaid_sale = [0x42; 16];
    let partial_sale = [0x43; 16];
    let connection = open_database(&path).unwrap();
    insert_account(&connection, account_a, 100);
    insert_account(&connection, account_b, 200);
    insert_account(&connection, untouched, 300);
    insert_customer(&connection, CUSTOMER);
    drop(connection);
    add_sale(&path, unpaid_sale, 3000, 0x31);
    add_sale(&path, partial_sale, 3000, 0x41);

    let mut store = AdapterSqliteSaleStore::new(open_database(&path).unwrap());
    let unpaid = run_finalization_with_seed(
        &mut store,
        input(
            unpaid_sale,
            Vec::new(),
            Some(PendingSaleTerms {
                customer_id: CUSTOMER,
                due_at_ms: DUE_AT_MS,
            }),
        ),
        0x70,
    )
    .unwrap();
    assert_eq!(
        [unpaid.total_cents, unpaid.paid_cents, unpaid.pending_cents],
        [3000, 0, 3000]
    );
    let partial = run_finalization_with_seed(
        &mut store,
        input(
            partial_sale,
            vec![
                payment(account_a, 500, Context::Home),
                payment(account_b, 1000, Context::Factory),
            ],
            Some(PendingSaleTerms {
                customer_id: CUSTOMER,
                due_at_ms: DUE_AT_MS,
            }),
        ),
        0x80,
    )
    .unwrap();
    assert_eq!(
        [
            partial.total_cents,
            partial.paid_cents,
            partial.pending_cents
        ],
        [3000, 1500, 1500]
    );
    drop(store);

    let reopened = open_database(&path).unwrap();
    assert_healthy(&reopened);
    assert_eq!(
        rows(
            &reopened,
            "SELECT sale_id,amount_cents FROM main.receivables ORDER BY sale_id"
        ),
        vec![
            vec![Value::Blob(unpaid_sale.to_vec()), Value::Integer(3000)],
            vec![Value::Blob(partial_sale.to_vec()), Value::Integer(1500)],
        ]
    );
    assert_eq!(
        rows(&reopened, "SELECT count(*) FROM main.movements"),
        vec![vec![Value::Integer(2)]]
    );

    let accounts = list_accounts(&AdapterSqliteAccountStore::new(
        open_database(&path).unwrap(),
    ))
    .unwrap();
    let movement_reader = open_database(&path).unwrap();
    movement_reader
        .pragma_update(None, "query_only", true)
        .unwrap();
    let movements = list_movements(&AdapterSqliteMovementStore::new(movement_reader)).unwrap();
    assert_eq!(movements.len(), 2);
    assert!(movements.iter().any(|movement| {
        matches!(movement.kind, MovementKind::Entry { account_id } if account_id == account_a)
            && matches!(movement.context, Context::Home)
    }));
    assert!(movements.iter().any(|movement| {
        matches!(movement.kind, MovementKind::Entry { account_id } if account_id == account_b)
            && matches!(movement.context, Context::Factory)
    }));

    for (id, initial, expected) in [
        (account_a, 100, 600),
        (account_b, 200, 1200),
        (untouched, 300, 300),
    ] {
        let account = accounts
            .iter()
            .find(|account| account.id == id)
            .expect("account reader returned fixture account");
        assert_eq!(account.initial_balance_cents, initial);
        assert_eq!(account.balance_cents(&movements), Ok(expected));
    }
}
