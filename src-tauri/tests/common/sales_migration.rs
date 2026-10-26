use super::TempDirectory;
use millani_artes::database::{
    execute_versioned_migration, open_database, schema_version, MigrationError,
    ACCOUNTS_MIGRATION_SQL, AUDIT_EVENT_MIGRATION_SQL, CATEGORIES_MIGRATION_SQL,
    CUSTOMERS_MIGRATION_SQL, CUSTOMERS_MIGRATION_VERSION, MOVEMENTS_MIGRATION_SQL,
    MOVEMENT_CATEGORIES_MIGRATION_SQL, PRODUCTS_MIGRATION_SQL, PRODUCTS_MIGRATION_VERSION,
    RECURRENCES_MIGRATION_SQL, SALES_MIGRATION_SQL, SALES_MIGRATION_VERSION,
};
use rusqlite::{params, types::Value, Connection, Error, ErrorCode};
use std::path::Path;

fn rows(c: &Connection, sql: &str) -> Vec<Vec<Value>> {
    let mut s = c.prepare(sql).unwrap();
    let n = s.column_count();
    s.query_map([], |r| {
        (0..n)
            .map(|i| r.get(i))
            .collect::<rusqlite::Result<Vec<Value>>>()
    })
    .unwrap()
    .collect::<rusqlite::Result<_>>()
    .unwrap()
}
fn v9(path: &Path) -> Connection {
    let mut c = execute_versioned_migration(open_database(path).unwrap(), 1, "").unwrap();
    for (v, sql) in [
        (2, AUDIT_EVENT_MIGRATION_SQL),
        (3, ACCOUNTS_MIGRATION_SQL),
        (4, CATEGORIES_MIGRATION_SQL),
        (5, MOVEMENTS_MIGRATION_SQL),
        (6, MOVEMENT_CATEGORIES_MIGRATION_SQL),
        (7, RECURRENCES_MIGRATION_SQL),
        (PRODUCTS_MIGRATION_VERSION, PRODUCTS_MIGRATION_SQL),
        (CUSTOMERS_MIGRATION_VERSION, CUSTOMERS_MIGRATION_SQL),
    ] {
        c = execute_versioned_migration(c, v, sql).unwrap();
    }
    c
}
fn migrate(c: Connection) -> Connection {
    assert_eq!(SALES_MIGRATION_VERSION, 10);
    execute_versioned_migration(c, SALES_MIGRATION_VERSION, SALES_MIGRATION_SQL).unwrap()
}
fn health(c: &Connection, version: u32) {
    assert!(c.is_autocommit());
    assert_eq!(schema_version(c, version).unwrap(), version);
    assert_eq!(
        rows(c, "PRAGMA main.integrity_check"),
        vec![vec![Value::Text("ok".into())]]
    );
    assert!(rows(c, "PRAGMA main.foreign_key_check").is_empty());
    assert_eq!(
        rows(
            c,
            "SELECT version FROM main.schema_migration ORDER BY version"
        ),
        (1..=version)
            .map(|v| vec![Value::Integer(i64::from(v))])
            .collect::<Vec<_>>()
    );
}
fn old_data(c: &Connection) -> Vec<Vec<Vec<Value>>> {
    [
        "accounts",
        "categories",
        "movements",
        "audit_event",
        "recurrences",
        "recurrence_obligations",
        "products",
        "customers",
    ]
    .iter()
    .map(|table| rows(c, &format!("SELECT * FROM main.{table} ORDER BY 1")))
    .collect()
}
fn schema(c: &Connection) -> Vec<Vec<Value>> {
    rows(
        c,
        "SELECT type,name,tbl_name,sql FROM main.sqlite_schema ORDER BY type,name",
    )
}
fn seed(c: &Connection) {
    c.execute(
        "INSERT INTO main.accounts VALUES(?1,'Conta sintética','cash',-1234)",
        params![[1_u8; 16].as_slice()],
    )
    .unwrap();
    c.execute(
        "INSERT INTO main.categories VALUES(?1,'Categoria histórica',0)",
        params![[2_u8; 16].as_slice()],
    )
    .unwrap();
    c.execute("INSERT INTO main.movements(id,kind,amount_cents,context,account_id,category_id) VALUES(?1,'exit',7,'home',?2,?3)", params![[3_u8;16].as_slice(),[1_u8;16].as_slice(),[2_u8;16].as_slice()]).unwrap();
    c.execute(
        "INSERT INTO main.audit_event VALUES(?1,1234,'movement_exit_create','movement',?2)",
        params![[4_u8; 16].as_slice(), [3_u8; 16].as_slice()],
    )
    .unwrap();
    c.execute(
        "INSERT INTO main.recurrences VALUES(?1,'Recorrência','fixed',7,'home',31)",
        params![[5_u8; 16].as_slice()],
    )
    .unwrap();
    c.execute(
        "INSERT INTO main.recurrence_obligations VALUES(?1,?2,2026,2,1234,'paid',7,?3,'home')",
        params![
            [6_u8; 16].as_slice(),
            [5_u8; 16].as_slice(),
            [3_u8; 16].as_slice()
        ],
    )
    .unwrap();
    c.execute(
        "INSERT INTO main.products VALUES(?1,'Produto histórico',?2,0,0)",
        params![[7_u8; 16].as_slice(), "  descrição ‘Á’\r\n\0  "],
    )
    .unwrap();
    c.execute(
        "INSERT INTO main.customers VALUES(?1,'Cliente histórico',NULL)",
        params![[8_u8; 16].as_slice()],
    )
    .unwrap();
}

#[test]
fn sal010_empty_database_has_exact_strict_shape_and_roundtrips_states() {
    let d = TempDirectory::new();
    let p = d.path().join("sal010-empty.sqlite");
    let c = migrate(v9(&p));
    health(&c, 10);
    let columns = rows(&c, "PRAGMA main.table_xinfo('sales')");
    assert_eq!(columns.len(), 2);
    for (i, (name, kind)) in [("id", "BLOB"), ("status", "TEXT")].iter().enumerate() {
        assert_eq!(
            columns[i],
            vec![
                Value::Integer(i as i64),
                Value::Text((*name).into()),
                Value::Text((*kind).into()),
                Value::Integer(1),
                Value::Null,
                Value::Integer(i64::from(i == 0)),
                Value::Integer(0)
            ]
        );
    }
    assert_eq!(
        rows(
            &c,
            "SELECT strict FROM pragma_table_list WHERE schema='main' AND name='sales'"
        ),
        vec![vec![Value::Integer(1)]]
    );
    assert_eq!(rows(&c, "PRAGMA main.index_list('sales')").len(), 1);
    assert!(rows(&c, "PRAGMA main.foreign_key_list('sales')").is_empty());
    assert!(rows(&c, "SELECT * FROM main.sales").is_empty());
    for (id, status) in [(0_u8, "open"), (1, "finalized"), (u8::MAX, "open")] {
        c.execute(
            "INSERT INTO main.sales VALUES(?1,?2)",
            params![[id; 16].as_slice(), status],
        )
        .unwrap();
    }
    let saved = rows(&c, "SELECT id,status FROM main.sales ORDER BY id");
    assert_eq!(
        saved,
        [(0_u8, "open"), (1, "finalized"), (u8::MAX, "open")]
            .map(|(id, status)| vec![Value::Blob(vec![id; 16]), Value::Text(status.into())])
    );
    c.execute(
        "UPDATE main.sales SET status='finalized' WHERE id=?1",
        params![[0_u8; 16].as_slice()],
    )
    .unwrap();
    assert_eq!(
        rows(&c, "SELECT status FROM main.sales WHERE id=zeroblob(16)"),
        vec![vec![Value::Text("finalized".into())]]
    );
    c.execute(
        "UPDATE main.sales SET status='open' WHERE id=?1",
        params![[0_u8; 16].as_slice()],
    )
    .unwrap();
    assert_eq!(
        rows(&c, "SELECT id,status FROM main.sales ORDER BY id"),
        saved
    );
    drop(c);
    let c = open_database(&p).unwrap();
    health(&c, 10);
    assert_eq!(
        rows(&c, "SELECT id,status FROM main.sales ORDER BY id"),
        saved
    );
}
#[test]
fn sal010_constraints_reject_invalid_inserts_updates_and_duplicates_without_mutation() {
    let d = TempDirectory::new();
    let c = migrate(v9(&d.path().join("sal010-constraints.sqlite")));
    c.execute(
        "INSERT INTO main.sales VALUES(?1,'open')",
        params![[1_u8; 16].as_slice()],
    )
    .unwrap();
    let before = rows(&c, "SELECT * FROM main.sales ORDER BY id");
    let valid = vec![Value::Blob(vec![2_u8; 16]), Value::Text("open".into())];
    let mut bad_values = vec![
        (0, Value::Null),
        (0, Value::Blob(vec![])),
        (0, Value::Blob(vec![2; 15])),
        (0, Value::Blob(vec![2; 17])),
        (0, Value::Text("1234567890123456".into())),
        (0, Value::Integer(2)),
        (1, Value::Null),
        (1, Value::Blob(b"open".to_vec())),
        (1, Value::Integer(1)),
        (1, Value::Real(1.5)),
    ];
    for state in [
        "",
        "paid",
        "Open",
        "FINALIZED",
        " open",
        "open ",
        "open\r\n",
        "open\0",
        "open\0tail",
        "finalized\0",
        "ｏｐｅｎ",
        "aberta",
    ] {
        bad_values.push((1, Value::Text(state.into())));
    }
    for (index, bad) in bad_values {
        let mut row = valid.clone();
        row[index] = bad;
        for update in [false, true] {
            let result = if update {
                c.execute(
                    "UPDATE main.sales SET id=?1,status=?2 WHERE id=?3",
                    rusqlite::params_from_iter(
                        row.iter().cloned().chain([Value::Blob(vec![1_u8; 16])]),
                    ),
                )
            } else {
                c.execute(
                    "INSERT INTO main.sales VALUES(?1,?2)",
                    rusqlite::params_from_iter(row.iter()),
                )
            };
            assert!(
                matches!(result,Err(Error::SqliteFailure(ref e,_))if e.code==ErrorCode::ConstraintViolation),
                "SALES_INVALID_ACCEPTED:{index}"
            );
            assert_eq!(rows(&c, "SELECT * FROM main.sales ORDER BY id"), before);
            assert!(c.is_autocommit());
        }
    }
    assert!(
        matches!(c.execute("INSERT INTO main.sales VALUES(?1,'finalized')",params![[1_u8;16].as_slice()]),Err(Error::SqliteFailure(ref e,_))if e.code==ErrorCode::ConstraintViolation)
    );
    assert_eq!(rows(&c, "SELECT * FROM main.sales ORDER BY id"), before);
    c.execute(
        "INSERT INTO main.sales VALUES(?1,'finalized')",
        params![[3_u8; 16].as_slice()],
    )
    .unwrap();
    assert_eq!(rows(&c, "SELECT * FROM main.sales").len(), 2);
    health(&c, 10);
}
#[test]
fn sal010_upgrade_preserves_all_existing_history_and_reopens() {
    let d = TempDirectory::new();
    let p = d.path().join("sal010-upgrade.sqlite");
    let c = v9(&p);
    seed(&c);
    let before = old_data(&c);
    let old_schema = schema(&c);
    let c = migrate(c);
    assert_eq!(old_data(&c), before);
    health(&c, 10);
    assert!(old_schema.iter().all(|r| schema(&c).contains(r)));
    assert!(rows(&c, "SELECT * FROM main.sales").is_empty());
    drop(c);
    let c = open_database(&p).unwrap();
    health(&c, 10);
    assert_eq!(old_data(&c), before);
    assert!(old_schema.iter().all(|r| schema(&c).contains(r)));
}
#[test]
fn sal010_ledger_failure_rolls_back_sales_ddl_and_allows_retry() {
    let d = TempDirectory::new();
    let p = d.path().join("sal010-rollback.sqlite");
    let c = v9(&p);
    seed(&c);
    c.execute_batch("CREATE TRIGGER main.sal010_ledger_fail BEFORE INSERT ON schema_migration WHEN NEW.version=10 BEGIN SELECT RAISE(ABORT,'synthetic migration fault'); END;").unwrap();
    let before = old_data(&c);
    let old_schema = schema(&c);
    assert!(matches!(
        execute_versioned_migration(c, 10, SALES_MIGRATION_SQL),
        Err(MigrationError::ApplyFailed)
    ));
    let c = open_database(&p).unwrap();
    health(&c, 9);
    assert_eq!(old_data(&c), before);
    assert_eq!(schema(&c), old_schema);
    assert_eq!(
        rows(
            &c,
            "SELECT count(*) FROM main.sqlite_schema WHERE name='sales'"
        ),
        vec![vec![Value::Integer(0)]]
    );
    c.execute_batch("DROP TRIGGER main.sal010_ledger_fail;")
        .unwrap();
    let c = migrate(c);
    health(&c, 10);
    assert_eq!(old_data(&c), before);
    drop(c);
    health(&open_database(&p).unwrap(), 10);
}
#[test]
fn sal010_name_collision_and_duplicate_version_preserve_schema_data_and_ledger() {
    let d = TempDirectory::new();
    let p = d.path().join("sal010-collision.sqlite");
    let c = v9(&p);
    seed(&c);
    c.execute_batch("CREATE TABLE main.sales(sentinel TEXT); INSERT INTO main.sales VALUES('synthetic sentinel');").unwrap();
    let before = old_data(&c);
    let old_schema = schema(&c);
    assert!(matches!(
        execute_versioned_migration(c, 10, SALES_MIGRATION_SQL),
        Err(MigrationError::ApplyFailed)
    ));
    let c = open_database(&p).unwrap();
    health(&c, 9);
    assert_eq!(schema(&c), old_schema);
    assert_eq!(old_data(&c), before);
    assert_eq!(
        rows(&c, "SELECT * FROM main.sales"),
        vec![vec![Value::Text("synthetic sentinel".into())]]
    );
    c.execute_batch("DROP TABLE main.sales;").unwrap();
    let c = migrate(c);
    let complete = schema(&c);
    assert!(matches!(
        execute_versioned_migration(c, 10, SALES_MIGRATION_SQL),
        Err(MigrationError::InvalidVersion)
    ));
    let c = open_database(&p).unwrap();
    health(&c, 10);
    assert_eq!(schema(&c), complete);
    assert_eq!(old_data(&c), before);
}
