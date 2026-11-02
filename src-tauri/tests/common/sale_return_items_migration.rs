use super::TempDirectory;
use millani_artes::database::{
    execute_versioned_migration, open_database, schema_version, MigrationError,
    ACCOUNTS_MIGRATION_SQL, AUDIT_EVENT_MIGRATION_SQL, CATEGORIES_MIGRATION_SQL,
    CUSTOMERS_MIGRATION_SQL, MOVEMENTS_MIGRATION_SQL, MOVEMENT_CATEGORIES_MIGRATION_SQL,
    PRODUCTS_MIGRATION_SQL, RECEIPTS_MIGRATION_SQL, RECEIPTS_MIGRATION_VERSION,
    RECEIVABLES_MIGRATION_SQL, RECURRENCES_MIGRATION_SQL, SALES_MIGRATION_SQL,
    SALE_ITEMS_MIGRATION_SQL, SALE_RETURN_ITEMS_MIGRATION_SQL, SALE_RETURN_ITEMS_MIGRATION_VERSION,
};
use rusqlite::{params, types::Value, Connection};
use std::path::Path;

fn rows(connection: &Connection, sql: &str) -> Vec<Vec<Value>> {
    let mut statement = connection.prepare(sql).unwrap();
    let columns = statement.column_count();
    statement
        .query_map([], |row| {
            (0..columns).map(|column| row.get(column)).collect()
        })
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap()
}

fn v13(path: &Path) -> Connection {
    let mut connection = execute_versioned_migration(open_database(path).unwrap(), 1, "").unwrap();
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
        (RECEIPTS_MIGRATION_VERSION, RECEIPTS_MIGRATION_SQL),
    ] {
        connection = execute_versioned_migration(connection, version, sql).unwrap();
    }
    assert_eq!(schema_version(&connection, 14).unwrap(), 13);
    connection
}

fn migrate_v14(connection: Connection) -> Connection {
    assert_eq!(SALE_RETURN_ITEMS_MIGRATION_VERSION, 14);
    execute_versioned_migration(
        connection,
        SALE_RETURN_ITEMS_MIGRATION_VERSION,
        SALE_RETURN_ITEMS_MIGRATION_SQL,
    )
    .unwrap()
}

fn schema_snapshot(connection: &Connection) -> Vec<Vec<Value>> {
    rows(
        connection,
        "SELECT type, name, tbl_name, sql FROM main.sqlite_schema WHERE name NOT GLOB 'sqlite_*' ORDER BY type, name",
    )
}

fn assert_health(connection: &Connection, expected_version: u32) {
    assert!(connection.is_autocommit());
    assert_eq!(schema_version(connection, 14).unwrap(), expected_version);
    assert_eq!(
        rows(
            connection,
            "SELECT version FROM main.schema_migration ORDER BY version"
        ),
        (1..=expected_version)
            .map(|version| vec![Value::Integer(i64::from(version))])
            .collect::<Vec<_>>()
    );
    assert_eq!(
        rows(connection, "PRAGMA main.integrity_check"),
        vec![vec![Value::Text("ok".into())]]
    );
    assert!(rows(connection, "PRAGMA main.foreign_key_check").is_empty());
}

fn assert_return_item_schema(connection: &Connection) {
    let mut statement = connection
        .prepare("PRAGMA main.table_xinfo('sale_return_items')")
        .unwrap();
    let columns = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
            ))
        })
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap();
    assert_eq!(
        columns,
        vec![
            ("id".into(), "BLOB".into(), 1, 1, 0),
            ("sale_item_id".into(), "BLOB".into(), 1, 0, 0),
            ("quantity".into(), "INTEGER".into(), 1, 0, 0),
        ]
    );

    let sql: String = connection
        .query_row(
            "SELECT sql FROM main.sqlite_schema WHERE type='table' AND name='sale_return_items'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(sql.contains("STRICT"));
    assert!(sql.contains("sale_return_items_quantity_positive"));
    assert!(sql.contains("sale_return_items_sale_item_id_fkey"));

    let indexes = rows(connection, "PRAGMA main.index_list('sale_return_items')");
    assert!(indexes.iter().any(|row| {
        row[1] == Value::Text("sale_return_items_sale_item_id_idx".into())
            && row[2] == Value::Integer(0)
    }));

    let foreign_keys = rows(
        connection,
        "PRAGMA main.foreign_key_list('sale_return_items')",
    );
    assert_eq!(foreign_keys.len(), 1);
    assert_eq!(foreign_keys[0][2], Value::Text("sale_items".into()));
    assert_eq!(foreign_keys[0][3], Value::Text("sale_item_id".into()));
    assert_eq!(foreign_keys[0][4], Value::Text("id".into()));
    assert_eq!(foreign_keys[0][5], Value::Text("NO ACTION".into()));
    assert_eq!(foreign_keys[0][6], Value::Text("NO ACTION".into()));
}

fn insert_return_item(
    connection: &Connection,
    id: &[u8],
    sale_item_id: &[u8],
    quantity: i64,
) -> rusqlite::Result<usize> {
    connection.execute(
        "INSERT INTO main.sale_return_items (id, sale_item_id, quantity) VALUES (?1, ?2, ?3)",
        params![id, sale_item_id, quantity],
    )
}

fn seed_sale_item(connection: &Connection, sale_id: &[u8; 16], item_id: &[u8; 16]) {
    connection
        .execute(
            "INSERT INTO main.sales (id, status) VALUES (?1, 'finalized')",
            [sale_id.as_slice()],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO main.sale_items
                (id, sale_id, position, product_id, description, quantity,
                 unit_price_cents, discount_cents, total_cents)
             VALUES (?1, ?2, 0, NULL, 'Synthetic sale item', 3, 100, 0, 300)",
            params![item_id.as_slice(), sale_id.as_slice()],
        )
        .unwrap();
}

#[test]
fn ret004_migration_adds_returns_to_a_new_file_database() {
    let directory = TempDirectory::new();
    let path = directory.path().join("sale-returns.sqlite");
    let connection = migrate_v14(v13(&path));
    assert_health(&connection, 14);
    assert_return_item_schema(&connection);
    assert_eq!(
        rows(
            &connection,
            "SELECT type, name FROM main.sqlite_schema WHERE name IN ('sale_return_items', 'sale_return_items_sale_item_id_idx') ORDER BY type, name"
        ),
        vec![
            vec![Value::Text("index".into()), Value::Text("sale_return_items_sale_item_id_idx".into())],
            vec![Value::Text("table".into()), Value::Text("sale_return_items".into())]
        ]
    );
    drop(connection);

    let reopened = open_database(&path).unwrap();
    assert_health(&reopened, 14);
    assert_return_item_schema(&reopened);
    assert!(rows(&reopened, "SELECT * FROM main.sale_return_items").is_empty());
}

#[test]
fn ret004_migration_preserves_upgrade_and_partial_item_history() {
    let directory = TempDirectory::new();
    let path = directory.path().join("sale-returns-upgrade.sqlite");
    let connection = v13(&path);
    let sale_id = [31_u8; 16];
    let sale_item_id = [32_u8; 16];
    seed_sale_item(&connection, &sale_id, &sale_item_id);

    let schema_before = schema_snapshot(&connection);
    let sales_before = rows(&connection, "SELECT id, status FROM main.sales ORDER BY id");
    let items_before = rows(&connection, "SELECT * FROM main.sale_items ORDER BY id");
    drop(connection);

    let connection = migrate_v14(open_database(&path).unwrap());
    assert_health(&connection, 14);
    assert_return_item_schema(&connection);
    assert_eq!(
        rows(&connection, "SELECT id, status FROM main.sales ORDER BY id"),
        sales_before
    );
    assert_eq!(
        rows(&connection, "SELECT * FROM main.sale_items ORDER BY id"),
        items_before
    );

    let prior_objects = schema_snapshot(&connection)
        .into_iter()
        .filter(|row| {
            row[1] != Value::Text("sale_return_items".into())
                && row[1] != Value::Text("sale_return_items_sale_item_id_idx".into())
        })
        .collect::<Vec<_>>();
    assert_eq!(prior_objects, schema_before);

    let first_id = [41_u8; 16];
    let second_id = [42_u8; 16];
    assert_eq!(
        insert_return_item(&connection, &first_id, &sale_item_id, 1).unwrap(),
        1
    );
    assert_eq!(
        insert_return_item(&connection, &second_id, &sale_item_id, 2).unwrap(),
        1
    );
    assert!(connection.is_autocommit());

    let bad_id = [43_u8; 15];
    let bad_item_length = [44_u8; 15];
    let missing_item = [45_u8; 16];
    let bad_item_length_id = [46_u8; 16];
    let missing_item_id = [47_u8; 16];
    let zero_quantity_id = [48_u8; 16];
    let negative_quantity_id = [49_u8; 16];
    for (id, item_id, quantity) in [
        (bad_id.as_slice(), sale_item_id.as_slice(), 1),
        (bad_item_length_id.as_slice(), bad_item_length.as_slice(), 1),
        (missing_item_id.as_slice(), missing_item.as_slice(), 1),
        (zero_quantity_id.as_slice(), sale_item_id.as_slice(), 0),
        (negative_quantity_id.as_slice(), sale_item_id.as_slice(), -1),
        (first_id.as_slice(), sale_item_id.as_slice(), 1),
    ] {
        assert!(insert_return_item(&connection, id, item_id, quantity).is_err());
        assert!(connection.is_autocommit());
        assert_eq!(
            rows(&connection, "SELECT count(*) FROM main.sale_return_items"),
            vec![vec![Value::Integer(2)]]
        );
    }
    assert_health(&connection, 14);
    drop(connection);

    let reopened = open_database(&path).unwrap();
    assert_health(&reopened, 14);
    assert_eq!(
        rows(
            &reopened,
            "SELECT id, sale_item_id, quantity FROM main.sale_return_items ORDER BY id"
        ),
        vec![
            vec![
                Value::Blob(first_id.to_vec()),
                Value::Blob(sale_item_id.to_vec()),
                Value::Integer(1)
            ],
            vec![
                Value::Blob(second_id.to_vec()),
                Value::Blob(sale_item_id.to_vec()),
                Value::Integer(2)
            ]
        ]
    );
}

#[test]
fn ret004_migration_rolls_back_partial_ddl_and_allows_retry() {
    let directory = TempDirectory::new();
    let path = directory.path().join("sale-returns-rollback.sqlite");
    let connection = v13(&path);
    connection
        .execute(
            "CREATE INDEX main.sale_return_items_sale_item_id_idx ON sale_items (sale_id)",
            [],
        )
        .unwrap();
    let schema_before = schema_snapshot(&connection);
    drop(connection);

    let connection = open_database(&path).unwrap();
    let result = execute_versioned_migration(
        connection,
        SALE_RETURN_ITEMS_MIGRATION_VERSION,
        SALE_RETURN_ITEMS_MIGRATION_SQL,
    );
    assert!(matches!(result, Err(MigrationError::ApplyFailed)));

    let connection = open_database(&path).unwrap();
    assert_health(&connection, 13);
    assert_eq!(schema_snapshot(&connection), schema_before);
    connection
        .execute("DROP INDEX main.sale_return_items_sale_item_id_idx", [])
        .unwrap();
    let connection = migrate_v14(connection);
    assert_health(&connection, 14);
    assert_return_item_schema(&connection);
}
