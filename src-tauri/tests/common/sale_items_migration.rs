use super::TempDirectory;
use millani_artes::database::{
    execute_versioned_migration, open_database, schema_version, MigrationError,
    ACCOUNTS_MIGRATION_SQL, AUDIT_EVENT_MIGRATION_SQL, CATEGORIES_MIGRATION_SQL,
    CUSTOMERS_MIGRATION_SQL, CUSTOMERS_MIGRATION_VERSION, MOVEMENTS_MIGRATION_SQL,
    MOVEMENT_CATEGORIES_MIGRATION_SQL, PRODUCTS_MIGRATION_SQL, PRODUCTS_MIGRATION_VERSION,
    RECURRENCES_MIGRATION_SQL, SALES_MIGRATION_SQL, SALES_MIGRATION_VERSION,
    SALE_ITEMS_MIGRATION_SQL, SALE_ITEMS_MIGRATION_VERSION,
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
fn v10(path: &Path) -> Connection {
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
        (SALES_MIGRATION_VERSION, SALES_MIGRATION_SQL),
    ] {
        c = execute_versioned_migration(c, v, sql).unwrap();
    }
    c
}
fn migrate(c: Connection) -> Connection {
    assert_eq!(SALE_ITEMS_MIGRATION_VERSION, 11);
    execute_versioned_migration(c, SALE_ITEMS_MIGRATION_VERSION, SALE_ITEMS_MIGRATION_SQL).unwrap()
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
        "sales",
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
    c.execute(
        "INSERT INTO main.sales VALUES(?1,'open')",
        params![[9_u8; 16].as_slice()],
    )
    .unwrap();
}

fn parents(c: &Connection) {
    for (id, status) in [(0x41_u8, "open"), (0x42, "finalized")] {
        c.execute(
            "INSERT INTO main.sales VALUES(?1,?2)",
            params![[id; 16].as_slice(), status],
        )
        .unwrap();
    }
    c.execute(
        "INSERT INTO main.products VALUES(?1,'Template','A',1500,1)",
        params![[0x21_u8; 16].as_slice()],
    )
    .unwrap();
}
fn item_row(
    id: u8,
    sale: u8,
    position: i64,
    product: Option<u8>,
    description: &str,
    amounts: [i64; 4],
) -> Vec<Value> {
    let mut row = vec![
        Value::Blob(vec![id; 16]),
        Value::Blob(vec![sale; 16]),
        Value::Integer(position),
        product.map_or(Value::Null, |p| Value::Blob(vec![p; 16])),
        Value::Text(description.into()),
    ];
    row.extend(amounts.map(Value::Integer));
    row
}
fn insert(c: &Connection, row: &[Value]) -> rusqlite::Result<usize> {
    c.execute(
        "INSERT INTO main.sale_items VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        rusqlite::params_from_iter(row.iter()),
    )
}
fn items(c: &Connection) -> Vec<Vec<Value>> {
    rows(c, "SELECT * FROM main.sale_items ORDER BY sale_id,position")
}

#[test]
fn sal011_empty_database_has_exact_shape_relations_order_and_snapshot_roundtrip() {
    let d = TempDirectory::new();
    let p = d.path().join("sal011-empty.sqlite");
    let c = migrate(v10(&p));
    health(&c, 11);
    let columns = rows(&c, "PRAGMA main.table_xinfo('sale_items')");
    assert_eq!(columns.len(), 9);
    for (i, (name, kind)) in [
        ("id", "BLOB"),
        ("sale_id", "BLOB"),
        ("position", "INTEGER"),
        ("product_id", "BLOB"),
        ("description", "TEXT"),
        ("quantity", "INTEGER"),
        ("unit_price_cents", "INTEGER"),
        ("discount_cents", "INTEGER"),
        ("total_cents", "INTEGER"),
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
                Value::Integer(i64::from(i != 3)),
                Value::Null,
                Value::Integer(i64::from(i == 0)),
                Value::Integer(0)
            ]
        );
    }
    assert_eq!(
        rows(
            &c,
            "SELECT strict FROM pragma_table_list WHERE schema='main' AND name='sale_items'"
        ),
        vec![vec![Value::Integer(1)]]
    );
    let keys = rows(&c, "PRAGMA main.foreign_key_list('sale_items')");
    assert_eq!(keys.len(), 2);
    for (column, table) in [("sale_id", "sales"), ("product_id", "products")] {
        let key = keys
            .iter()
            .find(|k| k[3] == Value::Text(column.into()))
            .unwrap();
        assert_eq!(
            &key[2..],
            &[
                Value::Text(table.into()),
                Value::Text(column.into()),
                Value::Text("id".into()),
                Value::Text("NO ACTION".into()),
                Value::Text("NO ACTION".into()),
                Value::Text("NONE".into())
            ]
        );
    }
    let indexes = rows(&c, "PRAGMA main.index_list('sale_items')");
    assert_eq!(indexes.len(), 2);
    let mut index_columns = Vec::new();
    for index in indexes {
        assert_eq!(index[2], Value::Integer(1));
        let Value::Text(name) = &index[1] else {
            panic!("ITEM_INDEX_NAME")
        };
        let info = rows(
            &c,
            &format!("PRAGMA main.index_info('{}')", name.replace('\'', "''")),
        );
        index_columns.push(info.into_iter().map(|v| v[2].clone()).collect::<Vec<_>>());
    }
    assert!(index_columns.contains(&vec![Value::Text("id".into())]));
    assert!(index_columns.contains(&vec![
        Value::Text("sale_id".into()),
        Value::Text("position".into())
    ]));
    assert!(items(&c).is_empty());
    parents(&c);
    let fixture = [
        item_row(0, 0x41, 2, None, "  Frete ‘Á’\r\n\0  ", [1, 1000, 0, 1000]),
        item_row(
            u8::MAX,
            0x41,
            0,
            Some(0x21),
            "Snapshot",
            [2, i64::MAX, i64::MAX, i64::MAX],
        ),
        item_row(3, 0x42, 0, None, "Zero", [i64::MAX, 0, 0, 0]),
        item_row(4, 0x42, i64::MAX, Some(0x21), "", [1, 0, 0, 0]),
    ];
    for row in &fixture {
        assert_eq!(insert(&c, row).unwrap(), 1);
    }
    let expected = vec![
        fixture[1].clone(),
        fixture[0].clone(),
        fixture[2].clone(),
        fixture[3].clone(),
    ];
    assert_eq!(items(&c), expected);
    drop(c);
    let c = open_database(&p).unwrap();
    health(&c, 11);
    assert_eq!(items(&c), expected);
}

#[test]
fn sal011_constraints_reject_invalid_insert_update_identity_position_links_and_totals() {
    let d = TempDirectory::new();
    let c = migrate(v10(&d.path().join("sal011-constraints.sqlite")));
    parents(&c);
    insert(&c, &item_row(1, 0x41, 0, None, "Alvo", [1, 1, 0, 1])).unwrap();
    let before = items(&c);
    let valid = item_row(2, 0x41, 1, Some(0x21), "Snapshot", [2, 1500, 300, 2700]);
    let mut bad_values = Vec::new();
    for column in [0, 1, 3] {
        for bad in [
            Value::Blob(vec![]),
            Value::Blob(vec![7; 15]),
            Value::Blob(vec![7; 17]),
            Value::Text("1234567890123456".into()),
            Value::Integer(1),
        ] {
            bad_values.push((column, bad));
        }
    }
    for column in [0, 1, 2, 4, 5, 6, 7, 8] {
        bad_values.push((column, Value::Null));
    }
    bad_values.extend([
        (1, Value::Blob(vec![0x77; 16])),
        (3, Value::Blob(vec![0x77; 16])),
        (2, Value::Integer(-1)),
        (2, Value::Real(1.5)),
        (2, Value::Text("invalid".into())),
        (4, Value::Blob(vec![1])),
    ]);
    for column in [5, 6, 7, 8] {
        for bad in [
            Value::Integer(-1),
            Value::Integer(i64::MIN),
            Value::Real(1.5),
            Value::Text("invalid".into()),
        ] {
            bad_values.push((column, bad));
        }
    }
    bad_values.extend([
        (5, Value::Integer(0)),
        (5, Value::Integer(i64::MAX)),
        (6, Value::Integer(1501)),
        (7, Value::Integer(3001)),
        (7, Value::Integer(i64::MAX)),
        (8, Value::Integer(2701)),
        (8, Value::Integer(i64::MAX)),
    ]);
    for (index, bad) in bad_values {
        let mut row = valid.clone();
        row[index] = bad;
        for update in [false, true] {
            let result = if update {
                c.execute("UPDATE main.sale_items SET id=?1,sale_id=?2,position=?3,product_id=?4,description=?5,quantity=?6,unit_price_cents=?7,discount_cents=?8,total_cents=?9 WHERE id=?10",rusqlite::params_from_iter(row.iter().cloned().chain([Value::Blob(vec![1;16])])))
            } else {
                insert(&c, &row)
            };
            assert!(
                matches!(result,Err(Error::SqliteFailure(ref e,_))if e.code==ErrorCode::ConstraintViolation),
                "SALES_ITEM_INVALID_ACCEPTED:{index}"
            );
            assert_eq!(items(&c), before);
            assert!(c.is_autocommit());
        }
    }
    for row in [
        item_row(1, 0x41, 1, None, "duplicate id", [1, 1, 0, 1]),
        item_row(3, 0x41, 0, None, "duplicate position", [1, 1, 0, 1]),
    ] {
        assert!(
            matches!(insert(&c,&row),Err(Error::SqliteFailure(ref e,_))if e.code==ErrorCode::ConstraintViolation)
        );
        assert_eq!(items(&c), before);
    }
    insert(&c, &valid).unwrap();
    assert_eq!(items(&c).len(), 2);
    health(&c, 11);
}

#[test]
fn sal011_exact_monetary_constraint_matches_independent_i128_oracle_at_boundaries() {
    let d = TempDirectory::new();
    let c = migrate(v10(&d.path().join("sal011-money.sqlite")));
    parents(&c);
    let edges = [0, 1, 2, 3, i64::MAX / 2, i64::MAX - 1, i64::MAX];
    let mut cases = Vec::new();
    for q in edges {
        for p in edges {
            for discount in edges {
                for total in edges {
                    cases.push([q, p, discount, total]);
                }
            }
        }
    }
    cases.extend([
        [2, i64::MAX, i64::MAX, i64::MAX],
        [i64::MAX, 2, i64::MAX, i64::MAX],
        [i64::MAX, 1, i64::MAX - 1, 1],
        [i64::MAX, 1, 1, i64::MAX - 1],
        [
            2,
            4611686018427387904,
            4611686018427387904,
            4611686018427387904,
        ],
        [3, 6148914691236517205, i64::MAX, i64::MAX],
        [2, i64::MAX, i64::MAX, i64::MAX - 1],
        [2, i64::MAX, i64::MAX - 1, i64::MAX],
        [1, i64::MAX, i64::MAX, i64::MAX],
        [1, 0, i64::MAX, i64::MAX],
        [i64::MAX, i64::MAX, i64::MAX, i64::MAX],
        [i64::MAX, 1, i64::MAX - 2, 2],
    ]);
    assert_eq!(cases.len(), 2413);
    let tx = c.unchecked_transaction().unwrap();
    for amounts in cases {
        let [q, p, discount, total] = amounts;
        let expected = q > 0
            && p >= 0
            && discount >= 0
            && total >= 0
            && i128::from(q) * i128::from(p) - i128::from(discount) == i128::from(total);
        let row = item_row(0x61, 0x41, 0, None, "Oracle", amounts);
        let result = insert(&tx, &row);
        if expected {
            assert_eq!(result.unwrap(), 1);
            assert_eq!(items(&tx), vec![row]);
            assert_eq!(rows(&tx,"SELECT typeof(quantity),typeof(unit_price_cents),typeof(discount_cents),typeof(total_cents) FROM main.sale_items"),vec![vec![Value::Text("integer".into());4]]);
            assert_eq!(tx.execute("DELETE FROM main.sale_items", []).unwrap(), 1);
        } else {
            assert!(
                matches!(result,Err(Error::SqliteFailure(ref e,_))if e.code==ErrorCode::ConstraintViolation),
                "ITEM_ARITH_ORACLE_MISMATCH:{amounts:?}"
            );
        }
        assert!(items(&tx).is_empty());
    }
    tx.commit().unwrap();
    health(&c, 11);
}

#[test]
fn sal011_parent_foreign_keys_restrict_mutation_and_keep_copied_values_after_reopen() {
    let d = TempDirectory::new();
    let p = d.path().join("sal011-parents.sqlite");
    let c = migrate(v10(&p));
    parents(&c);
    insert(
        &c,
        &item_row(1, 0x41, 0, Some(0x21), "A", [2, 1500, 0, 3000]),
    )
    .unwrap();
    let before = items(&c);
    let old = old_data(&c);
    for sql in [
        "DELETE FROM main.sales WHERE id=?1",
        "UPDATE main.sales SET id=zeroblob(16) WHERE id=?1",
        "DELETE FROM main.products WHERE id=?1",
        "UPDATE main.products SET id=zeroblob(16) WHERE id=?1",
    ] {
        let id = if sql.contains("main.sales") {
            0x41_u8
        } else {
            0x21
        };
        assert!(
            matches!(c.execute(sql,params![[id;16].as_slice()]),Err(Error::SqliteFailure(ref e,_))if e.code==ErrorCode::ConstraintViolation),
            "SALES_ITEM_PARENT_MUTATION_ACCEPTED"
        );
        assert_eq!(old_data(&c), old);
        assert_eq!(items(&c), before);
    }
    assert_eq!(c.execute("UPDATE main.products SET description='B',suggested_price_cents=1700,is_active=0 WHERE id=?1",params![[0x21_u8;16].as_slice()]).unwrap(),1);
    assert_eq!(items(&c), before);
    drop(c);
    let c = open_database(&p).unwrap();
    health(&c, 11);
    assert_eq!(items(&c), before);
    assert_eq!(rows(&c,"SELECT description,suggested_price_cents,is_active FROM main.products WHERE id=x'21212121212121212121212121212121'"),vec![vec![Value::Text("B".into()),Value::Integer(1700),Value::Integer(0)]]);
}

#[test]
fn sal011_upgrade_preserves_all_existing_history_and_reopens() {
    let d = TempDirectory::new();
    let p = d.path().join("sal011-upgrade.sqlite");
    let c = v10(&p);
    seed(&c);
    let before = old_data(&c);
    let old_schema = schema(&c);
    let c = migrate(c);
    assert_eq!(old_data(&c), before);
    health(&c, 11);
    assert!(old_schema.iter().all(|r| schema(&c).contains(r)));
    assert!(items(&c).is_empty());
    drop(c);
    let c = open_database(&p).unwrap();
    health(&c, 11);
    assert_eq!(old_data(&c), before);
    assert!(old_schema.iter().all(|r| schema(&c).contains(r)));
}

#[test]
fn sal011_ledger_failure_rolls_back_item_ddl_and_allows_retry() {
    let d = TempDirectory::new();
    let p = d.path().join("sal011-rollback.sqlite");
    let c = v10(&p);
    seed(&c);
    c.execute_batch("CREATE TRIGGER main.sal011_ledger_fail BEFORE INSERT ON schema_migration WHEN NEW.version=11 BEGIN SELECT RAISE(ABORT,'synthetic migration fault'); END;").unwrap();
    let before = old_data(&c);
    let old_schema = schema(&c);
    assert!(matches!(
        execute_versioned_migration(c, 11, SALE_ITEMS_MIGRATION_SQL),
        Err(MigrationError::ApplyFailed)
    ));
    let c = open_database(&p).unwrap();
    health(&c, 10);
    assert_eq!(old_data(&c), before);
    assert_eq!(schema(&c), old_schema);
    assert_eq!(
        rows(
            &c,
            "SELECT count(*) FROM main.sqlite_schema WHERE name='sale_items'"
        ),
        vec![vec![Value::Integer(0)]]
    );
    c.execute_batch("DROP TRIGGER main.sal011_ledger_fail;")
        .unwrap();
    let c = migrate(c);
    health(&c, 11);
    assert_eq!(old_data(&c), before);
    drop(c);
    health(&open_database(&p).unwrap(), 11);
}

#[test]
fn sal011_name_collision_and_duplicate_version_preserve_schema_data_and_ledger() {
    let d = TempDirectory::new();
    let p = d.path().join("sal011-collision.sqlite");
    let c = v10(&p);
    seed(&c);
    c.execute_batch("CREATE TABLE main.sale_items(sentinel TEXT); INSERT INTO main.sale_items VALUES('synthetic sentinel');").unwrap();
    let before = old_data(&c);
    let old_schema = schema(&c);
    assert!(matches!(
        execute_versioned_migration(c, 11, SALE_ITEMS_MIGRATION_SQL),
        Err(MigrationError::ApplyFailed)
    ));
    let c = open_database(&p).unwrap();
    health(&c, 10);
    assert_eq!(schema(&c), old_schema);
    assert_eq!(old_data(&c), before);
    assert_eq!(
        rows(&c, "SELECT * FROM main.sale_items"),
        vec![vec![Value::Text("synthetic sentinel".into())]]
    );
    c.execute_batch("DROP TABLE main.sale_items;").unwrap();
    let c = migrate(c);
    let complete = schema(&c);
    assert!(matches!(
        execute_versioned_migration(c, 11, SALE_ITEMS_MIGRATION_SQL),
        Err(MigrationError::InvalidVersion)
    ));
    let c = open_database(&p).unwrap();
    health(&c, 11);
    assert_eq!(schema(&c), complete);
    assert_eq!(old_data(&c), before);
}
