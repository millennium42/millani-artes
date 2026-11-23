use millani_artes::application::create_account::{self, CreateAccountError};
use millani_artes::application::list_accounts::{list_accounts, ListAccountsError};
use millani_artes::database;
use millani_artes::domain::ports::{Clock, UuidGenerator};
use millani_artes::domain::AccountType;
use millani_artes::infrastructure::accounts::AdapterSqliteAccountStore;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

const DATABASE_FILE_NAME: &str = "millani-artes.sqlite3";
const LATEST_SCHEMA_VERSION: u32 = 17;

pub(crate) struct AccountSetupState {
    store: Mutex<AdapterSqliteAccountStore>,
    app_data_dir: PathBuf,
}

impl AccountSetupState {
    pub(crate) fn new(store: AdapterSqliteAccountStore, app_data_dir: PathBuf) -> Self {
        Self {
            store: Mutex::new(store),
            app_data_dir,
        }
    }

    pub(crate) fn read_accounts_for_entry_form(
        &self,
    ) -> Result<Vec<millani_artes::domain::Account>, &'static str> {
        let store = self.store.lock().map_err(|_| "ACCOUNT_LIST_FAILED")?;
        list_accounts(&*store).map_err(|_: ListAccountsError| "ACCOUNT_LIST_FAILED")
    }

    fn create_account(
        &self,
        name: String,
        account_type: AccountType,
        initial_balance_cents: i64,
    ) -> Result<(), &'static str> {
        let mut store = self.store.lock().map_err(|_| "ACCOUNT_CREATE_FAILED")?;
        let mut ids = SystemUuidGenerator;

        match create_account::create_account(
            name,
            account_type,
            initial_balance_cents,
            &mut ids,
            &SystemClock,
            &mut *store,
        ) {
            Ok(_) => Ok(()),
            Err(CreateAccountError::InvalidName) => Err("ACCOUNT_NAME_INVALID"),
            Err(CreateAccountError::StoreFailed) => {
                if let Ok(recovered) = initialize_account_store_at(&self.app_data_dir) {
                    *store = recovered;
                }
                Err("ACCOUNT_CREATE_FAILED")
            }
            Err(CreateAccountError::IdGenerationFailed | CreateAccountError::ClockFailed) => {
                Err("ACCOUNT_CREATE_FAILED")
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct StartupError;

impl fmt::Display for StartupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("APP_DATABASE_INIT_FAILED")
    }
}

impl Error for StartupError {}

pub(crate) fn initialize_account_store_at(
    app_data_dir: &Path,
) -> Result<AdapterSqliteAccountStore, StartupError> {
    initialize_database_at(app_data_dir).map(AdapterSqliteAccountStore::new)
}

pub(crate) fn initialize_database_at(
    app_data_dir: &Path,
) -> Result<rusqlite::Connection, StartupError> {
    if !app_data_dir.is_absolute() {
        return Err(StartupError);
    }

    fs::create_dir_all(app_data_dir).map_err(|_| StartupError)?;
    let connection = database::open_database(&app_data_dir.join(DATABASE_FILE_NAME))
        .map_err(|_| StartupError)?;
    migrate_to_latest(connection)
}

fn migrate_to_latest(
    mut connection: rusqlite::Connection,
) -> Result<rusqlite::Connection, StartupError> {
    let mut current =
        database::schema_version(&connection, LATEST_SCHEMA_VERSION).map_err(|_| StartupError)?;

    while current < LATEST_SCHEMA_VERSION {
        let next = current.checked_add(1).ok_or(StartupError)?;
        connection = match next {
            1 => database::execute_versioned_migration(connection, 1, ""),
            2 => database::execute_versioned_migration(
                connection,
                2,
                database::AUDIT_EVENT_MIGRATION_SQL,
            ),
            3 => database::execute_versioned_migration(
                connection,
                3,
                database::ACCOUNTS_MIGRATION_SQL,
            ),
            4 => database::execute_versioned_migration(
                connection,
                4,
                database::CATEGORIES_MIGRATION_SQL,
            ),
            5 => database::execute_versioned_migration(
                connection,
                5,
                database::MOVEMENTS_MIGRATION_SQL,
            ),
            6 => database::execute_versioned_migration(
                connection,
                6,
                database::MOVEMENT_CATEGORIES_MIGRATION_SQL,
            ),
            7 => database::execute_versioned_migration(
                connection,
                7,
                database::RECURRENCES_MIGRATION_SQL,
            ),
            8 => database::execute_versioned_migration(
                connection,
                8,
                database::PRODUCTS_MIGRATION_SQL,
            ),
            9 => database::execute_versioned_migration(
                connection,
                9,
                database::CUSTOMERS_MIGRATION_SQL,
            ),
            10 => {
                database::execute_versioned_migration(connection, 10, database::SALES_MIGRATION_SQL)
            }
            11 => database::execute_versioned_migration(
                connection,
                11,
                database::SALE_ITEMS_MIGRATION_SQL,
            ),
            12 => database::execute_versioned_migration(
                connection,
                12,
                database::RECEIVABLES_MIGRATION_SQL,
            ),
            13 => database::execute_versioned_migration(
                connection,
                13,
                database::RECEIPTS_MIGRATION_SQL,
            ),
            14 => database::execute_versioned_migration(
                connection,
                14,
                database::SALE_RETURN_ITEMS_MIGRATION_SQL,
            ),
            15 => database::execute_sale_return_financials_migration(connection),
            16 => database::execute_sale_return_refunds_migration(connection),
            17 => database::execute_sale_cancellations_migration(connection),
            _ => return Err(StartupError),
        }
        .map_err(|_| StartupError)?;
        current = next;
    }

    Ok(connection)
}

fn account_type_from_key(value: &str) -> Result<AccountType, &'static str> {
    match value {
        "cash" => Ok(AccountType::Cash),
        "bank_pix" => Ok(AccountType::BankPix),
        "card" => Ok(AccountType::Card),
        "other" => Ok(AccountType::Other),
        _ => Err("ACCOUNT_TYPE_INVALID"),
    }
}

fn initial_balance_cents_from_wire(value: &str) -> Result<i64, &'static str> {
    let cents = value
        .parse::<i64>()
        .map_err(|_| "INITIAL_BALANCE_INVALID")?;
    if cents.to_string() != value {
        return Err("INITIAL_BALANCE_INVALID");
    }
    Ok(cents)
}

#[derive(Clone, Copy)]
pub(crate) struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }
}

#[derive(Clone, Copy)]
pub(crate) struct SystemUuidGenerator;

impl UuidGenerator for SystemUuidGenerator {
    type Error = getrandom::Error;

    fn generate(&mut self) -> Result<[u8; 16], Self::Error> {
        let mut bytes = [0_u8; 16];
        getrandom::fill(&mut bytes)?;
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        Ok(bytes)
    }
}

#[tauri::command]
pub(crate) fn create_account(
    state: tauri::State<'_, AccountSetupState>,
    write_lock: tauri::State<'_, super::AppDatabaseWriteLock>,
    name: String,
    account_type: String,
    initial_balance_cents: String,
) -> Result<(), String> {
    let _write_guard = write_lock
        .lock()
        .map_err(|_| "ACCOUNT_CREATE_FAILED".to_owned())?;
    let account_type = account_type_from_key(&account_type).map_err(str::to_owned)?;
    let initial_balance_cents =
        initial_balance_cents_from_wire(&initial_balance_cents).map_err(str::to_owned)?;
    state
        .create_account(name, account_type, initial_balance_cents)
        .map_err(str::to_owned)
}

#[cfg(test)]
#[path = "account_setup_tests.rs"]
mod tests;
