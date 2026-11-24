use crate::infrastructure::account_setup::initialize_database_at;
use crate::infrastructure::transfer_setup::{
    create_transfer_for, create_transfer_from_request_for, initialize_transfer_store_at,
    TransferSetupState,
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
            "millani-ui007-transfer-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).expect("UI007_TEMP_CREATE_FAILED");
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

fn setup() -> (TestDirectory, TransferSetupState, [u8; 16], [u8; 16]) {
    let directory = TestDirectory::new();
    let source_id = [0xA7; 16];
    let destination_id = [0xB8; 16];
    let connection = initialize_database_at(directory.path()).expect("UI007_DB_INIT_FAILED");
    for (id, name) in [
        (source_id, "Origem sintética"),
        (destination_id, "Destino sintético"),
    ] {
        connection
            .execute(
                "INSERT INTO main.accounts(id, name, account_type, initial_balance_cents) VALUES(?1, ?2, ?3, ?4)",
                params![id.to_vec(), name, "cash", 0_i64],
            )
            .expect("UI007_ACCOUNT_SEED_FAILED");
    }
    let store =
        initialize_transfer_store_at(directory.path()).expect("UI007_TRANSFER_STORE_INIT_FAILED");
    let state = TransferSetupState::new(store, directory.path().to_path_buf());
    (directory, state, source_id, destination_id)
}

fn body(source_id: &[u8; 16], destination_id: &[u8; 16]) -> tauri::ipc::InvokeBody {
    let mut body = tauri::ipc::InvokeBody::Json(Default::default());
    let tauri::ipc::InvokeBody::Json(arguments) = &mut body else {
        unreachable!("UI007_JSON_BODY_EXPECTED");
    };
    arguments["sourceAccountId"] = source_id.iter().map(|byte| u64::from(*byte)).collect();
    arguments["destinationAccountId"] =
        destination_id.iter().map(|byte| u64::from(*byte)).collect();
    arguments["amountCents"] = "1234".into();
    arguments["context"] = "factory".into();
    body
}

fn counts(path: &Path) -> (i64, i64) {
    let connection = database::open_database(&path.join("millani-artes.sqlite3"))
        .expect("UI007_DB_REOPEN_FAILED");
    let movements = connection
        .query_row("SELECT COUNT(*) FROM main.movements", [], |row| row.get(0))
        .expect("UI007_MOVEMENT_COUNT_FAILED");
    let audits = connection
        .query_row(
            "SELECT COUNT(*) FROM main.audit_event WHERE entity='movement' AND operation='movement_transfer_create'",
            [],
            |row| row.get(0),
        )
        .expect("UI007_AUDIT_COUNT_FAILED");
    (movements, audits)
}

#[test]
fn valid_request_crosses_json_parser_use_case_and_sqlite_commit() {
    let (directory, state, source_id, destination_id) = setup();
    let write_lock = AppDatabaseWriteLock::default();

    create_transfer_from_request_for(&state, &write_lock, &body(&source_id, &destination_id))
        .expect("UI007_TRANSFER_REQUEST_FAILED");

    let connection = database::open_database(&directory.path().join("millani-artes.sqlite3"))
        .expect("UI007_DB_REOPEN_FAILED");
    let row: (String, i64, String, Vec<u8>, Vec<u8>) = connection
        .query_row(
            "SELECT kind, amount_cents, context, source_account_id, destination_account_id FROM main.movements",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        )
        .expect("UI007_TRANSFER_ROW_MISSING");
    assert_eq!(
        row,
        (
            "transfer".to_owned(),
            1234,
            "factory".to_owned(),
            source_id.to_vec(),
            destination_id.to_vec()
        )
    );
    assert_eq!(counts(directory.path()), (1, 1));
}

#[test]
fn malformed_json_and_extra_keys_return_static_error_without_writing() {
    let (directory, state, source_id, destination_id) = setup();
    let write_lock = AppDatabaseWriteLock::default();
    let mut malformed = body(&source_id, &destination_id);
    let tauri::ipc::InvokeBody::Json(arguments) = &mut malformed else {
        unreachable!("UI007_JSON_BODY_EXPECTED");
    };
    arguments["sourceAccountId"] = "SENSITIVE_ACCOUNT_ID".into();

    let error = create_transfer_from_request_for(&state, &write_lock, &malformed)
        .expect_err("UI007_MALFORMED_JSON_MUST_FAIL");
    assert_eq!(error, "TRANSFER_INVALID");
    assert!(!error.contains("SENSITIVE_ACCOUNT_ID"));

    let mut extra = body(&source_id, &destination_id);
    let tauri::ipc::InvokeBody::Json(arguments) = &mut extra else {
        unreachable!("UI007_JSON_BODY_EXPECTED");
    };
    arguments["extra"] = "SENSITIVE_EXTRA".into();
    assert_eq!(
        create_transfer_from_request_for(&state, &write_lock, &extra),
        Err("TRANSFER_INVALID")
    );
    assert_eq!(counts(directory.path()), (0, 0));
}

#[test]
fn invalid_ids_amounts_context_and_domain_rules_return_static_error_without_writing() {
    let (directory, state, source_id, destination_id) = setup();
    let write_lock = AppDatabaseWriteLock::default();
    let mut cases = vec![
        (vec![1; 15], destination_id.to_vec(), "1234", "home"),
        (source_id.to_vec(), destination_id.to_vec(), "01", "home"),
        (source_id.to_vec(), destination_id.to_vec(), "-1", "home"),
        (
            source_id.to_vec(),
            destination_id.to_vec(),
            "9223372036854775808",
            "home",
        ),
        (
            source_id.to_vec(),
            destination_id.to_vec(),
            "1234",
            "invalid",
        ),
        (source_id.to_vec(), source_id.to_vec(), "1234", "home"),
        (source_id.to_vec(), destination_id.to_vec(), "0", "home"),
    ];

    for (source, destination, amount, context) in cases.drain(..) {
        assert_eq!(
            create_transfer_for(
                &state,
                &write_lock,
                source,
                destination,
                amount.to_owned(),
                context.to_owned(),
            ),
            Err("TRANSFER_INVALID")
        );
    }
    assert_eq!(counts(directory.path()), (0, 0));
}

#[test]
fn poisoned_shared_write_lock_returns_static_error_without_writing() {
    let (directory, state, source_id, destination_id) = setup();
    let write_lock = AppDatabaseWriteLock::default();
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = write_lock.lock().expect("UI007_TEST_LOCK_FAILED");
        panic!("UI007_TEST_POISON_LOCK");
    }));

    assert_eq!(
        create_transfer_from_request_for(&state, &write_lock, &body(&source_id, &destination_id)),
        Err("TRANSFER_CREATE_FAILED")
    );
    assert_eq!(counts(directory.path()), (0, 0));
}
