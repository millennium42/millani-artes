use super::TempDirectory;
use millani_artes::database::{
    execute_versioned_migration, open_database, schema_version, MigrationError,
    ACCOUNTS_MIGRATION_SQL, AUDIT_EVENT_MIGRATION_SQL, CATEGORIES_MIGRATION_SQL,
    MOVEMENTS_MIGRATION_SQL, MOVEMENT_CATEGORIES_MIGRATION_SQL, RECURRENCES_MIGRATION_SQL,
    RECURRENCES_MIGRATION_VERSION,
};
use rusqlite::{params, types::Value, Connection};
use std::path::Path;

fn v6(path: &Path) -> Connection {
    let mut c = execute_versioned_migration(open_database(path).unwrap(), 1, "").unwrap();
    for (v, s) in [
        (2, AUDIT_EVENT_MIGRATION_SQL),
        (3, ACCOUNTS_MIGRATION_SQL),
        (4, CATEGORIES_MIGRATION_SQL),
        (5, MOVEMENTS_MIGRATION_SQL),
        (6, MOVEMENT_CATEGORIES_MIGRATION_SQL),
    ] {
        c = execute_versioned_migration(c, v, s).unwrap();
    }
    c
}
fn migrate(c: Connection) -> Connection {
    assert_eq!(RECURRENCES_MIGRATION_VERSION, 7);
    execute_versioned_migration(c, RECURRENCES_MIGRATION_VERSION, RECURRENCES_MIGRATION_SQL)
        .unwrap()
}
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
            .map(|v| vec![Value::Integer(v.into())])
            .collect::<Vec<_>>()
    );
}
fn recurrence(c: &Connection, id: u8, kind: &str, amount: Option<i64>, context: &str, day: i64) {
    c.execute("INSERT INTO main.recurrences(id,name,kind,amount_cents,context,due_day) VALUES(?1,?2,?3,?4,?5,?6)",params![[id;16].as_slice(),"\0Recorrência – café",kind,amount,context,day]).unwrap();
}
fn seed(c: &Connection) {
    recurrence(c, 1, "fixed", Some(7), "home", 31);
    c.execute("INSERT INTO main.accounts(id,name,account_type,initial_balance_cents) VALUES(?1,'A','cash',-1234)",params![[3_u8;16].as_slice()]).unwrap();
    for id in [4_u8, 5, 6] {
        c.execute("INSERT INTO main.movements(id,kind,amount_cents,context,account_id) VALUES(?1,'exit',7,'home',?2)",params![[id;16].as_slice(),[3_u8;16].as_slice()]).unwrap();
    }
}
fn obligation(
    c: &Connection,
    id: u8,
    month: i64,
    state: &str,
    amount: Option<i64>,
    movement: Option<Vec<u8>>,
) {
    c.execute("INSERT INTO main.recurrence_obligations(id,recurrence_id,period_year,period_month,due_at_ms,state,amount_cents,movement_id,context) VALUES(?1,?2,2026,?3,-1234,?4,?5,?6,'home')",params![[id;16].as_slice(),[1_u8;16].as_slice(),month,state,amount,movement]).unwrap();
}

#[test]
fn recurrence_migration_creates_empty_strict_schema_and_roundtrips_states() {
    let d = TempDirectory::new();
    let p = d.path().join("rec007-empty.sqlite");
    let c = migrate(v6(&p));
    health(&c, 7);
    for (table, names, types) in [
        (
            "recurrences",
            vec!["id", "name", "kind", "amount_cents", "context", "due_day"],
            vec!["BLOB", "TEXT", "TEXT", "INTEGER", "TEXT", "INTEGER"],
        ),
        (
            "recurrence_obligations",
            vec![
                "id",
                "recurrence_id",
                "period_year",
                "period_month",
                "due_at_ms",
                "state",
                "amount_cents",
                "movement_id",
                "context",
            ],
            vec![
                "BLOB", "BLOB", "INTEGER", "INTEGER", "INTEGER", "TEXT", "INTEGER", "BLOB", "TEXT",
            ],
        ),
    ] {
        let columns = rows(&c, &format!("PRAGMA main.table_xinfo('{table}')"));
        assert_eq!(
            columns.iter().map(|r| r[1].clone()).collect::<Vec<_>>(),
            names
                .into_iter()
                .map(|n| Value::Text(n.into()))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            columns.iter().map(|r| r[2].clone()).collect::<Vec<_>>(),
            types
                .into_iter()
                .map(|n| Value::Text(n.into()))
                .collect::<Vec<_>>()
        );
        assert!(columns
            .iter()
            .all(|r| r[4] == Value::Null && r[6] == Value::Integer(0)));
        assert_eq!(columns[0][5], Value::Integer(1));
        assert!(rows(&c, &format!("SELECT * FROM main.{table}")).is_empty());
        assert_eq!(
            rows(
                &c,
                &format!(
                    "SELECT strict FROM pragma_table_list WHERE schema='main' AND name='{table}'"
                )
            ),
            vec![vec![Value::Integer(1)]]
        );
    }
    assert_eq!(rows(&c, "PRAGMA main.index_list('recurrences')").len(), 1);
    assert_eq!(
        rows(&c, "PRAGMA main.index_list('recurrence_obligations')").len(),
        3
    );
    let fks = rows(&c, "PRAGMA main.foreign_key_list('recurrence_obligations')");
    assert_eq!(fks.len(), 2);
    assert!(fks.iter().all(
        |r| r[5] == Value::Text("NO ACTION".into()) && r[6] == Value::Text("NO ACTION".into())
    ));
    for (id, kind, amount, ctx, day) in [
        (1, "fixed", Some(1), "home", 1),
        (2, "fixed", Some(i64::MAX), "factory", 31),
        (3, "variable", None, "home", 15),
        (4, "variable", None, "factory", 28),
    ] {
        recurrence(&c, id, kind, amount, ctx, day);
    }
    let saved = rows(
        &c,
        "SELECT name,kind,amount_cents,context,due_day FROM main.recurrences ORDER BY id",
    );
    assert_eq!(saved.len(), 4);
    assert_eq!(
        saved[0],
        vec![
            Value::Text("\0Recorrência – café".into()),
            Value::Text("fixed".into()),
            Value::Integer(1),
            Value::Text("home".into()),
            Value::Integer(1)
        ]
    );
    assert_eq!(saved[1][2], Value::Integer(i64::MAX));
    assert_eq!(saved[2][2], Value::Null);
    assert_eq!(saved[3][2], Value::Null);
    c.execute(
        "INSERT INTO main.accounts VALUES(?1,'A','cash',0)",
        params![[3_u8; 16].as_slice()],
    )
    .unwrap();
    c.execute("INSERT INTO main.movements(id,kind,amount_cents,context,account_id) VALUES(?1,'exit',7,'home',?2)",params![[4_u8;16].as_slice(),[3_u8;16].as_slice()]).unwrap();
    obligation(&c, 10, 1, "awaiting_value", None, None);
    obligation(&c, 11, 2, "payable", Some(1), None);
    obligation(&c, 12, 3, "overdue", Some(i64::MAX), None);
    obligation(&c, 13, 4, "paid", Some(7), Some(vec![4; 16]));
    let before = rows(&c, "SELECT * FROM main.recurrence_obligations ORDER BY id");
    assert_eq!(before.len(), 4);
    assert_eq!(before[0][6], Value::Null);
    assert_eq!(before[3][7], Value::Blob(vec![4; 16]));
    drop(c);
    let c = open_database(&p).unwrap();
    health(&c, 7);
    assert_eq!(
        rows(&c, "SELECT * FROM main.recurrence_obligations ORDER BY id"),
        before
    );
}

#[test]
fn recurrence_schema_rejects_invalid_configurations_without_changing_rows() {
    let d = TempDirectory::new();
    let c = migrate(v6(&d.path().join("rec007-config.sqlite")));
    recurrence(&c, 1, "fixed", Some(1), "home", 31);
    let valid = vec![
        Value::Blob(vec![2; 16]),
        Value::Text("R".into()),
        Value::Text("fixed".into()),
        Value::Integer(7),
        Value::Text("factory".into()),
        Value::Integer(1),
    ];
    let before = rows(&c, "SELECT * FROM main.recurrences ORDER BY id");
    for (index, bad) in [
        (0, Value::Null),
        (0, Value::Blob(vec![2; 15])),
        (0, Value::Blob(vec![2; 17])),
        (0, Value::Text("1234567890123456".into())),
        (0, Value::Blob(vec![1; 16])),
        (1, Value::Null),
        (1, Value::Text("".into())),
        (2, Value::Text("other".into())),
        (2, Value::Null),
        (3, Value::Null),
        (3, Value::Integer(0)),
        (3, Value::Integer(-1)),
        (3, Value::Integer(i64::MIN)),
        (3, Value::Real(1.5)),
        (4, Value::Text("other".into())),
        (4, Value::Null),
        (5, Value::Integer(0)),
        (5, Value::Integer(32)),
        (5, Value::Null),
        (5, Value::Real(1.5)),
    ] {
        let mut row = valid.clone();
        row[index] = bad;
        assert!(
            c.execute(
                "INSERT INTO main.recurrences VALUES(?1,?2,?3,?4,?5,?6)",
                rusqlite::params_from_iter(row)
            )
            .is_err(),
            "RECURRENCE_INVALID_ACCEPTED:{index}"
        );
        assert_eq!(
            rows(&c, "SELECT * FROM main.recurrences ORDER BY id"),
            before
        );
    }
    for amount in [0, 1, i64::MAX] {
        assert!(
            c.execute(
                "INSERT INTO main.recurrences VALUES(?1,'R','variable',?2,'home',1)",
                params![[2_u8; 16].as_slice(), amount]
            )
            .is_err(),
            "VARIABLE_PRESET_ACCEPTED"
        );
    }
    health(&c, 7);
}

#[test]
fn obligation_schema_enforces_state_shapes_foreign_keys_and_unique_links() {
    let d = TempDirectory::new();
    let c = migrate(v6(&d.path().join("rec007-obligations.sqlite")));
    seed(&c);
    let valid = vec![
        Value::Blob(vec![10; 16]),
        Value::Blob(vec![1; 16]),
        Value::Integer(2026),
        Value::Integer(1),
        Value::Integer(1234),
        Value::Text("payable".into()),
        Value::Integer(7),
        Value::Null,
    ];
    for (index, bad) in [
        (0, Value::Null),
        (0, Value::Blob(vec![10; 15])),
        (0, Value::Text("1234567890123456".into())),
        (1, Value::Null),
        (1, Value::Blob(vec![99; 16])),
        (2, Value::Integer(0)),
        (2, Value::Integer(10000)),
        (2, Value::Null),
        (3, Value::Integer(0)),
        (3, Value::Integer(13)),
        (3, Value::Null),
        (4, Value::Null),
        (4, Value::Real(1.5)),
        (5, Value::Text("other".into())),
        (5, Value::Null),
        (6, Value::Null),
        (6, Value::Integer(0)),
        (6, Value::Integer(-1)),
        (6, Value::Integer(i64::MIN)),
        (6, Value::Real(1.5)),
        (7, Value::Blob(vec![4; 16])),
    ] {
        let mut row = valid.clone();
        row[index] = bad;
        assert!(
            c.execute(
                "INSERT INTO main.recurrence_obligations VALUES(?1,?2,?3,?4,?5,?6,?7,?8,'home')",
                rusqlite::params_from_iter(row)
            )
            .is_err(),
            "OBLIGATION_INVALID_ACCEPTED:{index}"
        );
        assert!(rows(&c, "SELECT * FROM main.recurrence_obligations").is_empty());
    }
    for state in ["awaiting_value", "overdue", "paid"] {
        let mut row = valid.clone();
        row[5] = Value::Text(state.into());
        if state == "overdue" {
            row[6] = Value::Integer(0);
        }
        assert!(
            c.execute(
                "INSERT INTO main.recurrence_obligations VALUES(?1,?2,?3,?4,?5,?6,?7,?8,'home')",
                rusqlite::params_from_iter(row)
            )
            .is_err(),
            "STATE_SHAPE_ACCEPTED:{state}"
        );
    }
    for movement in [vec![0_u8; 16], vec![9_u8; 16], vec![4_u8; 15]] {
        assert!(
            c.execute(
                "INSERT INTO main.recurrence_obligations VALUES(?1,?2,2026,1,0,'paid',7,?3,'home')",
                params![[10_u8; 16].as_slice(), [1_u8; 16].as_slice(), movement]
            )
            .is_err(),
            "INVALID_PAID_LINK_ACCEPTED"
        );
    }
    for context in [Value::Null, Value::Text("other".into())] {
        assert!(c.execute("INSERT INTO main.recurrence_obligations VALUES(?1,?2,2026,1,0,'payable',7,NULL,?3)",params![[10_u8;16].as_slice(),[1_u8;16].as_slice(),context]).is_err(),"INVALID_CONTEXT_ACCEPTED");
    }
    obligation(&c, 10, 1, "paid", Some(7), Some(vec![4; 16]));
    let before = rows(&c, "SELECT * FROM main.recurrence_obligations");
    assert!(c.execute("INSERT INTO main.recurrence_obligations VALUES(?1,?2,2026,1,0,'payable',7,NULL,'home')",params![[11_u8;16].as_slice(),[1_u8;16].as_slice()]).is_err(),"PERIOD_DUPLICATE_ACCEPTED");
    assert!(
        c.execute(
            "INSERT INTO main.recurrence_obligations VALUES(?1,?2,2026,2,0,'paid',7,?3,'home')",
            params![
                [11_u8; 16].as_slice(),
                [1_u8; 16].as_slice(),
                [4_u8; 16].as_slice()
            ]
        )
        .is_err(),
        "MOVEMENT_REUSED_ACCEPTED"
    );
    assert!(c
        .execute(
            "DELETE FROM main.recurrences WHERE id=?1",
            params![[1_u8; 16].as_slice()]
        )
        .is_err());
    assert!(c
        .execute(
            "DELETE FROM main.movements WHERE id=?1",
            params![[4_u8; 16].as_slice()]
        )
        .is_err());
    assert!(c
        .execute(
            "UPDATE main.movements SET id=?1 WHERE id=?2",
            params![[9_u8; 16].as_slice(), [4_u8; 16].as_slice()]
        )
        .is_err());
    assert_eq!(
        rows(&c, "SELECT * FROM main.recurrence_obligations"),
        before
    );
    health(&c, 7);
}

#[test]
fn recurrence_upgrade_preserves_existing_financial_history_and_reopens() {
    let (_d, p, account) = super::exit_persistence::setup();
    let c = open_database(&p).unwrap();
    c.execute(
        "INSERT INTO main.categories VALUES(?1,'Histórico',0)",
        params![[8_u8; 16].as_slice()],
    )
    .unwrap();
    c.execute("INSERT INTO main.movements(id,kind,amount_cents,context,account_id,category_id) VALUES(?1,'entry',5,'factory',?2,?3)",params![[9_u8;16].as_slice(),account.as_slice(),[8_u8;16].as_slice()]).unwrap();
    let before = super::exit_persistence::snapshot(&c);
    let categories = rows(&c, "SELECT * FROM main.categories");
    let movements = rows(&c, "SELECT * FROM main.movements");
    let c = migrate(c);
    health(&c, 7);
    drop(c);
    let c = open_database(&p).unwrap();
    health(&c, 7);
    let after = super::exit_persistence::snapshot(&c);
    assert_eq!(before.0, after.0);
    assert_eq!(before.1, after.1);
    assert_eq!(before.2, after.2);
    assert_eq!(rows(&c, "SELECT * FROM main.categories"), categories);
    assert_eq!(rows(&c, "SELECT * FROM main.movements"), movements);
    assert!(rows(&c, "SELECT * FROM main.recurrences").is_empty());
    assert!(rows(&c, "SELECT * FROM main.recurrence_obligations").is_empty());
}

#[test]
fn failure_on_second_table_rolls_back_first_table_and_ledger_then_retry_succeeds() {
    let d = TempDirectory::new();
    let p = d.path().join("rec007-rollback.sqlite");
    let c = v6(&p);
    c.execute(
        "CREATE TABLE main.recurrence_obligations(sentinel BLOB)",
        [],
    )
    .unwrap();
    c.execute(
        "INSERT INTO main.recurrence_obligations VALUES(?1)",
        params![[42_u8; 16].as_slice()],
    )
    .unwrap();
    let before = rows(
        &c,
        "SELECT type,name,tbl_name,sql FROM main.sqlite_schema ORDER BY type,name",
    );
    let old = super::exit_persistence::snapshot(&c);
    assert!(matches!(
        execute_versioned_migration(c, RECURRENCES_MIGRATION_VERSION, RECURRENCES_MIGRATION_SQL),
        Err(MigrationError::ApplyFailed)
    ));
    let c = open_database(&p).unwrap();
    health(&c, 6);
    assert_eq!(
        rows(
            &c,
            "SELECT type,name,tbl_name,sql FROM main.sqlite_schema ORDER BY type,name"
        ),
        before
    );
    assert_eq!(super::exit_persistence::snapshot(&c), old);
    assert_eq!(
        rows(&c, "SELECT * FROM main.recurrence_obligations"),
        vec![vec![Value::Blob(vec![42; 16])]]
    );
    c.execute("DROP TABLE main.recurrence_obligations", [])
        .unwrap();
    let c = migrate(c);
    health(&c, 7);
    drop(c);
    health(&open_database(&p).unwrap(), 7);
}
