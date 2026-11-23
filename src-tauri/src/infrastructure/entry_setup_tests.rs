use crate::infrastructure::account_setup::{initialize_database_at, AccountSetupState};
use crate::infrastructure::entry_setup::{
    create_entry_for, initialize_entry_store_at, list_accounts_for, EntrySetupState,
};
use crate::infrastructure::AppDatabaseWriteLock;
use millani_artes::database;
use millani_artes::infrastructure::accounts::AdapterSqliteAccountStore;
use rusqlite::params;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let path = std::env::temp_dir().join(format!(
            "millani-ui005-entry-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).expect("UI005_TEMP_CREATE_FAILED");
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

fn setup() -> (TestDirectory, AccountSetupState, EntrySetupState, [u8; 16]) {
    let directory = TestDirectory::new();
    let account_id = [0xA1; 16];
    let connection = initialize_database_at(directory.path()).expect("UI005_DB_INIT_FAILED");
    connection
        .execute(
            "INSERT INTO main.accounts(id, name, account_type, initial_balance_cents) VALUES(?1, ?2, ?3, ?4)",
            params![account_id.to_vec(), "Conta sintética", "cash", 0_i64],
        )
        .expect("UI005_ACCOUNT_SEED_FAILED");
    let account_state = AccountSetupState::new(
        AdapterSqliteAccountStore::new(connection),
        directory.path().to_path_buf(),
    );
    let movement_store =
        initialize_entry_store_at(directory.path()).expect("UI005_ENTRY_STORE_INIT_FAILED");
    let entry_state = EntrySetupState::new(movement_store, directory.path().to_path_buf());
    (directory, account_state, entry_state, account_id)
}

fn counts(path: &Path) -> (i64, i64) {
    let connection = database::open_database(&path.join("millani-artes.sqlite3"))
        .expect("UI005_DB_REOPEN_FAILED");
    let movements = connection
        .query_row("SELECT COUNT(*) FROM main.movements", [], |row| row.get(0))
        .expect("UI005_MOVEMENT_COUNT_FAILED");
    let audits = connection
        .query_row("SELECT COUNT(*) FROM main.audit_event", [], |row| {
            row.get(0)
        })
        .expect("UI005_AUDIT_COUNT_FAILED");
    (movements, audits)
}

#[test]
fn list_accounts_returns_only_id_and_name_and_masks_read_error() {
    let (_directory, account_state, _entry_state, account_id) = setup();
    assert_eq!(
        list_accounts_for(&account_state).expect("UI005_LIST_ACCOUNTS_FAILED"),
        vec![(account_id, "Conta sintética".to_string())]
    );

    let invalid_store = AdapterSqliteAccountStore::new(
        rusqlite::Connection::open_in_memory().expect("UI005_MEMORY_DB_FAILED"),
    );
    let invalid_state = AccountSetupState::new(invalid_store, std::env::temp_dir());
    assert_eq!(
        list_accounts_for(&invalid_state),
        Err("ACCOUNT_LIST_FAILED")
    );
}

#[test]
fn create_entry_uses_the_shared_write_lock_and_persists_entry_with_audit() {
    let (directory, account_state, entry_state, account_id) = setup();
    assert_eq!(
        list_accounts_for(&account_state).expect("UI005_LIST_ACCOUNTS_FAILED")[0].0,
        account_id
    );
    create_entry_for(
        &entry_state,
        &AppDatabaseWriteLock::default(),
        account_id.to_vec(),
        "1234".to_string(),
        "home".to_string(),
    )
    .expect("UI005_ENTRY_CREATE_FAILED");

    assert_eq!(counts(directory.path()), (1, 1));
}

#[test]
fn malformed_ipc_and_domain_rejection_return_static_errors_without_writes() {
    let (directory, _account_state, entry_state, account_id) = setup();
    let write_lock = AppDatabaseWriteLock::default();

    assert_eq!(
        create_entry_for(
            &entry_state,
            &write_lock,
            vec![1; 15],
            "1234".to_string(),
            "home".to_string(),
        ),
        Err("ENTRY_INVALID")
    );
    assert_eq!(
        create_entry_for(
            &entry_state,
            &write_lock,
            account_id.to_vec(),
            "9223372036854775808".to_string(),
            "home".to_string(),
        ),
        Err("ENTRY_INVALID")
    );
    assert_eq!(
        create_entry_for(
            &entry_state,
            &write_lock,
            account_id.to_vec(),
            "0".to_string(),
            "home".to_string(),
        ),
        Err("ENTRY_INVALID")
    );
    assert_eq!(
        create_entry_for(
            &entry_state,
            &write_lock,
            account_id.to_vec(),
            "1234".to_string(),
            "invalid".to_string(),
        ),
        Err("ENTRY_INVALID")
    );
    assert_eq!(counts(directory.path()), (0, 0));
}

#[test]
fn failed_write_rolls_back_and_recovers_for_retry_without_leaking_sql() {
    let (directory, _account_state, entry_state, account_id) = setup();
    let write_lock = AppDatabaseWriteLock::default();
    let connection = database::open_database(&directory.path().join("millani-artes.sqlite3"))
        .expect("UI005_DB_REOPEN_FAILED");
    connection
        .execute_batch(
            "CREATE TRIGGER ui005_fail_entry BEFORE INSERT ON movements BEGIN SELECT RAISE(FAIL, 'sensitive SQL failure'); END;",
        )
        .expect("UI005_TRIGGER_CREATE_FAILED");

    let failure = create_entry_for(
        &entry_state,
        &write_lock,
        account_id.to_vec(),
        "1234".to_string(),
        "factory".to_string(),
    )
    .expect_err("UI005_TRIGGER_MUST_FAIL");
    assert_eq!(failure, "ENTRY_CREATE_FAILED");
    assert!(!failure.contains("sensitive"));
    assert_eq!(counts(directory.path()), (0, 0));

    let connection = database::open_database(&directory.path().join("millani-artes.sqlite3"))
        .expect("UI005_DB_REOPEN_FAILED");
    connection
        .execute_batch("DROP TRIGGER ui005_fail_entry;")
        .expect("UI005_TRIGGER_DROP_FAILED");
    create_entry_for(
        &entry_state,
        &write_lock,
        account_id.to_vec(),
        "1234".to_string(),
        "factory".to_string(),
    )
    .expect("UI005_RETRY_FAILED");
    assert_eq!(counts(directory.path()), (1, 1));
}
