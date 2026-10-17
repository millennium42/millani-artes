// movements_migration.rs
use millani_artes::database::{
    append_audit_event, execute_versioned_migration, execute_write_transaction, open_database,
    schema_version, AuditEventError, MigrationError, ACCOUNTS_MIGRATION_SQL,
    ACCOUNTS_MIGRATION_VERSION, AUDIT_EVENT_MIGRATION_SQL, AUDIT_EVENT_MIGRATION_VERSION,
    CATEGORIES_MIGRATION_SQL, CATEGORIES_MIGRATION_VERSION, MOVEMENTS_MIGRATION_SQL,
    MOVEMENTS_MIGRATION_VERSION,
};
use rusqlite::Connection;

type Column = (i64, String, String, i64, Option<String>, i64, i64);
type AuditRow = (Vec<u8>, i64, String, String, Vec<u8>);
type MovementRow = (
    Vec<u8>,
    String,
    i64,
    String,
    Option<Vec<u8>>,
    Option<Vec<u8>>,
    Option<Vec<u8>>,
);

fn db013_event(
    event_id: [u8; 16],
    occurred_at_ms: i64,
    operation: &'static str,
    entity: &'static str,
    entity_id: [u8; 16],
) -> millani_artes::database::AuditEvent {
    millani_artes::database::AuditEvent {
        event_id,
        occurred_at_ms,
        operation,
        entity,
        entity_id,
    }
}

#[test]
fn mm001_movements_migrate_empty_database_and_enforce_constraints() {
    use super::TempDirectory;

    let directory = TempDirectory::new();
    let path = directory.path().join("mm001-empty.sqlite");
    let connection = open_database(&path).expect("DATABASE_OPEN_FAILED");
    let connection =
        execute_versioned_migration(connection, 1, "").expect("BASELINE_MIGRATION_FAILED");
    let connection = execute_versioned_migration(
        connection,
        AUDIT_EVENT_MIGRATION_VERSION,
        AUDIT_EVENT_MIGRATION_SQL,
    )
    .expect("AUDIT_EVENT_MIGRATION_FAILED");
    let connection = execute_versioned_migration(
        connection,
        ACCOUNTS_MIGRATION_VERSION,
        ACCOUNTS_MIGRATION_SQL,
    )
    .expect("ACCOUNTS_MIGRATION_FAILED");
    let connection = execute_versioned_migration(
        connection,
        CATEGORIES_MIGRATION_VERSION,
        CATEGORIES_MIGRATION_SQL,
    )
    .expect("CATEGORIES_MIGRATION_FAILED");
    let connection = execute_versioned_migration(
        connection,
        MOVEMENTS_MIGRATION_VERSION,
        MOVEMENTS_MIGRATION_SQL,
    )
    .expect("MOVEMENTS_MIGRATION_FAILED");

    assert!(connection.is_autocommit());
    assert_eq!(schema_version(&connection, 5).unwrap(), 5);

    let versions: Vec<i64> = {
        let mut stmt = connection
            .prepare("SELECT version FROM main.schema_migration ORDER BY version")
            .unwrap();
        stmt.query_map([], |r| r.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(versions, [1, 2, 3, 4, 5]);

    let sql: String = connection
        .query_row(
            "SELECT sql FROM main.sqlite_master WHERE type='table' AND name='movements'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(sql.contains("STRICT"));

    // Verify table columns
    let cols: Vec<Column> = {
        let mut stmt = connection
            .prepare("PRAGMA main.table_xinfo('movements')")
            .unwrap();
        stmt.query_map([], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
            ))
        })
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap()
    };
    assert_eq!(cols.len(), 7);

    // Verify column names
    let col_names: Vec<String> = cols.iter().map(|c| c.1.clone()).collect();
    assert_eq!(
        col_names,
        vec![
            "id",
            "kind",
            "amount_cents",
            "context",
            "account_id",
            "source_account_id",
            "destination_account_id"
        ]
    );

    // Verify column types
    let col_types: Vec<String> = cols.iter().map(|c| c.2.clone()).collect();
    assert_eq!(
        col_types,
        vec!["BLOB", "TEXT", "INTEGER", "TEXT", "BLOB", "BLOB", "BLOB"]
    );

    // Verify notnull
    let col_notnull: Vec<i64> = cols.iter().map(|c| c.3).collect();
    assert_eq!(col_notnull, vec![1, 1, 1, 1, 0, 0, 0]);

    // Verify defaults are None
    for col in &cols {
        assert_eq!(col.4, None);
    }

    // Verify primary key (id is column 0, should be PK)
    assert_eq!(cols[0].5, 1);

    // Verify hidden is 0 for all columns
    for col in &cols {
        assert_eq!(col.6, 0);
    }

    // Verify foreign keys
    let mut fks: Vec<(i64, i64, String, String, String, String, String)> = {
        let mut stmt = connection
            .prepare("PRAGMA main.foreign_key_list('movements')")
            .unwrap();
        stmt.query_map([], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
            ))
        })
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap()
    };
    assert_eq!(fks.len(), 3);
    fks.sort_by(|a, b| a.3.cmp(&b.3));

    // Verify FK mappings
    let fk_from: Vec<String> = fks.iter().map(|f| f.3.clone()).collect();
    assert_eq!(
        fk_from,
        vec!["account_id", "destination_account_id", "source_account_id"]
    );

    let fk_to: Vec<String> = fks.iter().map(|f| f.4.clone()).collect();
    assert_eq!(fk_to, vec!["id", "id", "id"]);

    let fk_table: Vec<String> = fks.iter().map(|f| f.2.clone()).collect();
    assert_eq!(fk_table, vec!["accounts", "accounts", "accounts"]);

    for (_, _, _, _, _, on_update, on_delete) in &fks {
        assert_eq!(on_update, "NO ACTION");
        assert_eq!(on_delete, "NO ACTION");
    }

    // Verify index
    let idxs: Vec<(i64, String, i64, String)> = {
        let mut stmt = connection
            .prepare("PRAGMA main.index_list('movements')")
            .unwrap();
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(idxs.len(), 1);

    // Verify index is PK (unique)
    assert_eq!(idxs[0].2, 1);
    assert_eq!(idxs[0].3, "pk");

    let acc_a = [0xAA_u8; 16];
    let acc_b: [u8; 16] = [0x32; 16];
    connection.execute(
        "INSERT INTO main.accounts (id, name, account_type, initial_balance_cents) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![acc_a.as_slice(), "A", "bank_pix", 0_i64],
    ).unwrap();
    connection.execute(
        "INSERT INTO main.accounts (id, name, account_type, initial_balance_cents) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![acc_b.as_slice(), "B", "bank_pix", 0_i64],
    ).unwrap();

    let count: i64 = connection
        .query_row("SELECT COUNT(*) FROM main.movements", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 0);

    let amounts = [-9223372036854775808_i64, -1, 0, 1, 9223372036854775807];
    let kinds = ["entry", "exit", "transfer"];
    let contexts = ["home", "factory"];

    let read_rows = |conn: &Connection| -> Vec<MovementRow> {
        let mut stmt = conn.prepare("SELECT id, kind, amount_cents, context, account_id, source_account_id, destination_account_id FROM main.movements ORDER BY id").unwrap();
        stmt.query_map([], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
            ))
        })
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap()
    };

    let mut expected = Vec::new();
    let mut counter: u8 = 0;
    for kind in kinds.iter() {
        for ctx in contexts.iter() {
            for &amt in amounts.iter() {
                let id = [counter; 16];
                counter += 1;
                if *kind == "transfer" {
                    connection.execute(
                        "INSERT INTO main.movements (id, kind, amount_cents, context, source_account_id, destination_account_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                        rusqlite::params![id.as_slice(), *kind, amt, *ctx, acc_a.as_slice(), acc_b.as_slice()],
                    ).unwrap();
                    expected.push((
                        id.to_vec(),
                        kind.to_string(),
                        amt,
                        ctx.to_string(),
                        None,
                        Some(acc_a.to_vec()),
                        Some(acc_b.to_vec()),
                    ));
                } else {
                    connection.execute(
                        "INSERT INTO main.movements (id, kind, amount_cents, context, account_id) VALUES (?1, ?2, ?3, ?4, ?5)",
                        rusqlite::params![id.as_slice(), *kind, amt, *ctx, acc_a.as_slice()],
                    ).unwrap();
                    expected.push((
                        id.to_vec(),
                        kind.to_string(),
                        amt,
                        ctx.to_string(),
                        Some(acc_a.to_vec()),
                        None,
                        None,
                    ));
                }
            }
        }
    }

    let mut rows = read_rows(&connection);
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    let mut exp_sorted = expected.clone();
    exp_sorted.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(rows, exp_sorted);

    // Test invalid kind constraint
    let before = read_rows(&connection);
    let invalid_id = [0xE0_u8; 16];
    let _ = connection.execute(
        "INSERT INTO main.movements (id, kind, amount_cents, context, account_id) VALUES (?1, 'badkind', 0, 'home', ?2)",
        rusqlite::params![invalid_id.as_slice(), acc_a.as_slice()],
    ).expect_err("invalid kind");
    let after = read_rows(&connection);
    assert_eq!(before, after);

    // Test missing account FK
    let missing_acc_id = [0xEE_u8; 16];
    let missing_entry_id = [0xEF_u8; 16];
    let _ = connection.execute(
        "INSERT INTO main.movements (id, kind, amount_cents, context, account_id) VALUES (?1, 'entry', 100, 'home', ?2)",
        rusqlite::params![missing_entry_id.as_slice(), missing_acc_id.as_slice()],
    ).expect_err("missing account FK");

    let invalid_shape = connection
        .execute(
            "INSERT INTO main.movements(id,kind,amount_cents,context) VALUES (?1,'entry',1,'home')",
            rusqlite::params![[0xE1_u8; 16].as_slice()],
        )
        .unwrap_err();
    assert!(
        matches!(invalid_shape, rusqlite::Error::SqliteFailure(ref e, _) if e.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_CHECK)
    );
    let delete_error = connection
        .execute(
            "DELETE FROM main.accounts WHERE id=?1",
            rusqlite::params![acc_a.as_slice()],
        )
        .unwrap_err();
    assert!(
        matches!(delete_error, rusqlite::Error::SqliteFailure(ref e, _) if e.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_FOREIGNKEY)
    );
    assert_eq!(read_rows(&connection), before);
    assert_eq!(before.len(), 30);
    assert!(connection.is_autocommit());
    assert_eq!(schema_version(&connection, 5).unwrap(), 5);

    let integrity: String = connection
        .query_row("PRAGMA main.integrity_check", [], |r| r.get::<_, String>(0))
        .unwrap();
    assert_eq!(integrity, "ok");

    drop(connection);

    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert_eq!(schema_version(&reopened, 5).unwrap(), 5);
    assert!(reopened.is_autocommit());
    let rows2 = read_rows(&reopened);
    let mut rows2s = rows2;
    rows2s.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(rows2s, exp_sorted);
    assert_eq!(
        reopened
            .query_row("PRAGMA main.integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
}

#[test]
fn mm002_movements_upgrade_v4_preserves_accounts_and_audit_event() {
    use super::TempDirectory;

    let directory = TempDirectory::new();
    let path = directory.path().join("mm002-upgrade.sqlite");
    let connection = open_database(&path).expect("DATABASE_OPEN_FAILED");
    let connection =
        execute_versioned_migration(connection, 1, "").expect("BASELINE_MIGRATION_FAILED");
    let connection = execute_versioned_migration(
        connection,
        AUDIT_EVENT_MIGRATION_VERSION,
        AUDIT_EVENT_MIGRATION_SQL,
    )
    .expect("AUDIT_EVENT_MIGRATION_FAILED");
    let connection = execute_versioned_migration(
        connection,
        ACCOUNTS_MIGRATION_VERSION,
        ACCOUNTS_MIGRATION_SQL,
    )
    .expect("ACCOUNTS_MIGRATION_FAILED");
    let connection = execute_versioned_migration(
        connection,
        CATEGORIES_MIGRATION_VERSION,
        CATEGORIES_MIGRATION_SQL,
    )
    .expect("CATEGORIES_MIGRATION_FAILED");

    let acc1 = [0x31_u8; 16];
    let acc2: [u8; 16] = [0x32; 16];
    connection.execute(
        "INSERT INTO main.accounts (id, name, account_type, initial_balance_cents) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![acc1.as_slice(), "A", "bank_pix", 987654_i64],
    ).unwrap();
    connection.execute(
        "INSERT INTO main.accounts (id, name, account_type, initial_balance_cents) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![acc2.as_slice(), "B", "cash", 0_i64],
    ).unwrap();

    let cat_id = [0xC0_u8; 16];
    connection
        .execute(
            "INSERT INTO main.categories (id, name, is_active) VALUES (?1, ?2, ?3)",
            rusqlite::params![cat_id.as_slice(), "Test", 1_i64],
        )
        .unwrap();

    let event = db013_event(
        [0x41; 16],
        1_791_234_567_890,
        "category_schema_upgrade",
        "account",
        acc1,
    );
    let (connection, ()) = execute_write_transaction(connection, |tx| {
        append_audit_event(tx, &event)?;
        Ok::<(), AuditEventError>(())
    })
    .expect("AUDIT_EVENT_INSERT_FAILED");

    let accounts_snapshot: Vec<(Vec<u8>, String, String, i64)> = {
        let mut stmt = connection.prepare("SELECT id, name, account_type, initial_balance_cents FROM main.accounts ORDER BY id").unwrap();
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    let categories_snapshot: Vec<(Vec<u8>, String, i64)> = {
        let mut stmt = connection
            .prepare("SELECT id, name, is_active FROM main.categories ORDER BY id")
            .unwrap();
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    let audit_snapshot: Vec<AuditRow> = {
        let mut stmt = connection.prepare("SELECT event_id, occurred_at_ms, operation, entity, entity_id FROM main.audit_event ORDER BY event_id").unwrap();
        stmt.query_map([], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap()
    };
    let previous_schema: Vec<(String, String, String)> = {
        let mut stmt = connection.prepare("SELECT type, name, sql FROM main.sqlite_schema WHERE name NOT GLOB 'sqlite_*' ORDER BY name").unwrap();
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };

    let connection = execute_versioned_migration(
        connection,
        MOVEMENTS_MIGRATION_VERSION,
        MOVEMENTS_MIGRATION_SQL,
    )
    .expect("MOVEMENTS_MIGRATION_FAILED");

    assert!(connection.is_autocommit());
    assert_eq!(schema_version(&connection, 5).unwrap(), 5);

    let versions: Vec<i64> = {
        let mut stmt = connection
            .prepare("SELECT version FROM main.schema_migration ORDER BY version")
            .unwrap();
        stmt.query_map([], |r| r.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(versions, [1, 2, 3, 4, 5]);

    let accounts_after: Vec<(Vec<u8>, String, String, i64)> = {
        let mut stmt = connection.prepare("SELECT id, name, account_type, initial_balance_cents FROM main.accounts ORDER BY id").unwrap();
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(accounts_after, accounts_snapshot);

    let categories_after: Vec<(Vec<u8>, String, i64)> = {
        let mut stmt = connection
            .prepare("SELECT id, name, is_active FROM main.categories ORDER BY id")
            .unwrap();
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(categories_after, categories_snapshot);

    let audit_after: Vec<AuditRow> = {
        let mut stmt = connection.prepare("SELECT event_id, occurred_at_ms, operation, entity, entity_id FROM main.audit_event ORDER BY event_id").unwrap();
        stmt.query_map([], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap()
    };
    assert_eq!(audit_after, audit_snapshot);

    let after_schema: Vec<(String, String, String)> = {
        let mut stmt = connection.prepare("SELECT type, name, sql FROM main.sqlite_schema WHERE name NOT GLOB 'sqlite_*' AND name != 'movements' ORDER BY name").unwrap();
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(after_schema, previous_schema);

    assert_eq!(
        connection
            .query_row("PRAGMA main.integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );

    drop(connection);

    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert!(reopened.is_autocommit());
    assert_eq!(schema_version(&reopened, 5).unwrap(), 5);

    let accounts_reopen: Vec<(Vec<u8>, String, String, i64)> = {
        let mut stmt = reopened.prepare("SELECT id, name, account_type, initial_balance_cents FROM main.accounts ORDER BY id").unwrap();
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(accounts_reopen, accounts_snapshot);

    let categories_reopen: Vec<(Vec<u8>, String, i64)> = {
        let mut stmt = reopened
            .prepare("SELECT id, name, is_active FROM main.categories ORDER BY id")
            .unwrap();
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(categories_reopen, categories_snapshot);

    let audit_reopen: Vec<AuditRow> = {
        let mut stmt = reopened.prepare("SELECT event_id, occurred_at_ms, operation, entity, entity_id FROM main.audit_event ORDER BY event_id").unwrap();
        stmt.query_map([], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap()
    };
    assert_eq!(audit_reopen, audit_snapshot);

    assert_eq!(
        reopened
            .query_row("PRAGMA main.integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
}

#[test]
fn mm003_movements_collision_fixture_fails_and_retries() {
    use super::TempDirectory;

    let directory = TempDirectory::new();
    let path = directory.path().join("mm003-collision.sqlite");
    let connection = open_database(&path).expect("DATABASE_OPEN_FAILED");
    let connection =
        execute_versioned_migration(connection, 1, "").expect("BASELINE_MIGRATION_FAILED");
    let connection = execute_versioned_migration(
        connection,
        AUDIT_EVENT_MIGRATION_VERSION,
        AUDIT_EVENT_MIGRATION_SQL,
    )
    .expect("AUDIT_EVENT_MIGRATION_FAILED");
    let connection = execute_versioned_migration(
        connection,
        ACCOUNTS_MIGRATION_VERSION,
        ACCOUNTS_MIGRATION_SQL,
    )
    .expect("ACCOUNTS_MIGRATION_FAILED");
    let connection = execute_versioned_migration(
        connection,
        CATEGORIES_MIGRATION_VERSION,
        CATEGORIES_MIGRATION_SQL,
    )
    .expect("CATEGORIES_MIGRATION_FAILED");

    let acc1 = [0x31_u8; 16];
    let acc2: [u8; 16] = [0x32; 16];
    connection.execute(
        "INSERT INTO main.accounts (id, name, account_type, initial_balance_cents) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![acc1.as_slice(), "A", "bank_pix", 987654_i64],
    ).unwrap();
    connection.execute(
        "INSERT INTO main.accounts (id, name, account_type, initial_balance_cents) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![acc2.as_slice(), "B", "cash", 0_i64],
    ).unwrap();

    let cat_id = [0xC0_u8; 16];
    connection
        .execute(
            "INSERT INTO main.categories (id, name, is_active) VALUES (?1, ?2, ?3)",
            rusqlite::params![cat_id.as_slice(), "Test", 1_i64],
        )
        .unwrap();

    let event = db013_event(
        [0x41; 16],
        1_791_234_567_890,
        "category_schema_upgrade",
        "account",
        acc1,
    );
    let (connection, ()) = execute_write_transaction(connection, |tx| {
        append_audit_event(tx, &event)?;
        Ok::<(), AuditEventError>(())
    })
    .expect("AUDIT_EVENT_INSERT_FAILED");

    // Create fixture with sentinel ID
    let sentinel_id = [0x7D_u8; 16];
    connection
        .execute("CREATE TABLE main.movements (id BLOB PRIMARY KEY)", [])
        .unwrap();
    connection
        .execute(
            "INSERT INTO main.movements (id) VALUES (?1)",
            rusqlite::params![sentinel_id.as_slice()],
        )
        .unwrap();

    // Capture snapshots before failed migration
    let accounts_before: Vec<(Vec<u8>, String, String, i64)> = {
        let mut stmt = connection.prepare("SELECT id, name, account_type, initial_balance_cents FROM main.accounts ORDER BY id").unwrap();
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    let categories_before: Vec<(Vec<u8>, String, i64)> = {
        let mut stmt = connection
            .prepare("SELECT id, name, is_active FROM main.categories ORDER BY id")
            .unwrap();
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    let audit_before: Vec<AuditRow> = {
        let mut stmt = connection.prepare("SELECT event_id, occurred_at_ms, operation, entity, entity_id FROM main.audit_event ORDER BY event_id").unwrap();
        stmt.query_map([], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap()
    };
    let schema_before: Vec<(String, String, String)> = {
        let mut stmt = connection.prepare("SELECT type, name, sql FROM main.sqlite_schema WHERE name NOT GLOB 'sqlite_*' ORDER BY name").unwrap();
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };

    let result = execute_versioned_migration(
        connection,
        MOVEMENTS_MIGRATION_VERSION,
        MOVEMENTS_MIGRATION_SQL,
    );
    assert!(matches!(result, Err(MigrationError::ApplyFailed)));

    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert_eq!(schema_version(&reopened, 4).unwrap(), 4);
    assert!(reopened.is_autocommit());

    let ledger: Vec<i64> = {
        let mut stmt = reopened
            .prepare("SELECT version FROM main.schema_migration ORDER BY version")
            .unwrap();
        stmt.query_map([], |r| r.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(ledger, [1, 2, 3, 4]);

    // Verify sentinel still exists
    let count: i64 = reopened
        .query_row("SELECT COUNT(*) FROM main.movements", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);
    let saved_sentinel: Vec<u8> = reopened
        .query_row("SELECT id FROM main.movements", [], |r| r.get(0))
        .unwrap();
    assert_eq!(saved_sentinel, sentinel_id.to_vec());

    // Verify snapshots preserved
    let accounts_after_fail: Vec<(Vec<u8>, String, String, i64)> = {
        let mut stmt = reopened.prepare("SELECT id, name, account_type, initial_balance_cents FROM main.accounts ORDER BY id").unwrap();
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(accounts_after_fail, accounts_before);

    let categories_after_fail: Vec<(Vec<u8>, String, i64)> = {
        let mut stmt = reopened
            .prepare("SELECT id, name, is_active FROM main.categories ORDER BY id")
            .unwrap();
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(categories_after_fail, categories_before);

    let audit_after_fail: Vec<AuditRow> = {
        let mut stmt = reopened.prepare("SELECT event_id, occurred_at_ms, operation, entity, entity_id FROM main.audit_event ORDER BY event_id").unwrap();
        stmt.query_map([], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap()
    };
    assert_eq!(audit_after_fail, audit_before);

    let schema_after_fail: Vec<(String, String, String)> = {
        let mut stmt = reopened.prepare("SELECT type, name, sql FROM main.sqlite_schema WHERE name NOT GLOB 'sqlite_*' ORDER BY name").unwrap();
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(schema_after_fail, schema_before);

    assert_eq!(
        reopened
            .query_row("PRAGMA main.integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    drop(reopened);

    // Drop fixture and retry
    let connection = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    connection.execute("DROP TABLE main.movements", []).unwrap();
    let connection = execute_versioned_migration(
        connection,
        MOVEMENTS_MIGRATION_VERSION,
        MOVEMENTS_MIGRATION_SQL,
    )
    .expect("MOVEMENTS_MIGRATION_FAILED_RETRY");

    assert_eq!(schema_version(&connection, 5).unwrap(), 5);
    assert!(connection.is_autocommit());

    let versions: Vec<i64> = {
        let mut stmt = connection
            .prepare("SELECT version FROM main.schema_migration ORDER BY version")
            .unwrap();
        stmt.query_map([], |r| r.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(versions, [1, 2, 3, 4, 5]);
    assert_eq!(
        connection
            .query_row("PRAGMA main.integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    drop(connection);

    let reopened = open_database(&path).expect("DATABASE_REOPEN_FAILED");
    assert_eq!(schema_version(&reopened, 5).unwrap(), 5);
    assert!(reopened.is_autocommit());

    // Verify schema shape after retry
    let sql: String = reopened
        .query_row(
            "SELECT sql FROM main.sqlite_master WHERE type='table' AND name='movements'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(sql.contains("STRICT"));

    assert_eq!(
        reopened
            .query_row("PRAGMA main.integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
}
