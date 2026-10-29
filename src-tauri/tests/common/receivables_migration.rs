use super::TempDirectory;
use millani_artes::database::{
    execute_versioned_migration, open_database, schema_version, MigrationError,
    ACCOUNTS_MIGRATION_SQL, AUDIT_EVENT_MIGRATION_SQL, CATEGORIES_MIGRATION_SQL,
    CUSTOMERS_MIGRATION_SQL, MOVEMENTS_MIGRATION_SQL, MOVEMENT_CATEGORIES_MIGRATION_SQL,
    PRODUCTS_MIGRATION_SQL, RECEIVABLES_MIGRATION_SQL, RECEIVABLES_MIGRATION_VERSION,
    RECURRENCES_MIGRATION_SQL, SALES_MIGRATION_SQL, SALE_ITEMS_MIGRATION_SQL,
};
use rusqlite::{params, types::Value, Connection};
use std::path::Path;

fn rows(connection: &Connection, sql: &str) -> Vec<Vec<Value>> {
    let mut statement = connection.prepare(sql).unwrap();
    let columns = statement.column_count();
    statement
        .query_map([], |row| {
            (0..columns)
                .map(|index| row.get(index))
                .collect::<rusqlite::Result<Vec<Value>>>()
        })
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap()
}

fn v11(path: &Path) -> Connection {
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
    ] {
        connection = execute_versioned_migration(connection, version, sql).unwrap();
    }
    assert_eq!(schema_version(&connection, 11).unwrap(), 11);
    connection
}

fn migrate(path: &Path) -> Connection {
    let connection = v11(path);
    assert_eq!(RECEIVABLES_MIGRATION_VERSION, 12);
    execute_versioned_migration(
        connection,
        RECEIVABLES_MIGRATION_VERSION,
        RECEIVABLES_MIGRATION_SQL,
    )
    .unwrap()
}

fn health(connection: &Connection, version: u32) {
    assert!(connection.is_autocommit());
    assert_eq!(schema_version(connection, version).unwrap(), version);
    assert_eq!(
        rows(connection, "PRAGMA main.integrity_check"),
        vec![vec![Value::Text("ok".into())]]
    );
    assert!(rows(connection, "PRAGMA main.foreign_key_check").is_empty());
    assert_eq!(
        rows(
            connection,
            "SELECT version FROM main.schema_migration ORDER BY version"
        ),
        (1..=version)
            .map(|value| vec![Value::Integer(i64::from(value))])
            .collect::<Vec<_>>()
    );
}

fn insert_customer(connection: &Connection, id: [u8; 16]) {
    connection
        .execute(
            "INSERT INTO main.customers (id, name, phone) VALUES (?1, 'Cliente sintético', NULL)",
            [id.as_slice()],
        )
        .unwrap();
}

fn insert_sale(connection: &Connection, id: [u8; 16], status: &str) {
    connection
        .execute(
            "INSERT INTO main.sales (id, status) VALUES (?1, ?2)",
            params![id.as_slice(), status],
        )
        .unwrap();
}

fn insert_receivable(
    connection: &Connection,
    id: [u8; 16],
    sale_id: [u8; 16],
    amount_cents: i64,
    customer_id: [u8; 16],
    due_at_ms: i64,
) -> rusqlite::Result<usize> {
    connection.execute(
        "INSERT INTO main.receivables (id, sale_id, amount_cents, customer_id, due_at_ms) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id.as_slice(), sale_id.as_slice(), amount_cents, customer_id.as_slice(), due_at_ms],
    )
}

fn receivables(connection: &Connection) -> Vec<Vec<Value>> {
    rows(
        connection,
        "SELECT id, sale_id, amount_cents, customer_id, due_at_ms FROM main.receivables ORDER BY id",
    )
}

fn assert_rejected_without_mutation(connection: &Connection, sql: &str, expected: &[Vec<Value>]) {
    assert!(
        connection.execute(sql, []).is_err(),
        "INVALID_RECEIVABLE_WRITE_ACCEPTED: {sql}"
    );
    assert!(connection.is_autocommit());
    assert_eq!(receivables(connection), expected);
    assert_eq!(
        rows(
            connection,
            "SELECT version FROM main.schema_migration ORDER BY version"
        ),
        (1..=12)
            .map(|value| vec![Value::Integer(value)])
            .collect::<Vec<_>>()
    );
    assert_eq!(
        rows(connection, "PRAGMA main.integrity_check"),
        vec![vec![Value::Text("ok".into())]]
    );
    assert!(rows(connection, "PRAGMA main.foreign_key_check").is_empty());
}

fn sql_blob(byte: u8) -> String {
    format!("x'{}'", format!("{byte:02x}").repeat(16))
}

fn schema_without_receivables(connection: &Connection) -> Vec<Vec<Value>> {
    rows(
        connection,
        "SELECT type, name, tbl_name, sql FROM main.sqlite_schema WHERE tbl_name <> 'receivables' ORDER BY type, name",
    )
}

fn previous_data(connection: &Connection) -> Vec<Vec<Vec<Value>>> {
    [
        "accounts",
        "categories",
        "movements",
        "audit_event",
        "recurrences",
        "recurrence_obligations",
        "products",
        "customers",
        "sales",
        "sale_items",
    ]
    .iter()
    .map(|table| {
        rows(
            connection,
            &format!("SELECT * FROM main.{table} ORDER BY 1"),
        )
    })
    .collect()
}

fn seed_previous_data(connection: &Connection) {
    connection
        .execute(
            "INSERT INTO main.accounts VALUES (?1, 'Conta sintética', 'cash', -1234)",
            params![[1_u8; 16].as_slice()],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO main.categories VALUES (?1, 'Categoria histórica', 0)",
            params![[2_u8; 16].as_slice()],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO main.movements (id, kind, amount_cents, context, account_id, category_id) VALUES (?1, 'exit', 7, 'home', ?2, ?3)",
            params![[3_u8; 16].as_slice(), [1_u8; 16].as_slice(), [2_u8; 16].as_slice()],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO main.audit_event VALUES (?1, 1234, 'movement_exit_create', 'movement', ?2)",
            params![[4_u8; 16].as_slice(), [3_u8; 16].as_slice()],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO main.recurrences VALUES (?1, 'Recorrência', 'fixed', 7, 'home', 31)",
            params![[5_u8; 16].as_slice()],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO main.recurrence_obligations VALUES (?1, ?2, 2026, 2, 1234, 'paid', 7, ?3, 'home')",
            params![[6_u8; 16].as_slice(), [5_u8; 16].as_slice(), [3_u8; 16].as_slice()],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO main.products VALUES (?1, 'Produto histórico', ?2, 0, 0)",
            params![[7_u8; 16].as_slice(), "Descrição sintética"],
        )
        .unwrap();
    insert_customer(connection, [8_u8; 16]);
    insert_sale(connection, [9_u8; 16], "open");
    connection
        .execute(
            "INSERT INTO main.sale_items VALUES (?1, ?2, 0, NULL, 'Item histórico', 1, 7, 0, 7)",
            params![[10_u8; 16].as_slice(), [9_u8; 16].as_slice()],
        )
        .unwrap();
}

fn assert_receivables_schema(connection: &Connection) {
    assert_eq!(
        rows(connection, "PRAGMA main.table_xinfo('receivables')"),
        vec![
            vec![
                Value::Integer(0),
                Value::Text("id".into()),
                Value::Text("BLOB".into()),
                Value::Integer(1),
                Value::Null,
                Value::Integer(1),
                Value::Integer(0)
            ],
            vec![
                Value::Integer(1),
                Value::Text("sale_id".into()),
                Value::Text("BLOB".into()),
                Value::Integer(1),
                Value::Null,
                Value::Integer(0),
                Value::Integer(0)
            ],
            vec![
                Value::Integer(2),
                Value::Text("amount_cents".into()),
                Value::Text("INTEGER".into()),
                Value::Integer(1),
                Value::Null,
                Value::Integer(0),
                Value::Integer(0)
            ],
            vec![
                Value::Integer(3),
                Value::Text("customer_id".into()),
                Value::Text("BLOB".into()),
                Value::Integer(1),
                Value::Null,
                Value::Integer(0),
                Value::Integer(0)
            ],
            vec![
                Value::Integer(4),
                Value::Text("due_at_ms".into()),
                Value::Text("INTEGER".into()),
                Value::Integer(1),
                Value::Null,
                Value::Integer(0),
                Value::Integer(0)
            ],
        ]
    );
    let table_list = rows(connection, "PRAGMA main.table_list");
    let table = table_list
        .iter()
        .find(|row| {
            row[0] == Value::Text("main".into()) && row[1] == Value::Text("receivables".into())
        })
        .unwrap();
    assert_eq!(table[5], Value::Integer(1));

    let mut statement = connection
        .prepare("PRAGMA main.index_list('receivables')")
        .unwrap();
    let mut indexes = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
            ))
        })
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap();
    indexes.sort();
    assert_eq!(indexes, [(1, "pk".into(), 0), (1, "u".into(), 0)]);

    let mut statement = connection
        .prepare("PRAGMA main.foreign_key_list('receivables')")
        .unwrap();
    let mut foreign_keys = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
            ))
        })
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap();
    foreign_keys.sort();
    assert_eq!(
        foreign_keys,
        [
            (
                "customers".into(),
                "customer_id".into(),
                "id".into(),
                "NO ACTION".into(),
                "NO ACTION".into()
            ),
            (
                "sales".into(),
                "sale_id".into(),
                "id".into(),
                "NO ACTION".into(),
                "NO ACTION".into()
            )
        ]
    );

    let sql: String = connection
        .query_row(
            "SELECT sql FROM main.sqlite_schema WHERE type = 'table' AND name = 'receivables'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    for declaration in [
        "receivables_id_length",
        "receivables_sale_id_length",
        "receivables_amount_positive",
        "receivables_customer_id_length",
        "receivables_sale_id_fkey",
        "receivables_customer_id_fkey",
        "receivables_sale_id_unique",
    ] {
        assert!(
            sql.contains(declaration),
            "MISSING_CONSTRAINT_NAME: {declaration}"
        );
    }
}

#[test]
fn rcv002_empty_database_creates_exact_strict_schema_at_version_twelve() {
    let directory = TempDirectory::new();
    let path = directory.path().join("rcv002-empty.sqlite");
    let connection = migrate(&path);

    health(&connection, 12);
    assert_receivables_schema(&connection);
    assert!(receivables(&connection).is_empty());
}

#[test]
fn rcv002_roundtrips_extreme_values_without_enforcing_sale_finalization() {
    let directory = TempDirectory::new();
    let path = directory.path().join("rcv002-roundtrip.sqlite");
    let connection = migrate(&path);
    insert_customer(&connection, [0xa5; 16]);
    insert_sale(&connection, [0xff; 16], "open");
    insert_receivable(
        &connection,
        [0; 16],
        [0xff; 16],
        i64::MAX,
        [0xa5; 16],
        i64::MIN,
    )
    .unwrap();
    drop(connection);

    let reopened = open_database(&path).unwrap();
    health(&reopened, 12);
    assert_receivables_schema(&reopened);
    assert_eq!(
        receivables(&reopened),
        vec![vec![
            Value::Blob(vec![0; 16]),
            Value::Blob(vec![0xff; 16]),
            Value::Integer(i64::MAX),
            Value::Blob(vec![0xa5; 16]),
            Value::Integer(i64::MIN),
        ]]
    );
}

#[test]
fn rcv002_constraints_reject_invalid_inserts_and_updates_without_mutation() {
    let directory = TempDirectory::new();
    let path = directory.path().join("rcv002-constraints.sqlite");
    let connection = migrate(&path);
    insert_customer(&connection, [0x21; 16]);
    insert_sale(&connection, [0x22; 16], "finalized");
    insert_sale(&connection, [0x23; 16], "finalized");
    insert_sale(&connection, [0x24; 16], "finalized");
    insert_receivable(&connection, [0x31; 16], [0x22; 16], 100, [0x21; 16], 50).unwrap();
    insert_receivable(&connection, [0x32; 16], [0x23; 16], 200, [0x21; 16], 60).unwrap();
    let baseline = receivables(&connection);

    let invalid_inserts = [
        format!("NULL, {}, 1, {}, 0", sql_blob(0x24), sql_blob(0x21)),
        format!(
            "'abcdefghijklmnop', {}, 1, {}, 0",
            sql_blob(0x24),
            sql_blob(0x21)
        ),
        format!("zeroblob(15), {}, 1, {}, 0", sql_blob(0x24), sql_blob(0x21)),
        format!("zeroblob(17), {}, 1, {}, 0", sql_blob(0x24), sql_blob(0x21)),
        format!("{}, NULL, 1, {}, 0", sql_blob(0x24), sql_blob(0x21)),
        format!("{}, zeroblob(15), 1, {}, 0", sql_blob(0x24), sql_blob(0x21)),
        format!("{}, zeroblob(17), 1, {}, 0", sql_blob(0x24), sql_blob(0x21)),
        format!(
            "{}, {}, NULL, {}, 0",
            sql_blob(0x24),
            sql_blob(0x24),
            sql_blob(0x21)
        ),
        format!(
            "{}, {}, 0, {}, 0",
            sql_blob(0x24),
            sql_blob(0x24),
            sql_blob(0x21)
        ),
        format!(
            "{}, {}, -1, {}, 0",
            sql_blob(0x24),
            sql_blob(0x24),
            sql_blob(0x21)
        ),
        format!(
            "{}, {}, 1.5, {}, 0",
            sql_blob(0x24),
            sql_blob(0x24),
            sql_blob(0x21)
        ),
        format!(
            "{}, {}, 'not-cents', {}, 0",
            sql_blob(0x24),
            sql_blob(0x24),
            sql_blob(0x21)
        ),
        format!("{}, {}, 1, NULL, 0", sql_blob(0x24), sql_blob(0x24)),
        format!("{}, {}, 1, zeroblob(15), 0", sql_blob(0x24), sql_blob(0x24)),
        format!("{}, {}, 1, zeroblob(17), 0", sql_blob(0x24), sql_blob(0x24)),
        format!(
            "{}, {}, 1, {}, NULL",
            sql_blob(0x24),
            sql_blob(0x24),
            sql_blob(0x21)
        ),
        format!(
            "{}, {}, 1, {}, 1.5",
            sql_blob(0x24),
            sql_blob(0x24),
            sql_blob(0x21)
        ),
        format!(
            "{}, {}, 1, {}, 'not-time'",
            sql_blob(0x24),
            sql_blob(0x24),
            sql_blob(0x21)
        ),
        format!(
            "{}, {}, 1, {}, 0",
            sql_blob(0x25),
            sql_blob(0x55),
            sql_blob(0x21)
        ),
        format!(
            "{}, {}, 1, {}, 0",
            sql_blob(0x26),
            sql_blob(0x24),
            sql_blob(0x66)
        ),
        format!(
            "{}, {}, 1, {}, 0",
            sql_blob(0x31),
            sql_blob(0x24),
            sql_blob(0x21)
        ),
        format!(
            "{}, {}, 1, {}, 0",
            sql_blob(0x33),
            sql_blob(0x22),
            sql_blob(0x21)
        ),
    ];
    for values in invalid_inserts {
        assert_rejected_without_mutation(
            &connection,
            &format!("INSERT INTO main.receivables VALUES ({values})"),
            &baseline,
        );
    }

    let id = sql_blob(0x31);
    let invalid_updates = [
        format!("UPDATE main.receivables SET id = zeroblob(15) WHERE id = {id}"),
        format!("UPDATE main.receivables SET sale_id = zeroblob(15) WHERE id = {id}"),
        format!("UPDATE main.receivables SET amount_cents = 0 WHERE id = {id}"),
        format!(
            "UPDATE main.receivables SET sale_id = {} WHERE id = {id}",
            sql_blob(0x23)
        ),
        format!(
            "UPDATE main.receivables SET customer_id = {} WHERE id = {id}",
            sql_blob(0x66)
        ),
        format!("UPDATE main.receivables SET due_at_ms = 1.5 WHERE id = {id}"),
    ];
    for sql in invalid_updates {
        assert_rejected_without_mutation(&connection, &sql, &baseline);
    }
}

#[test]
fn rcv002_foreign_keys_reference_sale_and_customer_and_restrict_mutation() {
    let directory = TempDirectory::new();
    let path = directory.path().join("rcv002-foreign-keys.sqlite");
    let connection = migrate(&path);
    insert_customer(&connection, [0x41; 16]);
    insert_sale(&connection, [0x42; 16], "finalized");
    insert_receivable(&connection, [0x43; 16], [0x42; 16], 1, [0x41; 16], 0).unwrap();
    let baseline = receivables(&connection);
    assert_receivables_schema(&connection);

    for sql in [
        "DELETE FROM main.customers WHERE id = x'41414141414141414141414141414141'",
        "DELETE FROM main.sales WHERE id = x'42424242424242424242424242424242'",
        "UPDATE main.customers SET id = x'44444444444444444444444444444444' WHERE id = x'41414141414141414141414141414141'",
        "UPDATE main.sales SET id = x'45454545454545454545454545454545' WHERE id = x'42424242424242424242424242424242'",
    ] {
        assert!(connection.execute(sql, []).is_err());
        assert_eq!(receivables(&connection), baseline);
        assert_eq!(
            connection
                .query_row("SELECT count(*) FROM main.customers", [], |row| row.get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(
            connection
                .query_row("SELECT count(*) FROM main.sales", [], |row| row.get::<_, i64>(0))
                .unwrap(),
            1
        );
        health(&connection, 12);
    }
}

#[test]
fn rcv002_upgrade_from_v11_preserves_schema_data_and_ledger_after_reopen() {
    let directory = TempDirectory::new();
    let path = directory.path().join("rcv002-upgrade.sqlite");
    let connection = v11(&path);
    seed_previous_data(&connection);
    let old_schema = schema_without_receivables(&connection);
    let old_data = previous_data(&connection);

    let connection = execute_versioned_migration(
        connection,
        RECEIVABLES_MIGRATION_VERSION,
        RECEIVABLES_MIGRATION_SQL,
    )
    .unwrap();
    health(&connection, 12);
    assert_receivables_schema(&connection);
    assert_eq!(schema_without_receivables(&connection), old_schema);
    assert_eq!(previous_data(&connection), old_data);
    assert!(receivables(&connection).is_empty());
    assert_eq!(
        connection
            .query_row("SELECT count(*) FROM main.movements", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
    drop(connection);

    let reopened = open_database(&path).unwrap();
    health(&reopened, 12);
    assert_receivables_schema(&reopened);
    assert_eq!(schema_without_receivables(&reopened), old_schema);
    assert_eq!(previous_data(&reopened), old_data);
    assert!(receivables(&reopened).is_empty());
}

#[test]
fn rcv002_ledger_failure_rolls_back_ddl_and_retry_succeeds() {
    let directory = TempDirectory::new();
    let path = directory.path().join("rcv002-ledger-failure.sqlite");
    let connection = v11(&path);
    connection
        .execute_batch(
            "CREATE TRIGGER main.rcv002_ledger_fail BEFORE INSERT ON main.schema_migration WHEN NEW.version = 12 BEGIN SELECT RAISE(ABORT, 'RCV002 synthetic ledger failure'); END;",
        )
        .unwrap();
    let old_schema = schema_without_receivables(&connection);
    assert!(matches!(
        execute_versioned_migration(
            connection,
            RECEIVABLES_MIGRATION_VERSION,
            RECEIVABLES_MIGRATION_SQL
        ),
        Err(MigrationError::ApplyFailed)
    ));

    let connection = open_database(&path).unwrap();
    health(&connection, 11);
    assert_eq!(schema_without_receivables(&connection), old_schema);
    assert_eq!(
        connection
            .query_row(
                "SELECT count(*) FROM main.sqlite_schema WHERE type = 'table' AND name = 'receivables'",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        0
    );
    connection
        .execute_batch("DROP TRIGGER main.rcv002_ledger_fail")
        .unwrap();
    let connection = execute_versioned_migration(
        connection,
        RECEIVABLES_MIGRATION_VERSION,
        RECEIVABLES_MIGRATION_SQL,
    )
    .unwrap();
    health(&connection, 12);
    assert_receivables_schema(&connection);
}

#[test]
fn rcv002_name_and_version_collisions_do_not_apply_partial_migration() {
    let directory = TempDirectory::new();
    let path = directory.path().join("rcv002-name-collision.sqlite");
    let connection = v11(&path);
    connection
        .execute_batch(
            "CREATE TABLE main.receivables (sentinel INTEGER NOT NULL) STRICT; INSERT INTO main.receivables VALUES (42);",
        )
        .unwrap();
    let old_schema = schema_without_receivables(&connection);
    assert!(matches!(
        execute_versioned_migration(
            connection,
            RECEIVABLES_MIGRATION_VERSION,
            RECEIVABLES_MIGRATION_SQL
        ),
        Err(MigrationError::ApplyFailed)
    ));

    let connection = open_database(&path).unwrap();
    health(&connection, 11);
    assert_eq!(schema_without_receivables(&connection), old_schema);
    assert_eq!(
        connection
            .query_row("SELECT sentinel FROM main.receivables", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        42
    );
    drop(connection);

    let version_path = directory.path().join("rcv002-version-collision.sqlite");
    let connection = migrate(&version_path);
    assert!(matches!(
        execute_versioned_migration(
            connection,
            RECEIVABLES_MIGRATION_VERSION,
            RECEIVABLES_MIGRATION_SQL
        ),
        Err(MigrationError::InvalidVersion)
    ));
    let reopened = open_database(&version_path).unwrap();
    health(&reopened, 12);
    assert_receivables_schema(&reopened);
}
