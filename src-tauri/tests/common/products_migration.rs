use super::TempDirectory;
use millani_artes::database::{
    execute_versioned_migration, open_database, schema_version, MigrationError,
    ACCOUNTS_MIGRATION_SQL, AUDIT_EVENT_MIGRATION_SQL, CATEGORIES_MIGRATION_SQL,
    MOVEMENTS_MIGRATION_SQL, MOVEMENT_CATEGORIES_MIGRATION_SQL, PRODUCTS_MIGRATION_SQL,
    PRODUCTS_MIGRATION_VERSION, RECURRENCES_MIGRATION_SQL,
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
fn v7(path: &Path) -> Connection {
    let mut c = execute_versioned_migration(open_database(path).unwrap(), 1, "").unwrap();
    for (v, sql) in [
        (2, AUDIT_EVENT_MIGRATION_SQL),
        (3, ACCOUNTS_MIGRATION_SQL),
        (4, CATEGORIES_MIGRATION_SQL),
        (5, MOVEMENTS_MIGRATION_SQL),
        (6, MOVEMENT_CATEGORIES_MIGRATION_SQL),
        (7, RECURRENCES_MIGRATION_SQL),
    ] {
        c = execute_versioned_migration(c, v, sql).unwrap();
    }
    c
}
fn migrate(c: Connection) -> Connection {
    assert_eq!(PRODUCTS_MIGRATION_VERSION, 8);
    execute_versioned_migration(c, PRODUCTS_MIGRATION_VERSION, PRODUCTS_MIGRATION_SQL).unwrap()
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
}

#[test]
fn prd004_empty_database_has_exact_strict_product_shape_and_roundtrips_bounds() {
    let d = TempDirectory::new();
    let p = d.path().join("prd004-empty.sqlite");
    let c = migrate(v7(&p));
    health(&c, 8);
    let columns = rows(&c, "PRAGMA main.table_xinfo('products')");
    assert_eq!(columns.len(), 5);
    for (i, (name, kind)) in [
        ("id", "BLOB"),
        ("name", "TEXT"),
        ("description", "TEXT"),
        ("suggested_price_cents", "INTEGER"),
        ("is_active", "INTEGER"),
    ]
    .iter()
    .enumerate()
    {
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
            "SELECT strict FROM pragma_table_list WHERE schema='main' AND name='products'"
        ),
        vec![vec![Value::Integer(1)]]
    );
    assert_eq!(rows(&c, "PRAGMA main.index_list('products')").len(), 1);
    assert!(rows(&c, "PRAGMA main.foreign_key_list('products')").is_empty());
    for (id, price, active) in [(0_u8, 0, 0), (1, 1, 1), (2, i64::MAX, 0)] {
        c.execute(
            "INSERT INTO main.products VALUES(?1,?2,?3,?4,?5)",
            params![[id; 16].as_slice(), "\0Peça ‘azul’  ", "", price, active],
        )
        .unwrap();
    }
    c.execute(
        "UPDATE main.products SET description=?1 WHERE id=?2",
        params!["  descrição\r\n'livre'\0  ", [1_u8; 16].as_slice()],
    )
    .unwrap();
    let saved = rows(&c, "SELECT * FROM main.products ORDER BY id");
    assert_eq!(saved.len(), 3);
    assert_eq!(
        saved[0],
        vec![
            Value::Blob(vec![0; 16]),
            Value::Text("\0Peça ‘azul’  ".into()),
            Value::Text("".into()),
            Value::Integer(0),
            Value::Integer(0)
        ]
    );
    assert_eq!(
        saved[1][2],
        Value::Text("  descrição\r\n'livre'\0  ".into())
    );
    assert_eq!(saved[2][3], Value::Integer(i64::MAX));
    drop(c);
    let c = open_database(&p).unwrap();
    health(&c, 8);
    assert_eq!(rows(&c, "SELECT * FROM main.products ORDER BY id"), saved);
}

#[test]
fn prd004_product_constraints_reject_invalid_inserts_and_updates_without_mutation() {
    let d = TempDirectory::new();
    let c = migrate(v7(&d.path().join("prd004-constraints.sqlite")));
    c.execute(
        "INSERT INTO main.products VALUES(?1,'P','',0,1)",
        params![[1_u8; 16].as_slice()],
    )
    .unwrap();
    let before = rows(&c, "SELECT * FROM main.products ORDER BY id");
    let valid = vec![
        Value::Blob(vec![2_u8; 16]),
        Value::Text("P".into()),
        Value::Text("".into()),
        Value::Integer(0),
        Value::Integer(1),
    ];
    for (index, bad) in [
        (0, Value::Null),
        (0, Value::Blob(vec![2_u8; 15])),
        (0, Value::Blob(vec![2_u8; 17])),
        (0, Value::Text("1234567890123456".into())),
        (0, Value::Integer(2)),
        (1, Value::Null),
        (1, Value::Text("".into())),
        (1, Value::Blob(vec![1_u8])),
        (2, Value::Null),
        (2, Value::Blob(vec![1_u8])),
        (3, Value::Null),
        (3, Value::Integer(-1)),
        (3, Value::Integer(i64::MIN)),
        (3, Value::Real(1.5)),
        (3, Value::Text("free".into())),
        (3, Value::Blob(vec![1_u8])),
        (4, Value::Null),
        (4, Value::Integer(-1)),
        (4, Value::Integer(2)),
        (4, Value::Real(0.5)),
        (4, Value::Text("active".into())),
    ] {
        let mut row = valid.clone();
        row[index] = bad;
        for update in [false, true] {
            let result = if update {
                c.execute("UPDATE main.products SET id=?1,name=?2,description=?3,suggested_price_cents=?4,is_active=?5 WHERE id=?6",rusqlite::params_from_iter(row.iter().cloned().chain([Value::Blob(vec![1_u8;16])])))
            } else {
                c.execute(
                    "INSERT INTO main.products VALUES(?1,?2,?3,?4,?5)",
                    rusqlite::params_from_iter(row.iter()),
                )
            };
            assert!(
                matches!(result,Err(Error::SqliteFailure(ref error,_)) if error.code==ErrorCode::ConstraintViolation),
                "PRODUCT_INVALID_ACCEPTED:{index}"
            );
            assert_eq!(rows(&c, "SELECT * FROM main.products ORDER BY id"), before);
        }
    }
    assert!(
        matches!(c.execute("INSERT INTO main.products VALUES(?1,'duplicate','',0,1)",params![[1_u8;16].as_slice()]),Err(Error::SqliteFailure(ref e,_))if e.code==ErrorCode::ConstraintViolation)
    );
    assert_eq!(rows(&c, "SELECT * FROM main.products ORDER BY id"), before);
    health(&c, 8);
}

#[test]
fn prd004_upgrade_preserves_all_existing_history_and_reopens() {
    let d = TempDirectory::new();
    let p = d.path().join("prd004-upgrade.sqlite");
    let c = v7(&p);
    seed(&c);
    let before = old_data(&c);
    let old_schema = schema(&c);
    let c = migrate(c);
    assert_eq!(old_data(&c), before);
    health(&c, 8);
    assert!(old_schema.iter().all(|r| schema(&c).contains(r)));
    assert!(rows(&c, "SELECT * FROM main.products").is_empty());
    drop(c);
    let c = open_database(&p).unwrap();
    health(&c, 8);
    assert_eq!(old_data(&c), before);
    assert!(old_schema.iter().all(|r| schema(&c).contains(r)));
}

#[test]
fn prd004_ledger_failure_rolls_back_product_ddl_and_allows_retry() {
    let d = TempDirectory::new();
    let p = d.path().join("prd004-rollback.sqlite");
    let c = v7(&p);
    seed(&c);
    c.execute_batch("CREATE TRIGGER main.prd004_ledger_fail BEFORE INSERT ON schema_migration WHEN NEW.version=8 BEGIN SELECT RAISE(ABORT,'synthetic migration fault'); END;").unwrap();
    let before = old_data(&c);
    let old_schema = schema(&c);
    assert!(matches!(
        execute_versioned_migration(c, 8, PRODUCTS_MIGRATION_SQL),
        Err(MigrationError::ApplyFailed)
    ));
    let c = open_database(&p).unwrap();
    health(&c, 7);
    assert_eq!(old_data(&c), before);
    assert_eq!(schema(&c), old_schema);
    assert_eq!(
        rows(
            &c,
            "SELECT count(*) FROM main.sqlite_schema WHERE name='products'"
        ),
        vec![vec![Value::Integer(0)]]
    );
    c.execute_batch("DROP TRIGGER main.prd004_ledger_fail;")
        .unwrap();
    let c = migrate(c);
    health(&c, 8);
    assert_eq!(old_data(&c), before);
    drop(c);
    health(&open_database(&p).unwrap(), 8);
}

#[test]
fn prd004_name_collision_and_duplicate_version_preserve_schema_data_and_ledger() {
    let d = TempDirectory::new();
    let p = d.path().join("prd004-collision.sqlite");
    let c = v7(&p);
    seed(&c);
    c.execute_batch("CREATE TABLE main.products(sentinel TEXT); INSERT INTO main.products VALUES('synthetic sentinel');").unwrap();
    let before = old_data(&c);
    let old_schema = schema(&c);
    assert!(matches!(
        execute_versioned_migration(c, 8, PRODUCTS_MIGRATION_SQL),
        Err(MigrationError::ApplyFailed)
    ));
    let c = open_database(&p).unwrap();
    health(&c, 7);
    assert_eq!(schema(&c), old_schema);
    assert_eq!(old_data(&c), before);
    assert_eq!(
        rows(&c, "SELECT * FROM main.products"),
        vec![vec![Value::Text("synthetic sentinel".into())]]
    );
    c.execute_batch("DROP TABLE main.products;").unwrap();
    let c = migrate(c);
    let complete = schema(&c);
    assert!(matches!(
        execute_versioned_migration(c, 8, PRODUCTS_MIGRATION_SQL),
        Err(MigrationError::InvalidVersion)
    ));
    let c = open_database(&p).unwrap();
    health(&c, 8);
    assert_eq!(schema(&c), complete);
    assert_eq!(old_data(&c), before);
}
