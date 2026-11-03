use super::TempDirectory;
use millani_artes::database::*;
use rusqlite::{params, Connection};
use std::path::Path;

fn v15(path: &Path) -> Connection {
    let mut c = execute_versioned_migration(open_database(path).unwrap(), 1, "").unwrap();
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
    execute_sale_return_financials_migration(c).unwrap()
}
fn assert_no_foreign_key_violations(connection: &Connection) {
    let mut statement = connection.prepare("PRAGMA main.foreign_key_check").unwrap();
    assert!(statement.query([]).unwrap().next().unwrap().is_none());
}

#[test]
fn ret010_migration_runs_from_empty_v1_chain_to_v16() {
    let d = TempDirectory::new();
    let p = d.path().join("ret010-empty.sqlite");
    let c = execute_sale_return_refunds_migration(v15(&p)).unwrap();
    assert_eq!(SALE_RETURN_REFUNDS_MIGRATION_VERSION, 16);
    assert_eq!(schema_version(&c, 16).unwrap(), 16);
    assert_eq!(c.query_row("SELECT count(*) FROM main.sqlite_schema WHERE name IN ('sale_return_refunds','sale_return_refunds_return_item_idx','sale_return_refunds_require_exit','sale_return_refunds_immutable','sale_return_refunds_no_delete','sale_return_refunds_no_update')",[],|r|r.get::<_,i64>(0)).unwrap(),6);
    assert_eq!(
        c.query_row("PRAGMA main.integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    assert_no_foreign_key_violations(&c);
}
#[test]
fn ret010_upgrade_from_v15_preserves_existing_financial_rows() {
    let d = TempDirectory::new();
    let p = d.path().join("ret010-upgrade.sqlite");
    let c = v15(&p);
    let account: [u8; 16] = [0x31; 16];
    let sale: [u8; 16] = [0x32; 16];
    let item: [u8; 16] = [0x33; 16];
    let returned: [u8; 16] = [0x34; 16];
    let movement: [u8; 16] = [0x35; 16];
    c.execute("INSERT INTO main.accounts(id,name,account_type,initial_balance_cents) VALUES(?1,'Synthetic','cash',0)",[account.as_slice()]).unwrap();
    c.execute(
        "INSERT INTO main.sales(id,status) VALUES(?1,'finalized')",
        [sale.as_slice()],
    )
    .unwrap();
    c.execute("INSERT INTO main.sale_items(id,sale_id,position,product_id,description,quantity,unit_price_cents,discount_cents,total_cents) VALUES(?1,?2,0,NULL,'Synthetic',2,1000,0,2000)",params![item.as_slice(),sale.as_slice()]).unwrap();
    c.execute("INSERT INTO main.sale_return_items(id,sale_item_id,quantity,return_amount_cents,debt_reduction_cents) VALUES(?1,?2,1,1000,400)",params![returned.as_slice(),item.as_slice()]).unwrap();
    c.execute("INSERT INTO main.movements(id,kind,amount_cents,context,account_id) VALUES(?1,'exit',700,'factory',?2)",params![movement.as_slice(),account.as_slice()]).unwrap();
    let before:(i64,i64,i64)=c.query_row("SELECT r.return_amount_cents,r.debt_reduction_cents,m.amount_cents FROM main.sale_return_items r JOIN main.movements m ON m.id=?1 WHERE r.id=?2",params![movement.as_slice(),returned.as_slice()],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
    drop(c);
    let c = execute_sale_return_refunds_migration(open_database(&p).unwrap()).unwrap();
    let after:(i64,i64,i64)=c.query_row("SELECT r.return_amount_cents,r.debt_reduction_cents,m.amount_cents FROM main.sale_return_items r JOIN main.movements m ON m.id=?1 WHERE r.id=?2",params![movement.as_slice(),returned.as_slice()],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
    assert_eq!(after, before);
    assert_eq!(schema_version(&c, 16).unwrap(), 16);
    assert_eq!(
        c.query_row("PRAGMA main.integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    assert_no_foreign_key_violations(&c);
}
