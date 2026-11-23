use crate::infrastructure::account_setup::initialize_database_at;
use crate::infrastructure::exit_setup::{
    create_exit_for, create_exit_from_request_for, initialize_exit_store_at, ExitSetupState,
};
use crate::infrastructure::AppDatabaseWriteLock;
use millani_artes::database;
use rusqlite::params;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let path = std::env::temp_dir().join(format!(
            "millani-ui006-exit-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).expect("UI006_TEMP_CREATE_FAILED");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn setup() -> (TestDirectory, ExitSetupState, [u8; 16]) {
    let directory = TestDirectory::new();
    let account_id = [0xA6; 16];
    let connection = initialize_database_at(directory.path()).expect("UI006_DB_INIT_FAILED");
    connection
        .execute(
            "INSERT INTO main.accounts(id, name, account_type, initial_balance_cents) VALUES(?1, ?2, ?3, ?4)",
            params![account_id.to_vec(), "Conta sintética", "cash", 0_i64],
        )
        .expect("UI006_ACCOUNT_SEED_FAILED");
    let store = initialize_exit_store_at(directory.path()).expect("UI006_EXIT_STORE_INIT_FAILED");
    let state = ExitSetupState::new(store, directory.path().to_path_buf());
    (directory, state, account_id)
}

fn counts(path: &Path) -> (i64, i64) {
    let connection = database::open_database(&path.join("millani-artes.sqlite3"))
        .expect("UI006_DB_REOPEN_FAILED");
    let movements = connection
        .query_row("SELECT COUNT(*) FROM main.movements", [], |row| row.get(0))
        .expect("UI006_MOVEMENT_COUNT_FAILED");
    let audits = connection
        .query_row(
            "SELECT COUNT(*) FROM main.audit_event WHERE entity='movement' AND operation='movement_exit_create'",
            [],
            |row| row.get(0),
        )
        .expect("UI006_AUDIT_COUNT_FAILED");
    (movements, audits)
}

#[test]
fn create_exit_persists_one_exit_and_audit_with_exact_cents_and_context() {
    let (directory, state, account_id) = setup();
    let mut body = tauri::ipc::InvokeBody::Json(Default::default());
    let tauri::ipc::InvokeBody::Json(arguments) = &mut body else {
        unreachable!("UI006_JSON_BODY_EXPECTED");
    };
    arguments["accountId"] = account_id
        .iter()
        .map(|byte| u64::from(*byte))
        .collect::<Vec<_>>()
        .into();
    arguments["amountCents"] = "1234".into();
    arguments["context"] = "factory".into();

    create_exit_from_request_for(&state, &AppDatabaseWriteLock::default(), &body)
        .expect("UI006_EXIT_CREATE_FAILED");

    let connection = database::open_database(&directory.path().join("millani-artes.sqlite3"))
        .expect("UI006_DB_REOPEN_FAILED");
    let row: (String, i64, String, Vec<u8>) = connection
        .query_row(
            "SELECT kind, amount_cents, context, account_id FROM main.movements",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .expect("UI006_EXIT_ROW_MISSING");
    assert_eq!(
        row,
        (
            "exit".to_string(),
            1234,
            "factory".to_string(),
            account_id.to_vec()
        )
    );
    assert_eq!(counts(directory.path()), (1, 1));
}

#[test]
fn malformed_ipc_and_domain_rejection_return_static_errors_without_writes() {
    let (directory, state, account_id) = setup();
    let write_lock = AppDatabaseWriteLock::default();

    for (id, amount, context) in [
        (vec![1; 15], "1234", "home"),
        (account_id.to_vec(), "9223372036854775808", "home"),
        (account_id.to_vec(), "01", "home"),
        (account_id.to_vec(), "12x", "home"),
        (account_id.to_vec(), "1234", "invalid"),
        (account_id.to_vec(), "0", "home"),
    ] {
        assert_eq!(
            create_exit_for(
                &state,
                &write_lock,
                id,
                amount.to_string(),
                context.to_string(),
            ),
            Err("EXIT_INVALID")
        );
    }

    assert_eq!(counts(directory.path()), (0, 0));
}

#[test]
fn malformed_json_types_return_static_ipc_error_without_reflecting_values_or_writing() {
    let (directory, state, _) = setup();
    let write_lock = AppDatabaseWriteLock::default();
    let body = tauri::ipc::InvokeBody::Json(
        [
            ("accountId", "SENSITIVE_ACCOUNT_ID"),
            ("amountCents", "1234"),
            ("context", "home"),
        ]
        .into_iter()
        .collect(),
    );

    let failure = create_exit_from_request_for(&state, &write_lock, &body)
        .expect_err("UI006_MALFORMED_IPC_MUST_FAIL");
    assert_eq!(failure, "EXIT_INVALID");
    assert!(!failure.contains("SENSITIVE_ACCOUNT_ID"));
    assert_eq!(counts(directory.path()), (0, 0));
}

#[test]
fn poisoned_shared_write_lock_returns_static_error_without_writing() {
    let (directory, state, account_id) = setup();
    let write_lock = AppDatabaseWriteLock::default();
    let poisoned = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = write_lock.lock().expect("UI006_WRITE_LOCK_EXPECTED");
        panic!("UI006_POISON_WRITE_LOCK");
    }));
    assert!(poisoned.is_err());

    let failure = create_exit_for(
        &state,
        &write_lock,
        account_id.to_vec(),
        "1234".to_string(),
        "home".to_string(),
    )
    .expect_err("UI006_POISONED_LOCK_MUST_FAIL");
    assert_eq!(failure, "EXIT_CREATE_FAILED");
    assert_eq!(counts(directory.path()), (0, 0));
}

#[test]
fn audit_failure_rolls_back_and_recovers_for_retry_without_leaking_sql() {
    let (directory, state, account_id) = setup();
    let write_lock = AppDatabaseWriteLock::default();
    let connection = database::open_database(&directory.path().join("millani-artes.sqlite3"))
        .expect("UI006_DB_REOPEN_FAILED");
    connection
        .execute_batch(
            "CREATE TRIGGER ui006_fail_exit_audit BEFORE INSERT ON audit_event WHEN NEW.operation='movement_exit_create' BEGIN SELECT RAISE(FAIL, 'sensitive SQL failure'); END;",
        )
        .expect("UI006_TRIGGER_CREATE_FAILED");
    drop(connection);

    let failure = create_exit_for(
        &state,
        &write_lock,
        account_id.to_vec(),
        "1234".to_string(),
        "factory".to_string(),
    )
    .expect_err("UI006_AUDIT_TRIGGER_MUST_FAIL");
    assert_eq!(failure, "EXIT_CREATE_FAILED");
    assert!(!failure.contains("sensitive"));
    assert_eq!(counts(directory.path()), (0, 0));

    let connection = database::open_database(&directory.path().join("millani-artes.sqlite3"))
        .expect("UI006_DB_REOPEN_FAILED");
    connection
        .execute_batch("DROP TRIGGER ui006_fail_exit_audit;")
        .expect("UI006_TRIGGER_DROP_FAILED");
    drop(connection);

    create_exit_for(
        &state,
        &write_lock,
        account_id.to_vec(),
        "1234".to_string(),
        "factory".to_string(),
    )
    .expect("UI006_RETRY_FAILED");
    assert_eq!(counts(directory.path()), (1, 1));
}
