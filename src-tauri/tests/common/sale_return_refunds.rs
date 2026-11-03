use super::TempDirectory;
use millani_artes::application::record_sale_refund::{record_sale_refund, RecordSaleRefundError};
use millani_artes::database::*;
use millani_artes::domain::ports::{Clock, UuidGenerator};
use millani_artes::domain::{Context, Movement};
use millani_artes::infrastructure::sale_returns::AdapterSqliteSaleRefundStore;
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const SALE: [u8; 16] = [0x31; 16];
const ITEM1: [u8; 16] = [0x32; 16];
const ITEM2: [u8; 16] = [0x33; 16];
const RET1: [u8; 16] = [0x34; 16];
const RET2: [u8; 16] = [0x35; 16];
const ACCOUNT_A: [u8; 16] = [0x36; 16];
const ACCOUNT_B: [u8; 16] = [0x37; 16];
const ENTRY: [u8; 16] = [0x38; 16];
struct Ids(u8);
impl UuidGenerator for Ids {
    type Error = ();
    fn generate(&mut self) -> Result<[u8; 16], ()> {
        let n = self.0;
        self.0 += 1;
        Ok([n; 16])
    }
}
struct FixedClock;
impl Clock for FixedClock {
    fn now(&self) -> SystemTime {
        UNIX_EPOCH + Duration::from_millis(1800000000123)
    }
}

fn setup(name: &str) -> (TempDirectory, PathBuf) {
    let dir = TempDirectory::new();
    let path = dir.path().join(name);
    let mut c = execute_versioned_migration(open_database(&path).unwrap(), 1, "").unwrap();
    for (v, s) in [
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
        c = execute_versioned_migration(c, v, s).unwrap();
    }
    c = execute_sale_return_financials_migration(c).unwrap();
    c = execute_sale_return_refunds_migration(c).unwrap();
    c.execute("INSERT INTO main.accounts(id,name,account_type,initial_balance_cents) VALUES(?1,'Original','cash',0),(?2,'Chosen','cash',100)",params![ACCOUNT_A.as_slice(),ACCOUNT_B.as_slice()]).unwrap();
    c.execute(
        "INSERT INTO main.sales(id,status) VALUES(?1,'finalized')",
        [SALE.as_slice()],
    )
    .unwrap();
    c.execute("INSERT INTO main.sale_items(id,sale_id,position,product_id,description,quantity,unit_price_cents,discount_cents,total_cents) VALUES(?1,?3,0,NULL,'Item 1',3,1000,500,2500),(?2,?3,1,NULL,'Item 2',2,1000,0,2000)",params![ITEM1.as_slice(),ITEM2.as_slice(),SALE.as_slice()]).unwrap();
    c.execute("INSERT INTO main.sale_return_items(id,sale_item_id,quantity,return_amount_cents,debt_reduction_cents) VALUES(?1,?3,3,2500,1000),(?2,?4,1,1000,500)",params![RET1.as_slice(),RET2.as_slice(),ITEM1.as_slice(),ITEM2.as_slice()]).unwrap();
    c.execute("INSERT INTO main.movements(id,kind,amount_cents,context,account_id) VALUES(?1,'entry',2500,'home',?2)",params![ENTRY.as_slice(),ACCOUNT_A.as_slice()]).unwrap();
    drop(c);
    (dir, path)
}
fn refund(
    path: &Path,
    r: [u8; 16],
    a: [u8; 16],
    amount: i64,
    context: Context,
    seed: u8,
) -> Result<Movement, RecordSaleRefundError> {
    let mut store = AdapterSqliteSaleRefundStore::new(open_database(path).unwrap());
    record_sale_refund(
        r,
        a,
        amount,
        context,
        &mut Ids(seed),
        &FixedClock,
        &mut store,
    )
}
fn count(c: &Connection, q: &str) -> i64 {
    c.query_row(q, [], |r| r.get(0)).unwrap()
}
fn total(c: &Connection, r: [u8; 16]) -> i64 {
    c.query_row("SELECT COALESCE(SUM(m.amount_cents),0) FROM main.sale_return_refunds x JOIN main.movements m ON m.id=x.movement_id WHERE x.sale_return_item_id=?1",[r.as_slice()],|row|row.get(0)).unwrap()
}

#[test]
fn ret010_records_partials_caps_each_item_and_persists_chosen_exit_context_and_audit() {
    let (_dir, path) = setup("ret010-partial.sqlite");
    let a = refund(&path, RET1, ACCOUNT_B, 700, Context::Home, 0x51).unwrap();
    let b = refund(&path, RET1, ACCOUNT_B, 800, Context::Factory, 0x53).unwrap();
    assert_eq!(a.id, [0x51; 16]);
    assert_eq!(b.id, [0x53; 16]);
    assert_eq!(total(&open_database(&path).unwrap(), RET1), 1500);
    assert!(matches!(
        refund(&path, RET1, ACCOUNT_B, 1, Context::Home, 0x55),
        Err(RecordSaleRefundError::AmountExceedsAvailable)
    ));
    assert_eq!(
        refund(&path, RET2, ACCOUNT_B, 500, Context::Factory, 0x61)
            .unwrap()
            .amount_cents,
        500
    );
    let c = open_database(&path).unwrap();
    assert_eq!(total(&c, RET1), 1500);
    assert_eq!(total(&c, RET2), 500);
    type RefundRow = (String, i64, String, Vec<u8>, Vec<u8>);
    let rows: Vec<RefundRow> = {
        let mut s=c.prepare("SELECT m.kind,m.amount_cents,m.context,m.account_id,x.sale_return_item_id FROM main.sale_return_refunds x JOIN main.movements m ON m.id=x.movement_id WHERE x.sale_return_item_id=?1 ORDER BY m.id").unwrap();
        s.query_map([RET1.as_slice()], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap()
    };
    assert_eq!(rows.len(), 2);
    assert!(rows
        .iter()
        .all(|r| r.0 == "exit" && r.3 == ACCOUNT_B && r.4 == RET1));
    assert!(rows.iter().any(|r| r.1 == 700 && r.2 == "home"));
    assert!(rows.iter().any(|r| r.1 == 800 && r.2 == "factory"));
    assert_eq!(
        count(&c, "SELECT count(*) FROM main.sale_return_refunds"),
        3
    );
    assert_eq!(count(&c,"SELECT count(*) FROM main.audit_event WHERE operation='sale_return_refund_create' AND entity='movement'"),3);
    assert_eq!(count(&c,"SELECT count(*) FROM main.audit_event e JOIN main.sale_return_refunds x ON e.entity_id=x.movement_id WHERE e.operation='sale_return_refund_create'"),3);
}

#[test]
fn ret010_missing_return_and_account_leave_no_partial_writes_after_reopen() {
    let (_dir, path) = setup("ret010-reject.sqlite");
    assert!(matches!(
        refund(&path, [0xff; 16], ACCOUNT_B, 1, Context::Home, 0x51),
        Err(RecordSaleRefundError::ReturnItemNotFound)
    ));
    assert!(matches!(
        refund(&path, RET1, [0xfe; 16], 1, Context::Home, 0x53),
        Err(RecordSaleRefundError::StoreFailed)
    ));
    let c = open_database(&path).unwrap();
    assert_eq!(
        count(&c, "SELECT count(*) FROM main.sale_return_refunds"),
        0
    );
    assert_eq!(count(&c, "SELECT count(*) FROM main.movements"), 1);
    assert_eq!(
        count(
            &c,
            "SELECT count(*) FROM main.audit_event WHERE operation='sale_return_refund_create'"
        ),
        0
    );
}

#[test]
fn ret010_confirmed_refund_kind_amount_and_link_are_protected() {
    let (_dir, path) = setup("ret010-protect.sqlite");
    let m = refund(&path, RET1, ACCOUNT_B, 700, Context::Home, 0x51).unwrap();
    let c = open_database(&path).unwrap();
    assert!(c
        .execute(
            "UPDATE main.movements SET amount_cents=1501 WHERE id=?1",
            [m.id.as_slice()]
        )
        .is_err());
    assert!(c
        .execute(
            "UPDATE main.movements SET kind='entry' WHERE id=?1",
            [m.id.as_slice()]
        )
        .is_err());
    assert!(c
        .execute(
            "DELETE FROM main.sale_return_refunds WHERE movement_id=?1",
            [m.id.as_slice()]
        )
        .is_err());
    assert_eq!(total(&c, RET1), 700);
}

#[test]
fn ret010_concurrent_refunds_do_not_spend_reserved_value_twice() {
    let (_dir, path) = setup("ret010-concurrent.sqlite");
    let barrier = Arc::new(Barrier::new(3));
    let mut workers = Vec::new();
    for seed in [0x51, 0x61] {
        let p = path.clone();
        let b = Arc::clone(&barrier);
        workers.push(thread::spawn(move || {
            b.wait();
            refund(&p, RET1, ACCOUNT_B, 900, Context::Home, seed)
        }));
    }
    barrier.wait();
    let results: Vec<_> = workers.into_iter().map(|w| w.join().unwrap()).collect();
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    assert_eq!(results.iter().filter(|r| r.is_err()).count(), 1);
    let c = open_database(&path).unwrap();
    assert_eq!(
        count(&c, "SELECT count(*) FROM main.sale_return_refunds"),
        1
    );
    assert!(total(&c, RET1) <= 1500);
}
