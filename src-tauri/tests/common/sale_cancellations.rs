use super::TempDirectory;
use millani_artes::application::add_product_item::read_sale;
use millani_artes::application::cancel_paid_sale::{
    cancel_paid_sale, preview_sale_cancellation, SaleCancellationPreview,
    SaleCancellationRefundRoute, SaleCancellationRefundTarget,
};
use millani_artes::application::record_sale_refund::record_sale_refund;
use millani_artes::database::*;
use millani_artes::domain::ports::{Clock, UuidGenerator};
use millani_artes::domain::{Context, SaleStatus};
use millani_artes::infrastructure::sale_returns::{
    AdapterSqliteSaleCancellationStore, AdapterSqliteSaleRefundStore,
};
use millani_artes::infrastructure::sales::AdapterSqliteSaleStore;
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const SALE: [u8; 16] = [0x31; 16];
const ITEM_A: [u8; 16] = [0x32; 16];
const ITEM_B: [u8; 16] = [0x33; 16];
const PRIOR_RETURN: [u8; 16] = [0x34; 16];
const ACCOUNT_ORIGINAL: [u8; 16] = [0x35; 16];
const ACCOUNT_B: [u8; 16] = [0x36; 16];
const ACCOUNT_C: [u8; 16] = [0x37; 16];
const CUSTOMER: [u8; 16] = [0x38; 16];

struct Ids(u8);
impl UuidGenerator for Ids {
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
        UNIX_EPOCH + Duration::from_millis(1_800_000_000_123)
    }
}

fn latest_v16(path: &Path) -> Connection {
    let mut c = execute_versioned_migration(open_database(path).unwrap(), 1, "").unwrap();
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
        c = execute_versioned_migration(c, version, sql).unwrap();
    }
    c = execute_sale_return_financials_migration(c).unwrap();
    execute_sale_return_refunds_migration(c).unwrap()
}

fn setup(name: &str, fully_paid: bool) -> (TempDirectory, PathBuf) {
    let directory = TempDirectory::new();
    let path = directory.path().join(name);
    let c = latest_v16(&path);
    drop(execute_sale_cancellations_migration(c).unwrap());
    let c = open_database(&path).unwrap();
    c.execute("INSERT INTO main.accounts(id,name,account_type,initial_balance_cents) VALUES(?1,'Original','cash',0),(?2,'B','cash',0),(?3,'C','cash',0)",params![ACCOUNT_ORIGINAL.as_slice(),ACCOUNT_B.as_slice(),ACCOUNT_C.as_slice()]).unwrap();
    c.execute(
        "INSERT INTO main.customers(id,name,phone) VALUES(?1,'Synthetic',NULL)",
        [CUSTOMER.as_slice()],
    )
    .unwrap();
    c.execute(
        "INSERT INTO main.sales(id,status) VALUES(?1,'finalized')",
        [SALE.as_slice()],
    )
    .unwrap();
    c.execute("INSERT INTO main.sale_items(id,sale_id,position,product_id,description,quantity,unit_price_cents,discount_cents,total_cents) VALUES(?1,?3,0,NULL,'A',2,1000,0,2000),(?2,?3,1,NULL,'B',1,1000,0,1000)",params![ITEM_A.as_slice(),ITEM_B.as_slice(),SALE.as_slice()]).unwrap();
    c.execute("INSERT INTO main.receivables(id,sale_id,amount_cents,customer_id,due_at_ms) VALUES(?1,?2,3000,?3,1800000000000)",params![[0x39_u8;16].as_slice(),SALE.as_slice(),CUSTOMER.as_slice()]).unwrap();
    let received = if fully_paid { 2800 } else { 1800 };
    c.execute("INSERT INTO main.receipts(id,receivable_id,account_id,amount_cents,context) VALUES(?1,?2,?3,?4,'home')",params![[0x3a_u8;16].as_slice(),[0x39_u8;16].as_slice(),ACCOUNT_ORIGINAL.as_slice(),received]).unwrap();
    c.execute("INSERT INTO main.movements(id,kind,amount_cents,context,account_id) VALUES(?1,'entry',?2,'home',?3)",params![[0x3b_u8;16].as_slice(),received,ACCOUNT_ORIGINAL.as_slice()]).unwrap();
    c.execute("INSERT INTO main.sale_return_items(id,sale_item_id,quantity,return_amount_cents,debt_reduction_cents) VALUES(?1,?2,1,1000,200)",params![PRIOR_RETURN.as_slice(),ITEM_A.as_slice()]).unwrap();
    c.execute("INSERT INTO main.audit_event(event_id,occurred_at_ms,operation,entity,entity_id) VALUES(?1,1800000000000,'fixture_seed','sale',?2)",params![[0x3c_u8;16].as_slice(),SALE.as_slice()]).unwrap();
    drop(c);
    (directory, path)
}

fn setup_paid_cash_sale(name: &str) -> (TempDirectory, PathBuf) {
    let directory = TempDirectory::new();
    let path = directory.path().join(name);
    drop(execute_sale_cancellations_migration(latest_v16(&path)).unwrap());
    let c = open_database(&path).unwrap();
    c.execute(
        "INSERT INTO main.accounts(id,name,account_type,initial_balance_cents) VALUES(?1,'Cash','cash',0)",
        [ACCOUNT_ORIGINAL.as_slice()],
    )
    .unwrap();
    c.execute(
        "INSERT INTO main.sales(id,status) VALUES(?1,'finalized')",
        [SALE.as_slice()],
    )
    .unwrap();
    c.execute("INSERT INTO main.sale_items(id,sale_id,position,product_id,description,quantity,unit_price_cents,discount_cents,total_cents) VALUES(?1,?3,0,NULL,'A',2,1000,0,2000),(?2,?3,1,NULL,'B',1,1000,0,1000)",params![ITEM_A.as_slice(),ITEM_B.as_slice(),SALE.as_slice()]).unwrap();
    c.execute(
        "INSERT INTO main.movements(id,kind,amount_cents,context,account_id) VALUES(?1,'entry',3000,'home',?2)",
        params![[0x3b_u8; 16].as_slice(), ACCOUNT_ORIGINAL.as_slice()],
    )
    .unwrap();
    c.execute(
        "INSERT INTO main.audit_event(event_id,occurred_at_ms,operation,entity,entity_id) VALUES(?1,1800000000000,'fixture_seed','sale',?2)",
        params![[0x3c_u8; 16].as_slice(),SALE.as_slice()],
    )
    .unwrap();
    drop(c);
    (directory, path)
}

fn cancel(
    path: &Path,
    routes: Vec<SaleCancellationRefundRoute>,
) -> Result<(), millani_artes::application::cancel_paid_sale::CancelPaidSaleError> {
    let mut store = AdapterSqliteSaleCancellationStore::new(open_database(path).unwrap());
    let preview = preview_sale_cancellation(SALE, &mut store)?;
    cancel_paid_sale(preview, routes, &mut Ids(0x41), &FixedClock, &mut store)
}

fn routes_for(
    preview: &SaleCancellationPreview,
    account_id: [u8; 16],
) -> Vec<SaleCancellationRefundRoute> {
    preview
        .refunds
        .iter()
        .map(|refund| SaleCancellationRefundRoute {
            target: refund.target,
            account_id,
            context: Context::Home,
        })
        .collect()
}

#[test]
fn ret012_cancel_paid_sale_returns_remaining_items_and_refunds_each_open_reservation_once() {
    let (_directory, path) = setup("ret012-cancel.sqlite", true);
    let mut refunds = AdapterSqliteSaleRefundStore::new(open_database(&path).unwrap());
    record_sale_refund(
        PRIOR_RETURN,
        ACCOUNT_ORIGINAL,
        400,
        Context::Home,
        &mut Ids(0x61),
        &FixedClock,
        &mut refunds,
    )
    .unwrap();
    drop(refunds);

    let routes = vec![
        SaleCancellationRefundRoute {
            target: SaleCancellationRefundTarget::ExistingReturnItem(PRIOR_RETURN),
            account_id: ACCOUNT_B,
            context: Context::Factory,
        },
        SaleCancellationRefundRoute {
            target: SaleCancellationRefundTarget::CancellationReturnForSaleItem(ITEM_A),
            account_id: ACCOUNT_B,
            context: Context::Home,
        },
        SaleCancellationRefundRoute {
            target: SaleCancellationRefundTarget::CancellationReturnForSaleItem(ITEM_B),
            account_id: ACCOUNT_B,
            context: Context::Home,
        },
    ];
    cancel(&path, routes).unwrap();

    let c = open_database(&path).unwrap();
    assert_eq!(
        c.query_row(
            "SELECT count(*) FROM main.sale_cancellations WHERE sale_id=?1",
            [SALE.as_slice()],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
    assert_eq!(
        c.query_row("SELECT count(*) FROM main.sale_return_items", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        3
    );
    assert_eq!(
        c.query_row(
            "SELECT SUM(quantity) FROM main.sale_return_items WHERE sale_item_id=?1",
            [ITEM_A.as_slice()],
            |r| r.get::<_, i64>(0),
        )
        .unwrap(),
        2
    );
    assert_eq!(
        c.query_row(
            "SELECT SUM(quantity) FROM main.sale_return_items WHERE sale_item_id=?1",
            [ITEM_B.as_slice()],
            |r| r.get::<_, i64>(0),
        )
        .unwrap(),
        1
    );
    assert_eq!(
        c.query_row("SELECT count(*) FROM main.sale_return_refunds", [], |r| r
            .get::<_, i64>(
            0
        ))
        .unwrap(),
        4
    );
    assert_eq!(
        c.query_row(
            "SELECT sum(amount_cents) FROM main.movements WHERE kind='exit'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        2800
    );
    let mut routed: Vec<(i64, Vec<u8>, String)> = {
        let mut s = c.prepare("SELECT m.amount_cents,m.account_id,m.context FROM main.sale_return_refunds x JOIN main.movements m ON m.id=x.movement_id ORDER BY m.id").unwrap();
        s.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    routed.sort();
    let mut expected_routed = vec![
        (400, ACCOUNT_ORIGINAL.to_vec(), "home".to_owned()),
        (400, ACCOUNT_B.to_vec(), "factory".to_owned()),
        (1000, ACCOUNT_B.to_vec(), "home".to_owned()),
        (1000, ACCOUNT_B.to_vec(), "home".to_owned()),
    ];
    expected_routed.sort();
    assert_eq!(routed, expected_routed);
    for (account, expected_balance) in
        [(ACCOUNT_ORIGINAL, 2400), (ACCOUNT_B, -2400), (ACCOUNT_C, 0)]
    {
        assert_eq!(
            c.query_row(
                "SELECT COALESCE(SUM(CASE WHEN kind='entry' THEN amount_cents WHEN kind='exit' THEN -amount_cents ELSE 0 END),0) FROM main.movements WHERE account_id=?1",
                [account.as_slice()],
                |r| r.get::<_, i64>(0),
            )
            .unwrap(),
            expected_balance
        );
    }
    assert_eq!(
        c.query_row(
            "SELECT count(*) FROM main.audit_event WHERE operation='sale_return_item_create' AND entity='sale_return_item'",
            [],
            |r| r.get::<_, i64>(0),
        )
        .unwrap(),
        2
    );
    assert_eq!(
        c.query_row(
            "SELECT count(*) FROM main.audit_event WHERE operation='sale_return_refund_create' AND entity='movement'",
            [],
            |r| r.get::<_, i64>(0),
        )
        .unwrap(),
        4
    );
    assert_eq!(
        c.query_row("SELECT count(*) FROM main.audit_event WHERE operation='sale_cancelled' AND entity='sale' AND entity_id=?1", [SALE.as_slice()], |r| r.get::<_, i64>(0)).unwrap(),
        1
    );
    drop(c);

    let mut sales = AdapterSqliteSaleStore::new(open_database(&path).unwrap());
    let sale = read_sale(&mut sales, SALE).unwrap().unwrap();
    assert!(matches!(sale.status, SaleStatus::Cancelled));
    assert!(matches!(
        cancel(&path, Vec::new()),
        Err(millani_artes::application::cancel_paid_sale::CancelPaidSaleError::AlreadyCancelled)
    ));
    let c = open_database(&path).unwrap();
    assert_eq!(
        c.query_row("SELECT count(*) FROM main.sale_return_items", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        3
    );
    assert_eq!(
        c.query_row(
            "SELECT count(*) FROM main.audit_event WHERE operation='sale_cancelled'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
}

#[test]
fn ret012_cancels_fully_paid_sale_without_receivable() {
    let (_directory, path) = setup_paid_cash_sale("ret012-cash-sale.sqlite");
    let mut store = AdapterSqliteSaleCancellationStore::new(open_database(&path).unwrap());
    let preview = preview_sale_cancellation(SALE, &mut store).unwrap();
    assert_eq!(preview.returns.len(), 2);
    assert_eq!(preview.refunds.len(), 2);
    let routes = routes_for(&preview, ACCOUNT_ORIGINAL);
    cancel_paid_sale(preview, routes, &mut Ids(0x41), &FixedClock, &mut store).unwrap();

    let c = open_database(&path).unwrap();
    assert_eq!(
        c.query_row("SELECT count(*) FROM main.receivables", [], |row| row
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        c.query_row(
            "SELECT SUM(CASE WHEN kind='entry' THEN amount_cents WHEN kind='exit' THEN -amount_cents ELSE 0 END) FROM main.movements WHERE account_id=?1",
            [ACCOUNT_ORIGINAL.as_slice()],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        0
    );
    drop(c);
    let mut sales = AdapterSqliteSaleStore::new(open_database(&path).unwrap());
    assert!(matches!(
        read_sale(&mut sales, SALE).unwrap().unwrap().status,
        SaleStatus::Cancelled
    ));
}

#[test]
fn ret012_rejects_partially_paid_sale_before_any_cancellation_write() {
    let (_directory, path) = setup("ret012-open-debt.sqlite", false);
    let mut store = AdapterSqliteSaleCancellationStore::new(open_database(&path).unwrap());
    assert!(matches!(
        preview_sale_cancellation(SALE, &mut store),
        Err(
            millani_artes::application::cancel_paid_sale::CancelPaidSaleError::ReceivableNotSettled
        )
    ));
    let c = open_database(&path).unwrap();
    assert_eq!(
        c.query_row("SELECT count(*) FROM main.sale_cancellations", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        c.query_row("SELECT count(*) FROM main.sale_return_items", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        c.query_row("SELECT count(*) FROM main.sale_return_refunds", [], |r| r
            .get::<_, i64>(
            0
        ))
        .unwrap(),
        0
    );
}

#[test]
fn ret012_requires_exact_refund_routes_and_one_account_before_generating_ids_or_writing() {
    struct NoIds;
    impl UuidGenerator for NoIds {
        type Error = ();
        fn generate(&mut self) -> Result<[u8; 16], ()> {
            Err(())
        }
    }

    for (case, filename) in ["missing", "extra", "duplicate", "split-account"]
        .into_iter()
        .enumerate()
    {
        let (_directory, path) = setup(&format!("ret012-routes-{filename}.sqlite"), true);
        let mut store = AdapterSqliteSaleCancellationStore::new(open_database(&path).unwrap());
        let preview = preview_sale_cancellation(SALE, &mut store).unwrap();
        let mut routes = routes_for(&preview, ACCOUNT_B);
        match case {
            0 => {
                routes.pop();
            }
            1 => routes.push(SaleCancellationRefundRoute {
                target: SaleCancellationRefundTarget::ExistingReturnItem([0x71; 16]),
                account_id: ACCOUNT_B,
                context: Context::Home,
            }),
            3 => routes[1].account_id = ACCOUNT_C,
            _ => {
                let first = routes.first().unwrap();
                routes.push(SaleCancellationRefundRoute {
                    target: first.target,
                    account_id: first.account_id,
                    context: Context::Home,
                });
            }
        }
        assert_eq!(
            cancel_paid_sale(preview, routes, &mut NoIds, &FixedClock, &mut store),
            Err(millani_artes::application::cancel_paid_sale::CancelPaidSaleError::RefundRoutesInvalid)
        );
        let c = open_database(&path).unwrap();
        assert_eq!(
            c.query_row("SELECT count(*) FROM main.sale_cancellations", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert_eq!(
            c.query_row("SELECT count(*) FROM main.sale_return_items", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            1
        );
    }
}

#[test]
fn ret012_rejects_missing_or_nonfinalized_sale_without_writes() {
    let (_directory, path) = setup("ret012-sale-state.sqlite", true);
    let c = open_database(&path).unwrap();
    c.execute(
        "UPDATE main.sales SET status='open' WHERE id=?1",
        [SALE.as_slice()],
    )
    .unwrap();
    drop(c);

    let mut store = AdapterSqliteSaleCancellationStore::new(open_database(&path).unwrap());
    assert!(matches!(
        preview_sale_cancellation(SALE, &mut store),
        Err(millani_artes::application::cancel_paid_sale::CancelPaidSaleError::SaleNotFinalized)
    ));
    assert!(matches!(
        preview_sale_cancellation([0x78; 16], &mut store),
        Err(millani_artes::application::cancel_paid_sale::CancelPaidSaleError::SaleNotFound)
    ));
    let c = open_database(&path).unwrap();
    assert_eq!(
        c.query_row("SELECT count(*) FROM main.sale_cancellations", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        c.query_row("SELECT count(*) FROM main.sale_return_items", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
}

#[test]
fn ret012_database_guard_requires_finalized_and_settled_sale() {
    for (fully_paid, force_open, filename) in [(false, false, "unsettled"), (true, true, "open")] {
        let (_directory, path) = setup(&format!("ret012-db-guard-{filename}.sqlite"), fully_paid);
        let c = open_database(&path).unwrap();
        if force_open {
            c.execute(
                "UPDATE main.sales SET status='open' WHERE id=?1",
                [SALE.as_slice()],
            )
            .unwrap();
        }
        assert!(c
            .execute(
                "INSERT INTO main.sale_cancellations(sale_id,cancelled_at_ms) VALUES(?1,1800000000000)",
                [SALE.as_slice()],
            )
            .is_err());
        assert_eq!(
            c.query_row("SELECT count(*) FROM main.sale_cancellations", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
}

#[test]
fn ret012_revalidates_settlement_inside_write_transaction() {
    let (_directory, path) = setup("ret012-stale-preview.sqlite", true);
    let mut store = AdapterSqliteSaleCancellationStore::new(open_database(&path).unwrap());
    let preview = preview_sale_cancellation(SALE, &mut store).unwrap();
    let routes = routes_for(&preview, ACCOUNT_B);
    drop(store);
    let c = open_database(&path).unwrap();
    c.execute(
        "UPDATE main.receivables SET amount_cents=amount_cents+1 WHERE sale_id=?1",
        [SALE.as_slice()],
    )
    .unwrap();
    drop(c);

    let mut store = AdapterSqliteSaleCancellationStore::new(open_database(&path).unwrap());
    assert!(matches!(
        cancel_paid_sale(preview, routes, &mut Ids(0x71), &FixedClock, &mut store),
        Err(
            millani_artes::application::cancel_paid_sale::CancelPaidSaleError::ReceivableNotSettled
        )
    ));
    let c = open_database(&path).unwrap();
    assert_eq!(
        c.query_row("SELECT count(*) FROM main.sale_cancellations", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        c.query_row("SELECT count(*) FROM main.sale_return_items", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        c.query_row("SELECT count(*) FROM main.sale_return_refunds", [], |r| r
            .get::<_, i64>(
            0
        ))
        .unwrap(),
        0
    );
}

#[test]
fn ret012_rejects_unknown_refund_account_without_partial_writes() {
    let (_directory, path) = setup("ret012-unknown-account.sqlite", true);
    let mut store = AdapterSqliteSaleCancellationStore::new(open_database(&path).unwrap());
    let preview = preview_sale_cancellation(SALE, &mut store).unwrap();
    let routes = routes_for(&preview, [0x72; 16]);
    assert!(matches!(
        cancel_paid_sale(preview, routes, &mut Ids(0x73), &FixedClock, &mut store),
        Err(millani_artes::application::cancel_paid_sale::CancelPaidSaleError::AccountNotFound)
    ));
    let c = open_database(&path).unwrap();
    assert_eq!(
        c.query_row("SELECT count(*) FROM main.sale_cancellations", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        c.query_row("SELECT count(*) FROM main.sale_return_items", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        c.query_row(
            "SELECT count(*) FROM main.movements WHERE kind='exit'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
}

#[test]
fn ret012_cancellation_is_immutable_and_blocks_future_financial_records() {
    let (_directory, path) = setup("ret012-terminal-state.sqlite", true);
    let mut store = AdapterSqliteSaleCancellationStore::new(open_database(&path).unwrap());
    let preview = preview_sale_cancellation(SALE, &mut store).unwrap();
    let routes = routes_for(&preview, ACCOUNT_B);
    cancel_paid_sale(preview, routes, &mut Ids(0x73), &FixedClock, &mut store).unwrap();

    let c = open_database(&path).unwrap();
    assert!(c
        .execute(
            "UPDATE main.sale_cancellations SET cancelled_at_ms=cancelled_at_ms+1 WHERE sale_id=?1",
            [SALE.as_slice()],
        )
        .is_err());
    assert!(c
        .execute(
            "DELETE FROM main.sale_cancellations WHERE sale_id=?1",
            [SALE.as_slice()],
        )
        .is_err());
    assert!(c
        .execute(
            "INSERT INTO main.sale_return_items(id,sale_item_id,quantity,return_amount_cents,debt_reduction_cents) VALUES(?1,?2,1,100,0)",
            params![[0x74_u8; 16].as_slice(), ITEM_A.as_slice()],
        )
        .is_err());
    assert!(c
        .execute(
            "INSERT INTO main.receivables(id,sale_id,amount_cents,customer_id,due_at_ms) VALUES(?1,?2,1,?3,1800000000000)",
            params![[0x75_u8; 16].as_slice(), SALE.as_slice(), CUSTOMER.as_slice()],
        )
        .is_err());
    assert!(c
        .execute(
            "INSERT INTO main.receipts(id,receivable_id,account_id,amount_cents,context) VALUES(?1,?2,?3,1,'home')",
            params![[0x76_u8; 16].as_slice(), [0x39_u8; 16].as_slice(), ACCOUNT_ORIGINAL.as_slice()],
        )
        .is_err());

    c.execute_batch("SAVEPOINT ret012_refund_guard").unwrap();
    c.execute(
        "INSERT INTO main.movements(id,kind,amount_cents,context,account_id) VALUES(?1,'exit',1,'home',?2)",
        params![[0x91_u8; 16].as_slice(), ACCOUNT_B.as_slice()],
    )
    .unwrap();
    assert!(c
        .execute(
            "INSERT INTO main.sale_return_refunds(movement_id,sale_return_item_id) VALUES(?1,?2)",
            params![[0x91_u8; 16].as_slice(), PRIOR_RETURN.as_slice()],
        )
        .is_err());
    c.execute_batch("ROLLBACK TO ret012_refund_guard; RELEASE ret012_refund_guard")
        .unwrap();
    assert_eq!(
        c.query_row(
            "SELECT count(*) FROM main.sale_cancellations WHERE sale_id=?1",
            [SALE.as_slice()],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
    assert_eq!(
        c.query_row("SELECT count(*) FROM main.sale_return_items", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        3
    );
}
