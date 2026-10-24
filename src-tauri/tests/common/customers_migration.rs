use super::TempDirectory;
use millani_artes::database::{
    execute_versioned_migration, open_database, schema_version, MigrationError,
    ACCOUNTS_MIGRATION_SQL, AUDIT_EVENT_MIGRATION_SQL, CATEGORIES_MIGRATION_SQL,
    CUSTOMERS_MIGRATION_SQL, CUSTOMERS_MIGRATION_VERSION, MOVEMENTS_MIGRATION_SQL,
    MOVEMENT_CATEGORIES_MIGRATION_SQL, PRODUCTS_MIGRATION_SQL, PRODUCTS_MIGRATION_VERSION,
    RECURRENCES_MIGRATION_SQL,
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
fn v8(path: &Path) -> Connection {
    let mut c = execute_versioned_migration(open_database(path).unwrap(), 1, "").unwrap();
    for (v, sql) in [
        (2, AUDIT_EVENT_MIGRATION_SQL),
        (3, ACCOUNTS_MIGRATION_SQL),
        (4, CATEGORIES_MIGRATION_SQL),
        (5, MOVEMENTS_MIGRATION_SQL),
        (6, MOVEMENT_CATEGORIES_MIGRATION_SQL),
        (7, RECURRENCES_MIGRATION_SQL),
        (PRODUCTS_MIGRATION_VERSION, PRODUCTS_MIGRATION_SQL),
    ] {
        c = execute_versioned_migration(c, v, sql).unwrap();
    }
    c
}
fn migrate(c: Connection) -> Connection {
    assert_eq!(CUSTOMERS_MIGRATION_VERSION, 9);
    execute_versioned_migration(c, CUSTOMERS_MIGRATION_VERSION, CUSTOMERS_MIGRATION_SQL).unwrap()
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
}

const CUS004_VALID_DDDS: [u8; 67] = [
    11, 12, 13, 14, 15, 16, 17, 18, 19, 21, 22, 24, 27, 28, 31, 32, 33, 34, 35, 37, 38, 41, 42, 43,
    44, 45, 46, 47, 48, 49, 51, 53, 54, 55, 61, 62, 63, 64, 65, 66, 67, 68, 69, 71, 73, 74, 75, 77,
    79, 81, 82, 83, 84, 85, 86, 87, 88, 89, 91, 92, 93, 94, 95, 96, 97, 98, 99,
];

#[test]
fn cus004_empty_database_has_exact_strict_customer_shape_and_roundtrips_optional_phones() {
    let d = TempDirectory::new();
    let p = d.path().join("cus004-empty.sqlite");
    let c = migrate(v8(&p));
    health(&c, 9);
    let columns = rows(&c, "PRAGMA main.table_xinfo('customers')");
    assert_eq!(columns.len(), 3);
    for (i, (name, kind)) in [("id", "BLOB"), ("name", "TEXT"), ("phone", "TEXT")]
        .iter()
        .enumerate()
    {
        assert_eq!(
            columns[i],
            vec![
                Value::Integer(i as i64),
                Value::Text((*name).into()),
                Value::Text((*kind).into()),
                Value::Integer(i64::from(i < 2)),
                Value::Null,
                Value::Integer(i64::from(i == 0)),
                Value::Integer(0)
            ]
        );
    }
    assert_eq!(
        rows(
            &c,
            "SELECT strict FROM pragma_table_list WHERE schema='main' AND name='customers'"
        ),
        vec![vec![Value::Integer(1)]]
    );
    assert_eq!(rows(&c, "PRAGMA main.index_list('customers')").len(), 1);
    assert!(rows(&c, "PRAGMA main.foreign_key_list('customers')").is_empty());
    c.execute(
        "INSERT INTO main.customers(id,name) VALUES(?1,?2)",
        params![[0_u8; 16].as_slice(), "\0Cliente ‘Á’\r\n  "],
    )
    .unwrap();
    for ddd in CUS004_VALID_DDDS {
        let phone = format!("({ddd:02}) 91234-5678");
        c.execute(
            "INSERT INTO main.customers VALUES(?1,?2,?3)",
            params![[ddd; 16].as_slice(), "Nome duplicável", phone],
        )
        .unwrap();
    }
    let saved = rows(&c, "SELECT * FROM main.customers ORDER BY id");
    assert_eq!(saved.len(), 68);
    assert_eq!(
        saved[0],
        vec![
            Value::Blob(vec![0; 16]),
            Value::Text("\0Cliente ‘Á’\r\n  ".into()),
            Value::Null
        ]
    );
    for (i, ddd) in CUS004_VALID_DDDS.iter().enumerate() {
        assert_eq!(
            saved[i + 1],
            vec![
                Value::Blob(vec![*ddd; 16]),
                Value::Text("Nome duplicável".into()),
                Value::Text(format!("({ddd:02}) 91234-5678"))
            ]
        );
    }
    c.execute(
        "UPDATE main.customers SET phone=?1 WHERE id=?2",
        params!["(11) 99999-9999", [0_u8; 16].as_slice()],
    )
    .unwrap();
    assert_eq!(
        rows(&c, "SELECT phone FROM main.customers WHERE id=zeroblob(16)"),
        vec![vec![Value::Text("(11) 99999-9999".into())]]
    );
    c.execute(
        "UPDATE main.customers SET phone=NULL WHERE id=?1",
        params![[0_u8; 16].as_slice()],
    )
    .unwrap();
    assert_eq!(rows(&c, "SELECT * FROM main.customers ORDER BY id"), saved);
    drop(c);
    let c = open_database(&p).unwrap();
    health(&c, 9);
    assert_eq!(rows(&c, "SELECT * FROM main.customers ORDER BY id"), saved);
}

#[test]
fn cus004_customer_constraints_reject_invalid_inserts_and_updates_without_mutation() {
    let d = TempDirectory::new();
    let c = migrate(v8(&d.path().join("cus004-constraints.sqlite")));
    c.execute(
        "INSERT INTO main.customers VALUES(?1,'Cliente alvo',NULL)",
        params![[1_u8; 16].as_slice()],
    )
    .unwrap();
    let before = rows(&c, "SELECT * FROM main.customers ORDER BY id");
    let valid = vec![
        Value::Blob(vec![2_u8; 16]),
        Value::Text("Cliente sintético".into()),
        Value::Text("(11) 91234-5678".into()),
    ];
    let mut bad_values = vec![
        (0, Value::Null),
        (0, Value::Blob(vec![2; 0])),
        (0, Value::Blob(vec![2; 15])),
        (0, Value::Blob(vec![2; 17])),
        (0, Value::Text("1234567890123456".into())),
        (0, Value::Integer(2)),
        (1, Value::Null),
        (1, Value::Text("".into())),
        (1, Value::Blob(vec![1])),
        (2, Value::Blob(vec![1])),
        (2, Value::Integer(11912345678)),
        (2, Value::Real(1.5)),
    ];
    for phone in [
        "",
        " ",
        "\t\r\n",
        "11912345678",
        "+55 (11) 91234-5678",
        "(11)91234-5678",
        "(11) 912345678",
        "11) 91234-5678",
        "(11) 91234-5678 ",
        " (11) 91234-5678",
        "(11) 81234-5678",
        "(11) 01234-5678",
        "(11) 9123-4567",
        "(11) 912345-6789",
        "(1) 91234-5678",
        "(111) 91234-5678",
        "(11) 91234.5678",
        "(11) 91234-5678\0",
        "(11) 91234-5678\r\n",
    ] {
        bad_values.push((2, Value::Text(phone.into())));
    }
    for ddd in 0..=99_u8 {
        if !CUS004_VALID_DDDS.contains(&ddd) {
            bad_values.push((2, Value::Text(format!("({ddd:02}) 91234-5678"))));
        }
    }
    let baseline = "(11) 91234-5678";
    for index in 0..baseline.len() {
        for replacement in ["é", "😀", "١", "１", "\0", "A", "–", "\u{00a0}"] {
            let mut phone = baseline.to_owned();
            phone.replace_range(index..index + 1, replacement);
            bad_values.push((2, Value::Text(phone)));
        }
    }
    for replacement in ["é", "١", "１", "😀"] {
        for index in 0..=baseline.len() - replacement.len() {
            let mut phone = baseline.to_owned();
            phone.replace_range(index..index + replacement.len(), replacement);
            assert_eq!(phone.len(), 15);
            bad_values.push((2, Value::Text(phone)));
        }
    }
    for (index, bad) in bad_values {
        let mut row = valid.clone();
        row[index] = bad;
        for update in [false, true] {
            let result = if update {
                c.execute(
                    "UPDATE main.customers SET id=?1,name=?2,phone=?3 WHERE id=?4",
                    rusqlite::params_from_iter(
                        row.iter().cloned().chain([Value::Blob(vec![1_u8; 16])]),
                    ),
                )
            } else {
                c.execute(
                    "INSERT INTO main.customers VALUES(?1,?2,?3)",
                    rusqlite::params_from_iter(row.iter()),
                )
            };
            assert!(
                matches!(result,Err(Error::SqliteFailure(ref e,_))if e.code==ErrorCode::ConstraintViolation),
                "CUSTOMER_INVALID_ACCEPTED:{index}"
            );
            assert_eq!(rows(&c, "SELECT * FROM main.customers ORDER BY id"), before);
        }
    }
    assert!(
        matches!(c.execute("INSERT INTO main.customers VALUES(?1,'duplicate',NULL)",params![[1_u8;16].as_slice()]),Err(Error::SqliteFailure(ref e,_))if e.code==ErrorCode::ConstraintViolation)
    );
    assert_eq!(rows(&c, "SELECT * FROM main.customers ORDER BY id"), before);
    health(&c, 9);
}

#[test]
fn cus004_upgrade_preserves_all_existing_history_and_reopens() {
    let d = TempDirectory::new();
    let p = d.path().join("cus004-upgrade.sqlite");
    let c = v8(&p);
    seed(&c);
    let before = old_data(&c);
    let old_schema = schema(&c);
    let c = migrate(c);
    assert_eq!(old_data(&c), before);
    health(&c, 9);
    assert!(old_schema.iter().all(|r| schema(&c).contains(r)));
    assert!(rows(&c, "SELECT * FROM main.customers").is_empty());
    drop(c);
    let c = open_database(&p).unwrap();
    health(&c, 9);
    assert_eq!(old_data(&c), before);
    assert!(old_schema.iter().all(|r| schema(&c).contains(r)));
}

#[test]
fn cus004_ledger_failure_rolls_back_customer_ddl_and_allows_retry() {
    let d = TempDirectory::new();
    let p = d.path().join("cus004-rollback.sqlite");
    let c = v8(&p);
    seed(&c);
    c.execute_batch("CREATE TRIGGER main.cus004_ledger_fail BEFORE INSERT ON schema_migration WHEN NEW.version=9 BEGIN SELECT RAISE(ABORT,'synthetic migration fault'); END;").unwrap();
    let before = old_data(&c);
    let old_schema = schema(&c);
    assert!(matches!(
        execute_versioned_migration(c, 9, CUSTOMERS_MIGRATION_SQL),
        Err(MigrationError::ApplyFailed)
    ));
    let c = open_database(&p).unwrap();
    health(&c, 8);
    assert_eq!(old_data(&c), before);
    assert_eq!(schema(&c), old_schema);
    assert_eq!(
        rows(
            &c,
            "SELECT count(*) FROM main.sqlite_schema WHERE name='customers'"
        ),
        vec![vec![Value::Integer(0)]]
    );
    c.execute_batch("DROP TRIGGER main.cus004_ledger_fail;")
        .unwrap();
    let c = migrate(c);
    health(&c, 9);
    assert_eq!(old_data(&c), before);
    drop(c);
    health(&open_database(&p).unwrap(), 9);
}

#[test]
fn cus004_name_collision_and_duplicate_version_preserve_schema_data_and_ledger() {
    let d = TempDirectory::new();
    let p = d.path().join("cus004-collision.sqlite");
    let c = v8(&p);
    seed(&c);
    c.execute_batch("CREATE TABLE main.customers(sentinel TEXT); INSERT INTO main.customers VALUES('synthetic sentinel');").unwrap();
    let before = old_data(&c);
    let old_schema = schema(&c);
    assert!(matches!(
        execute_versioned_migration(c, 9, CUSTOMERS_MIGRATION_SQL),
        Err(MigrationError::ApplyFailed)
    ));
    let c = open_database(&p).unwrap();
    health(&c, 8);
    assert_eq!(schema(&c), old_schema);
    assert_eq!(old_data(&c), before);
    assert_eq!(
        rows(&c, "SELECT * FROM main.customers"),
        vec![vec![Value::Text("synthetic sentinel".into())]]
    );
    c.execute_batch("DROP TABLE main.customers;").unwrap();
    let c = migrate(c);
    let complete = schema(&c);
    assert!(matches!(
        execute_versioned_migration(c, 9, CUSTOMERS_MIGRATION_SQL),
        Err(MigrationError::InvalidVersion)
    ));
    let c = open_database(&p).unwrap();
    health(&c, 9);
    assert_eq!(schema(&c), complete);
    assert_eq!(old_data(&c), before);
}
