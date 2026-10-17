// FIN-016 constraints tests
use super::TempDirectory;
use millani_artes::database::{
    execute_versioned_migration, open_database, schema_version, ACCOUNTS_MIGRATION_SQL,
    ACCOUNTS_MIGRATION_VERSION, AUDIT_EVENT_MIGRATION_SQL, AUDIT_EVENT_MIGRATION_VERSION,
    CATEGORIES_MIGRATION_SQL, CATEGORIES_MIGRATION_VERSION, MOVEMENTS_MIGRATION_SQL,
    MOVEMENTS_MIGRATION_VERSION,
};
use rusqlite::{params, params_from_iter, types::Value, Connection, ErrorCode};
use std::path::Path;

fn open_v5(path: &Path) -> Connection {
    let conn = open_database(path).expect("open");
    let mut conn = execute_versioned_migration(conn, 1, "").expect("v1");
    let migrations = [
        (AUDIT_EVENT_MIGRATION_VERSION, AUDIT_EVENT_MIGRATION_SQL),
        (ACCOUNTS_MIGRATION_VERSION, ACCOUNTS_MIGRATION_SQL),
        (CATEGORIES_MIGRATION_VERSION, CATEGORIES_MIGRATION_SQL),
        (MOVEMENTS_MIGRATION_VERSION, MOVEMENTS_MIGRATION_SQL),
    ];
    for (v, s) in migrations {
        conn = execute_versioned_migration(conn, v, s).expect("migrate");
    }
    conn
}

fn seed_accounts(conn: &Connection) -> (Vec<u8>, Vec<u8>) {
    let a1 = vec![0xAAu8; 16];
    let a2 = vec![0xBBu8; 16];
    conn.execute(
        "INSERT INTO accounts(id, name, account_type, initial_balance_cents) VALUES (?1, ?2, ?3, ?4)",
        params![a1.as_slice(), "a1", "bank_pix", 0_i64],
    ).unwrap();
    conn.execute(
        "INSERT INTO accounts(id, name, account_type, initial_balance_cents) VALUES (?1, ?2, ?3, ?4)",
        params![a2.as_slice(), "a2", "bank_pix", 0_i64],
    ).unwrap();
    (a1, a2)
}

fn snapshot_movements(conn: &Connection) -> Vec<Vec<Value>> {
    let mut stmt = conn.prepare(
        "SELECT id,kind,amount_cents,context,account_id,source_account_id,destination_account_id FROM main.movements ORDER BY id"
    ).unwrap();
    stmt.query_map([], |r| {
        Ok(vec![
            r.get::<_, Value>(0).unwrap(),
            r.get::<_, Value>(1).unwrap(),
            r.get::<_, Value>(2).unwrap(),
            r.get::<_, Value>(3).unwrap(),
            r.get::<_, Value>(4).unwrap(),
            r.get::<_, Value>(5).unwrap(),
            r.get::<_, Value>(6).unwrap(),
        ])
    })
    .unwrap()
    .map(|r| r.unwrap())
    .collect()
}

fn snapshot_accounts(conn: &Connection) -> Vec<Vec<Value>> {
    let mut stmt = conn
        .prepare("SELECT id,name,account_type,initial_balance_cents FROM main.accounts ORDER BY id")
        .unwrap();
    stmt.query_map([], |r| {
        Ok(vec![
            r.get::<_, Value>(0).unwrap(),
            r.get::<_, Value>(1).unwrap(),
            r.get::<_, Value>(2).unwrap(),
            r.get::<_, Value>(3).unwrap(),
        ])
    })
    .unwrap()
    .map(|r| r.unwrap())
    .collect()
}

fn ledger_versions(conn: &Connection) -> Vec<i32> {
    let mut stmt = conn
        .prepare("SELECT version FROM main.schema_migration ORDER BY version")
        .unwrap();
    stmt.query_map([], |r| r.get(0))
        .unwrap()
        .map(|r| r.unwrap())
        .collect()
}

fn try_insert(conn: &Connection, vals: &[Value]) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO main.movements(id,kind,amount_cents,context,account_id,source_account_id,destination_account_id) VALUES (?,?,?,?,?,?,?)",
        params_from_iter(vals.iter()),
    ).map(|_| ())
}

fn is_constraint_violation(e: &rusqlite::Error) -> bool {
    matches!(e, rusqlite::Error::SqliteFailure(err, _) if err.code == ErrorCode::ConstraintViolation)
}

fn integrity_ok(conn: &Connection) -> bool {
    conn.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
        .unwrap()
        == "ok"
}

fn assert_extended(error: &rusqlite::Error, code: i32) {
    assert!(matches!(error,rusqlite::Error::SqliteFailure(e,_) if e.extended_code==code));
}
fn assert_preserved(conn: &Connection, movements: &[Vec<Value>], accounts: &[Vec<Value>]) {
    assert_eq!(snapshot_movements(conn), movements);
    assert_eq!(snapshot_accounts(conn), accounts);
    assert_eq!(ledger_versions(conn), [1, 2, 3, 4, 5]);
    assert_eq!(schema_version(conn, 5).unwrap(), 5);
    assert!(conn.is_autocommit());
    assert!(integrity_ok(conn));
    let mut statement = conn.prepare("PRAGMA main.foreign_key_check").unwrap();
    assert!(statement.query([]).unwrap().next().unwrap().is_none());
}

#[test]
fn fin016_invalid_rows_preserve_seed() {
    let dir = TempDirectory::new();
    let path = dir.path().join("db.sqlite");
    let connection = open_v5(&path);
    let (a1, a2) = seed_accounts(&connection);
    let valid_id = vec![0x01u8; 16];
    let vals = [
        Value::Blob(valid_id.clone()),
        Value::Text("entry".into()),
        Value::Integer(100),
        Value::Text("home".into()),
        Value::Blob(a1.clone()),
        Value::Null,
        Value::Null,
    ];
    try_insert(&connection, &vals).unwrap();
    let snap0 = snapshot_movements(&connection);
    let ledger0 = ledger_versions(&connection);
    let acc0 = snapshot_accounts(&connection);
    assert_eq!(ledger0, [1, 2, 3, 4, 5]);
    assert_eq!(schema_version(&connection, 5).unwrap(), 5);
    assert!(connection.is_autocommit());
    assert!(integrity_ok(&connection));

    // FIN016_RED_CONTROL
    let first_id = Value::Blob(vec![0xE0u8; 15]);
    let first_vals = [
        first_id,
        Value::Text("entry".into()),
        Value::Integer(1),
        Value::Text("home".into()),
        Value::Blob(a1.clone()),
        Value::Null,
        Value::Null,
    ];
    let first_error = try_insert(&connection, &first_vals).expect_err("invalid row accepted: id15");
    assert!(is_constraint_violation(&first_error));
    assert_preserved(&connection, &snap0, &acc0);

    let mut cur_id = 2u8;
    let mut cases: Vec<Vec<Value>> = Vec::new();
    let base_id = |id: u8| {
        let mut v = vec![0u8; 16];
        v[0] = id;
        v
    };

    // NULL required
    for col in 0..4 {
        let mut v = vec![
            Value::Blob(base_id(cur_id)),
            Value::Text("entry".into()),
            Value::Integer(1),
            Value::Text("home".into()),
            Value::Blob(a1.clone()),
            Value::Null,
            Value::Null,
        ];
        cur_id += 1;
        match col {
            0 => v[0] = Value::Null,
            1 => v[1] = Value::Null,
            2 => v[2] = Value::Null,
            3 => v[3] = Value::Null,
            _ => {}
        }
        cases.push(v);
    }
    // id variants
    for idv in [
        Value::Text("X".repeat(16)),
        Value::Blob(vec![]),
        Value::Blob(vec![0u8; 15]),
        Value::Blob(vec![0u8; 17]),
    ] {
        cases.push(vec![
            idv,
            Value::Text("entry".into()),
            Value::Integer(1),
            Value::Text("home".into()),
            Value::Blob(a1.clone()),
            Value::Null,
            Value::Null,
        ]);
        cur_id += 1;
    }
    // kind/context/amount variants
    cases.push(vec![
        Value::Blob(base_id(cur_id)),
        Value::Text("Entry".into()),
        Value::Integer(1),
        Value::Text("home".into()),
        Value::Blob(a1.clone()),
        Value::Null,
        Value::Null,
    ]);
    cur_id += 1;
    cases.push(vec![
        Value::Blob(base_id(cur_id)),
        Value::Blob(vec![0u8; 16]),
        Value::Integer(1),
        Value::Text("home".into()),
        Value::Blob(a1.clone()),
        Value::Null,
        Value::Null,
    ]);
    cur_id += 1;
    cases.push(vec![
        Value::Blob(base_id(cur_id)),
        Value::Text("entry".into()),
        Value::Integer(1),
        Value::Text("Home".into()),
        Value::Blob(a1.clone()),
        Value::Null,
        Value::Null,
    ]);
    cur_id += 1;
    cases.push(vec![
        Value::Blob(base_id(cur_id)),
        Value::Text("entry".into()),
        Value::Integer(1),
        Value::Blob(vec![0u8; 16]),
        Value::Blob(a1.clone()),
        Value::Null,
        Value::Null,
    ]);
    cur_id += 1;
    cases.push(vec![
        Value::Blob(base_id(cur_id)),
        Value::Text("entry".into()),
        Value::Blob(vec![0u8; 16]),
        Value::Text("home".into()),
        Value::Blob(a1.clone()),
        Value::Null,
        Value::Null,
    ]);
    cur_id += 1;

    // account datatype/length probes on Entry
    for idv in [
        Value::Text("X".repeat(16)),
        Value::Blob(vec![]),
        Value::Blob(vec![0u8; 15]),
        Value::Blob(vec![0u8; 17]),
    ] {
        cases.push(vec![
            Value::Blob(base_id(cur_id)),
            Value::Text("entry".into()),
            Value::Integer(1),
            Value::Text("home".into()),
            idv,
            Value::Null,
            Value::Null,
        ]);
        cur_id += 1;
    }
    // transfer source/dest datatype/length probes
    for idv in [
        Value::Text("X".repeat(16)),
        Value::Blob(vec![]),
        Value::Blob(vec![0u8; 15]),
        Value::Blob(vec![0u8; 17]),
    ] {
        cases.push(vec![
            Value::Blob(base_id(cur_id)),
            Value::Text("transfer".into()),
            Value::Integer(1),
            Value::Text("home".into()),
            Value::Null,
            idv.clone(),
            Value::Blob(a2.clone()),
        ]);
        cur_id += 1;
        cases.push(vec![
            Value::Blob(base_id(cur_id)),
            Value::Text("transfer".into()),
            Value::Integer(1),
            Value::Text("home".into()),
            Value::Null,
            Value::Blob(a1.clone()),
            idv,
        ]);
        cur_id += 1;
    }
    for (column, invalid) in [
        (1, Value::Text("unknown".into())),
        (3, Value::Text("unknown".into())),
        (2, Value::Real(1.5)),
        (2, Value::Text("not-a-number".into())),
    ] {
        let mut row = vec![
            Value::Blob(base_id(cur_id)),
            Value::Text("entry".into()),
            Value::Integer(1),
            Value::Text("home".into()),
            Value::Blob(a1.clone()),
            Value::Null,
            Value::Null,
        ];
        cur_id += 1;
        row[column] = invalid;
        cases.push(row);
    }
    cases.push(vec![
        Value::Blob(base_id(cur_id)),
        Value::Text("exit".into()),
        Value::Integer(1),
        Value::Text("home".into()),
        Value::Blob(a1.clone()),
        Value::Null,
        Value::Blob(a2.clone()),
    ]);
    cur_id += 1;
    // shape errors
    cases.push(vec![
        Value::Blob(base_id(cur_id)),
        Value::Text("entry".into()),
        Value::Integer(1),
        Value::Text("home".into()),
        Value::Null,
        Value::Null,
        Value::Null,
    ]);
    cur_id += 1;
    cases.push(vec![
        Value::Blob(base_id(cur_id)),
        Value::Text("entry".into()),
        Value::Integer(1),
        Value::Text("home".into()),
        Value::Blob(a1.clone()),
        Value::Blob(a2.clone()),
        Value::Null,
    ]);
    cur_id += 1;
    cases.push(vec![
        Value::Blob(base_id(cur_id)),
        Value::Text("entry".into()),
        Value::Integer(1),
        Value::Text("home".into()),
        Value::Blob(a1.clone()),
        Value::Null,
        Value::Blob(a2.clone()),
    ]);
    cur_id += 1;
    cases.push(vec![
        Value::Blob(base_id(cur_id)),
        Value::Text("exit".into()),
        Value::Integer(1),
        Value::Text("home".into()),
        Value::Null,
        Value::Null,
        Value::Null,
    ]);
    cur_id += 1;
    cases.push(vec![
        Value::Blob(base_id(cur_id)),
        Value::Text("exit".into()),
        Value::Integer(1),
        Value::Text("home".into()),
        Value::Blob(a1.clone()),
        Value::Blob(a2.clone()),
        Value::Null,
    ]);
    cur_id += 1;
    cases.push(vec![
        Value::Blob(base_id(cur_id)),
        Value::Text("transfer".into()),
        Value::Integer(1),
        Value::Text("home".into()),
        Value::Blob(a1.clone()),
        Value::Blob(a2.clone()),
        Value::Blob(a2.clone()),
    ]);
    cur_id += 1;
    cases.push(vec![
        Value::Blob(base_id(cur_id)),
        Value::Text("transfer".into()),
        Value::Integer(1),
        Value::Text("home".into()),
        Value::Null,
        Value::Null,
        Value::Blob(a2.clone()),
    ]);
    cur_id += 1;
    cases.push(vec![
        Value::Blob(base_id(cur_id)),
        Value::Text("transfer".into()),
        Value::Integer(1),
        Value::Text("home".into()),
        Value::Null,
        Value::Blob(a1.clone()),
        Value::Null,
    ]);

    assert_eq!(cases.len(), 38);
    for vals in cases {
        let before = snapshot_movements(&connection);
        let res = try_insert(&connection, &vals);
        assert!(res.is_err());
        assert!(is_constraint_violation(res.as_ref().unwrap_err()));
        let after = snapshot_movements(&connection);
        assert_eq!(before, after);
        assert_eq!(ledger_versions(&connection), ledger0);
        assert!(connection.is_autocommit());
        assert!(integrity_ok(&connection));
        assert_preserved(&connection, &snap0, &acc0);
    }
    drop(connection);
    let reopen_conn = open_database(&path).expect("reopen");
    assert_eq!(snapshot_movements(&reopen_conn), snap0);
    assert_eq!(ledger_versions(&reopen_conn), ledger0);
    assert!(integrity_ok(&reopen_conn));
    assert_preserved(&reopen_conn, &snap0, &acc0);
}

#[test]
fn fin016_duplicate_and_references_preserve_history() {
    let dir = TempDirectory::new();
    let path = dir.path().join("db.sqlite");
    let conn = open_v5(&path);
    let (a1, a2) = seed_accounts(&conn);
    let mk = |id: u8,
              kind: &str,
              amt: i64,
              acc: Option<Vec<u8>>,
              src: Option<Vec<u8>>,
              dst: Option<Vec<u8>>|
     -> Vec<Value> {
        let mut v = vec![0u8; 16];
        v[0] = id;
        let mut vals = vec![
            Value::Blob(v),
            Value::Text(kind.into()),
            Value::Integer(amt),
            Value::Text("home".into()),
            Value::Null,
            Value::Null,
            Value::Null,
        ];
        if let Some(a) = acc {
            vals[4] = Value::Blob(a);
        }
        if let Some(s) = src {
            vals[5] = Value::Blob(s);
        }
        if let Some(d) = dst {
            vals[6] = Value::Blob(d);
        }
        vals
    };
    let e1 = mk(1, "entry", 0, Some(a1.clone()), None, None);
    let e2 = mk(2, "exit", -5, Some(a2.clone()), None, None);
    let t1 = mk(3, "transfer", 0, None, Some(a1.clone()), Some(a2.clone()));
    try_insert(&conn, &e1).unwrap();
    try_insert(&conn, &e2).unwrap();
    try_insert(&conn, &t1).unwrap();
    let snap0 = snapshot_movements(&conn);
    let acc0 = snapshot_accounts(&conn);
    let ledger0 = ledger_versions(&conn);
    assert_eq!(schema_version(&conn, 5).unwrap(), 5);

    // duplicate PK
    let dup = mk(1, "entry", 1, Some(a1.clone()), None, None);
    let res = try_insert(&conn, &dup);
    assert!(res.is_err());
    assert!(is_constraint_violation(res.as_ref().unwrap_err()));
    assert_extended(
        res.as_ref().unwrap_err(),
        rusqlite::ffi::SQLITE_CONSTRAINT_PRIMARYKEY,
    );
    assert_preserved(&conn, &snap0, &acc0);
    assert_eq!(ledger_versions(&conn), ledger0);
    assert!(integrity_ok(&conn));

    // dangling FK
    let bad_acc = vec![0xFFu8; 16];
    let dang_entry = mk(4, "entry", 1, Some(bad_acc.clone()), None, None);
    let res = try_insert(&conn, &dang_entry);
    assert!(res.is_err());
    assert!(is_constraint_violation(res.as_ref().unwrap_err()));
    assert_extended(
        res.as_ref().unwrap_err(),
        rusqlite::ffi::SQLITE_CONSTRAINT_FOREIGNKEY,
    );
    assert_preserved(&conn, &snap0, &acc0);

    let dang_exit = mk(5, "exit", 1, Some(bad_acc.clone()), None, None);
    let res = try_insert(&conn, &dang_exit);
    assert!(res.is_err());
    assert!(is_constraint_violation(res.as_ref().unwrap_err()));
    assert_extended(
        res.as_ref().unwrap_err(),
        rusqlite::ffi::SQLITE_CONSTRAINT_FOREIGNKEY,
    );
    assert_preserved(&conn, &snap0, &acc0);

    let dang_t_src = mk(
        6,
        "transfer",
        1,
        None,
        Some(bad_acc.clone()),
        Some(a2.clone()),
    );
    let res = try_insert(&conn, &dang_t_src);
    assert!(res.is_err());
    assert!(is_constraint_violation(res.as_ref().unwrap_err()));
    assert_extended(
        res.as_ref().unwrap_err(),
        rusqlite::ffi::SQLITE_CONSTRAINT_FOREIGNKEY,
    );
    assert_preserved(&conn, &snap0, &acc0);

    let dang_t_dst = mk(
        7,
        "transfer",
        1,
        None,
        Some(a1.clone()),
        Some(bad_acc.clone()),
    );
    let res = try_insert(&conn, &dang_t_dst);
    assert!(res.is_err());
    assert!(is_constraint_violation(res.as_ref().unwrap_err()));
    assert_extended(
        res.as_ref().unwrap_err(),
        rusqlite::ffi::SQLITE_CONSTRAINT_FOREIGNKEY,
    );
    assert_preserved(&conn, &snap0, &acc0);

    // NO ACTION delete/update blocked
    let del_res = conn.execute("DELETE FROM accounts WHERE id = ?", params![a1.as_slice()]);
    assert!(del_res.is_err());
    assert!(is_constraint_violation(del_res.as_ref().unwrap_err()));
    assert_extended(
        del_res.as_ref().unwrap_err(),
        rusqlite::ffi::SQLITE_CONSTRAINT_FOREIGNKEY,
    );
    assert_preserved(&conn, &snap0, &acc0);
    let upd_res = conn.execute(
        "UPDATE accounts SET id = ? WHERE id = ?",
        params![vec![0xCCu8; 16].as_slice(), a1.as_slice()],
    );
    assert!(upd_res.is_err());
    assert!(is_constraint_violation(upd_res.as_ref().unwrap_err()));
    assert_extended(
        upd_res.as_ref().unwrap_err(),
        rusqlite::ffi::SQLITE_CONSTRAINT_FOREIGNKEY,
    );
    assert_preserved(&conn, &snap0, &acc0);

    assert_eq!(snapshot_movements(&conn), snap0);
    assert_eq!(snapshot_accounts(&conn), acc0);
    assert_eq!(ledger_versions(&conn), ledger0);
    assert!(conn.is_autocommit());
    assert!(integrity_ok(&conn));

    drop(conn);
    let reopen = open_database(&path).expect("reopen");
    assert_eq!(snapshot_movements(&reopen), snap0);
    assert_eq!(snapshot_accounts(&reopen), acc0);
    assert_eq!(ledger_versions(&reopen), ledger0);
    assert!(integrity_ok(&reopen));
    assert_preserved(&reopen, &snap0, &acc0);
}
