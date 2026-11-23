use super::{
    account_type_from_key, initial_balance_cents_from_wire, initialize_account_store_at,
    AccountSetupState,
};
use millani_artes::application::create_account::create_account;
use millani_artes::application::list_accounts::AccountReadStore;
use millani_artes::database::{self, execute_versioned_migration, schema_version};
use millani_artes::domain::ports::{Clock, UuidGenerator};
use millani_artes::domain::AccountType;
use millani_artes::infrastructure::accounts::AdapterSqliteAccountStore;
use std::convert::Infallible;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(1);

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "millani-ui003-{}-{}",
            std::process::id(),
            NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        ));
        assert!(!path.exists(), "UI003_TEST_DIRECTORY_ALREADY_EXISTS");
        std::fs::create_dir(&path).expect("UI003_TEST_DIRECTORY_CREATE_FAILED");
        Self(path)
    }

    fn data_path(&self) -> PathBuf {
        self.0.join("app-data")
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).expect("UI003_TEST_DIRECTORY_CLEANUP_FAILED");
    }
}

#[test]
fn initializes_a_new_database_through_version_17_and_reopens_it() {
    let temp = TestDirectory::new();
    let data_path = temp.data_path();

    let store = initialize_account_store_at(&data_path).expect("UI003_STARTUP_FAILED");
    drop(store);

    let connection = database::open_database(&data_path.join("millani-artes.sqlite3"))
        .expect("UI003_DATABASE_REOPEN_FAILED");
    assert_eq!(
        schema_version(&connection, 17).expect("UI003_SCHEMA_VERSION_FAILED"),
        17
    );
    drop(connection);

    let reopened = initialize_account_store_at(&data_path).expect("UI003_REOPEN_STARTUP_FAILED");
    drop(reopened);
}

#[test]
fn upgrades_a_partial_database_without_reapplying_applied_migrations() {
    let temp = TestDirectory::new();
    let data_path = temp.data_path();
    std::fs::create_dir(&data_path).expect("UI003_DATA_DIRECTORY_CREATE_FAILED");
    let database_path = data_path.join("millani-artes.sqlite3");
    let connection = database::open_database(&database_path).expect("UI003_DATABASE_OPEN_FAILED");
    let connection =
        execute_versioned_migration(connection, 1, "").expect("UI003_V1_MIGRATION_FAILED");
    let connection =
        execute_versioned_migration(connection, 2, database::AUDIT_EVENT_MIGRATION_SQL)
            .expect("UI003_V2_MIGRATION_FAILED");
    drop(connection);

    let store = initialize_account_store_at(&data_path).expect("UI003_PARTIAL_UPGRADE_FAILED");
    drop(store);
    let connection = database::open_database(&database_path).expect("UI003_UPGRADED_REOPEN_FAILED");
    assert_eq!(
        schema_version(&connection, 17).expect("UI003_SCHEMA_VERSION_FAILED"),
        17
    );
}

#[test]
fn rejects_a_future_migration_ledger_with_static_error() {
    let temp = TestDirectory::new();
    let data_path = temp.data_path();
    std::fs::create_dir(&data_path).expect("UI003_DATA_DIRECTORY_CREATE_FAILED");
    let database_path = data_path.join("millani-artes.sqlite3");
    let connection = database::open_database(&database_path).expect("UI003_DATABASE_OPEN_FAILED");
    connection
        .execute_batch(
            "CREATE TABLE main.schema_migration (version INTEGER PRIMARY KEY CHECK (version > 0) NOT NULL);
             INSERT INTO main.schema_migration(version) VALUES (18);",
        )
        .expect("UI003_INVALID_LEDGER_FIXTURE_FAILED");
    drop(connection);

    let result = initialize_account_store_at(&data_path);
    assert!(result.is_err());
    let error = result.err().expect("UI003_INVALID_LEDGER_ACCEPTED");
    let text = error.to_string();
    assert!(!text.contains(&data_path.display().to_string()));
    assert!(!text.contains("schema_migration"));
}

#[test]
fn rejects_a_malformed_migration_ledger_with_static_error() {
    let temp = TestDirectory::new();
    let data_path = temp.data_path();
    std::fs::create_dir(&data_path).expect("UI003_DATA_DIRECTORY_CREATE_FAILED");
    let database_path = data_path.join("millani-artes.sqlite3");
    let connection = database::open_database(&database_path).expect("UI003_DATABASE_OPEN_FAILED");
    connection
        .execute_batch("CREATE TABLE main.schema_migration (not_version INTEGER PRIMARY KEY);")
        .expect("UI003_INVALID_LEDGER_FIXTURE_FAILED");
    drop(connection);

    let error = match initialize_account_store_at(&data_path) {
        Err(error) => error.to_string(),
        Ok(_) => panic!("UI003_MALFORMED_LEDGER_ACCEPTED"),
    };
    assert_eq!(error, "APP_DATABASE_INIT_FAILED");
    assert!(!error.contains(&data_path.display().to_string()));
    assert!(!error.contains("schema_migration"));
}

#[test]
fn rejects_relative_app_data_paths_without_creating_files() {
    let result = initialize_account_store_at(Path::new("ui003-relative-app-data"));
    assert!(result.is_err());
}

#[test]
fn creates_the_account_and_audit_using_real_sqlite_without_a_movement() {
    let temp = TestDirectory::new();
    let data_path = temp.data_path();
    let mut store: AdapterSqliteAccountStore =
        initialize_account_store_at(&data_path).expect("UI003_STARTUP_FAILED");
    let mut ids = TestIds(0);
    let account = create_account(
        " Banco 🪴 ".to_owned(),
        AccountType::BankPix,
        i64::MIN,
        &mut ids,
        &FixedClock,
        &mut store,
    )
    .expect("UI003_ACCOUNT_CREATE_FAILED");

    let accounts = store
        .read_all_accounts()
        .expect("UI003_ACCOUNT_READ_FAILED");
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].id, account.id);
    assert_eq!(accounts[0].name, " Banco 🪴 ");
    assert!(matches!(accounts[0].account_type, AccountType::BankPix));
    assert_eq!(accounts[0].initial_balance_cents, i64::MIN);

    let connection = database::open_database(&data_path.join("millani-artes.sqlite3"))
        .expect("UI003_DATABASE_REOPEN_FAILED");
    let event_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM main.audit_event", [], |row| {
            row.get(0)
        })
        .expect("UI003_AUDIT_COUNT_FAILED");
    let movement_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM main.movements", [], |row| row.get(0))
        .expect("UI003_MOVEMENT_COUNT_FAILED");
    assert_eq!(event_count, 1);
    assert_eq!(movement_count, 0);
    drop(connection);
    drop(store);

    let reopened = initialize_account_store_at(&data_path).expect("UI003_REOPEN_STARTUP_FAILED");
    let accounts = reopened
        .read_all_accounts()
        .expect("UI003_REOPEN_ACCOUNT_READ_FAILED");
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].initial_balance_cents, i64::MIN);
}

#[test]
fn failed_account_write_rolls_back_and_recovers_connection_for_a_later_attempt() {
    let temp = TestDirectory::new();
    let data_path = temp.data_path();
    let store = initialize_account_store_at(&data_path).expect("UI003_STARTUP_FAILED");
    let state = AccountSetupState::new(store, data_path.clone());
    let database_path = data_path.join("millani-artes.sqlite3");

    let connection = database::open_database(&database_path).expect("UI003_DATABASE_OPEN_FAILED");
    connection
        .execute_batch(
            "CREATE TRIGGER main.reject_ui003_audit
             BEFORE INSERT ON audit_event
             BEGIN SELECT RAISE(ABORT, 'test failure'); END;",
        )
        .expect("UI003_TRIGGER_CREATE_FAILED");
    drop(connection);

    assert_eq!(
        state.create_account("Caixa".to_owned(), AccountType::Cash, 12_345),
        Err("ACCOUNT_CREATE_FAILED")
    );

    let connection = database::open_database(&database_path).expect("UI003_DATABASE_REOPEN_FAILED");
    let account_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM main.accounts", [], |row| row.get(0))
        .expect("UI003_ACCOUNT_COUNT_FAILED");
    let event_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM main.audit_event", [], |row| {
            row.get(0)
        })
        .expect("UI003_AUDIT_COUNT_FAILED");
    assert_eq!(account_count, 0);
    assert_eq!(event_count, 0);
    connection
        .execute_batch("DROP TRIGGER main.reject_ui003_audit;")
        .expect("UI003_TRIGGER_DROP_FAILED");
    drop(connection);

    state
        .create_account("Caixa".to_owned(), AccountType::Cash, 12_345)
        .expect("UI003_RETRY_FAILED");

    let connection = database::open_database(&database_path).expect("UI003_DATABASE_REOPEN_FAILED");
    let account_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM main.accounts", [], |row| row.get(0))
        .expect("UI003_ACCOUNT_COUNT_FAILED");
    let event_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM main.audit_event", [], |row| {
            row.get(0)
        })
        .expect("UI003_AUDIT_COUNT_FAILED");
    assert_eq!(account_count, 1);
    assert_eq!(event_count, 1);
}

#[test]
fn maps_only_the_four_existing_account_types() {
    assert!(matches!(
        account_type_from_key("cash").unwrap(),
        AccountType::Cash
    ));
    assert!(matches!(
        account_type_from_key("bank_pix").unwrap(),
        AccountType::BankPix
    ));
    assert!(matches!(
        account_type_from_key("card").unwrap(),
        AccountType::Card
    ));
    assert!(matches!(
        account_type_from_key("other").unwrap(),
        AccountType::Other
    ));
    assert!(account_type_from_key("context").is_err());
}

#[test]
fn parses_wire_cents_exactly_as_i64_and_rejects_noncanonical_or_overflow_text() {
    assert_eq!(
        initial_balance_cents_from_wire("-9223372036854775808").unwrap(),
        i64::MIN
    );
    assert_eq!(
        initial_balance_cents_from_wire("9223372036854775807").unwrap(),
        i64::MAX
    );
    for invalid in [
        "",
        "+1",
        "01",
        "-0",
        "1.00",
        "9223372036854775808",
        "-9223372036854775809",
    ] {
        assert!(
            initial_balance_cents_from_wire(invalid).is_err(),
            "{invalid}"
        );
    }
}

struct TestIds(u8);

impl UuidGenerator for TestIds {
    type Error = Infallible;

    fn generate(&mut self) -> Result<[u8; 16], Self::Error> {
        self.0 += 1;
        Ok([self.0; 16])
    }
}

struct FixedClock;

impl Clock for FixedClock {
    fn now(&self) -> SystemTime {
        UNIX_EPOCH + Duration::from_millis(1_234)
    }
}
