use super::TempDirectory;
use millani_artes::database::*;
use rusqlite::Connection;
use std::path::Path;

fn v16(path: &Path) -> Connection {
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

#[test]
fn ret012_migration_from_v16_preserves_data_and_installs_immutable_cancellation_state() {
    let d = TempDirectory::new();
    let p = d.path().join("ret012-v16-upgrade.sqlite");
    let c = v16(&p);
    let sale = [0x31_u8; 16];
    let item = [0x32_u8; 16];
    let account = [0x33_u8; 16];
    let customer = [0x34_u8; 16];
    let receivable = [0x35_u8; 16];
    let receipt = [0x36_u8; 16];
    let event = [0x37_u8; 16];
    c.execute(
        "INSERT INTO main.accounts(id,name,account_type,initial_balance_cents) VALUES(?1,'Cash','cash',0)",
        [account.as_slice()],
    )
    .unwrap();
    c.execute(
        "INSERT INTO main.customers(id,name,phone) VALUES(?1,'Synthetic',NULL)",
        [customer.as_slice()],
    )
    .unwrap();
    c.execute(
        "INSERT INTO main.sales(id,status) VALUES(?1,'finalized')",
        [sale.as_slice()],
    )
    .unwrap();
    c.execute(
        "INSERT INTO main.sale_items(id,sale_id,position,product_id,description,quantity,unit_price_cents,discount_cents,total_cents) VALUES(?1,?2,0,NULL,'Item',1,1000,0,1000)",
        rusqlite::params![item.as_slice(), sale.as_slice()],
    )
    .unwrap();
    c.execute(
        "INSERT INTO main.receivables(id,sale_id,amount_cents,customer_id,due_at_ms) VALUES(?1,?2,1000,?3,1800000000000)",
        rusqlite::params![receivable.as_slice(), sale.as_slice(), customer.as_slice()],
    )
    .unwrap();
    c.execute(
        "INSERT INTO main.receipts(id,receivable_id,account_id,amount_cents,context) VALUES(?1,?2,?3,1000,'home')",
        rusqlite::params![receipt.as_slice(), receivable.as_slice(), account.as_slice()],
    )
    .unwrap();
    c.execute(
        "INSERT INTO main.audit_event(event_id,occurred_at_ms,operation,entity,entity_id) VALUES(?1,1800000000000,'fixture_seed','sale',?2)",
        rusqlite::params![event.as_slice(), sale.as_slice()],
    )
    .unwrap();
    assert_eq!(schema_version(&c, 16).unwrap(), 16);
    drop(c);

    let c = execute_sale_cancellations_migration(open_database(&p).unwrap()).unwrap();
    assert_eq!(SALE_CANCELLATIONS_MIGRATION_VERSION, 17);
    assert_eq!(schema_version(&c, 17).unwrap(), 17);
    assert_eq!(
        c.query_row(
            "SELECT count(*) FROM main.sales WHERE id=?1 AND status='finalized'",
            [sale.as_slice()],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
    assert_eq!(
        c.query_row(
            "SELECT count(*) FROM main.sale_items WHERE id=?1",
            [item.as_slice()],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
    assert_eq!(
        c.query_row(
            "SELECT amount_cents FROM main.receivables WHERE id=?1",
            [receivable.as_slice()],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        1000
    );
    assert_eq!(
        c.query_row(
            "SELECT amount_cents FROM main.receipts WHERE id=?1",
            [receipt.as_slice()],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        1000
    );
    assert_eq!(
        c.query_row(
            "SELECT count(*) FROM main.audit_event WHERE event_id=?1",
            [event.as_slice()],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
    assert_eq!(
        c.query_row("PRAGMA main.integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    let mut foreign_keys = c.prepare("PRAGMA main.foreign_key_check").unwrap();
    assert!(foreign_keys.query([]).unwrap().next().unwrap().is_none());
}

#[test]
fn ret012_migration_runs_from_empty_file_through_v16_to_v17() {
    let d = TempDirectory::new();
    let p = d.path().join("ret012-empty-v16.sqlite");
    let c = v16(&p);
    assert_eq!(schema_version(&c, 16).unwrap(), 16);
    drop(c);

    let c = execute_sale_cancellations_migration(open_database(&p).unwrap()).unwrap();
    assert_eq!(schema_version(&c, 17).unwrap(), 17);
    assert_eq!(
        c.query_row("SELECT count(*) FROM main.sale_cancellations", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        c.query_row("PRAGMA main.integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    let mut foreign_keys = c.prepare("PRAGMA main.foreign_key_check").unwrap();
    assert!(foreign_keys.query([]).unwrap().next().unwrap().is_none());
}

#[test]
fn ret012_migration_failure_rolls_back_ddl_and_version_then_retries() {
    let d = TempDirectory::new();
    let p = d.path().join("ret012-migration-retry.sqlite");
    drop(v16(&p));
    let c = open_database(&p).unwrap();
    c.execute_batch("CREATE TABLE main.sale_cancellations(marker TEXT)")
        .unwrap();
    drop(c);

    assert!(execute_sale_cancellations_migration(open_database(&p).unwrap()).is_err());
    let c = open_database(&p).unwrap();
    assert_eq!(schema_version(&c, 16).unwrap(), 16);
    assert_eq!(
        c.query_row(
            "SELECT COUNT(*) FROM main.sqlite_schema WHERE type='trigger' AND name LIKE 'sale_cancellations_%'",
            [],
            |r| r.get::<_, i64>(0),
        )
        .unwrap(),
        0
    );
    assert_eq!(
        c.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('sale_cancellations') WHERE name='marker'",
            [],
            |r| r.get::<_, i64>(0),
        )
        .unwrap(),
        1
    );
    drop(c);
    open_database(&p)
        .unwrap()
        .execute("DROP TABLE main.sale_cancellations", [])
        .unwrap();

    let c = execute_sale_cancellations_migration(open_database(&p).unwrap()).unwrap();
    assert_eq!(schema_version(&c, 17).unwrap(), 17);
    assert_eq!(
        c.query_row(
            "SELECT COUNT(*) FROM main.sqlite_schema WHERE type='trigger' AND name LIKE 'sale_cancellations_%'",
            [],
            |r| r.get::<_, i64>(0),
        )
        .unwrap(),
        3
    );
}
