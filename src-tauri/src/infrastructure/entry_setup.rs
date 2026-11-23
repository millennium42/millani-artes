use millani_artes::application::create_entry::{
    create_entry as create_entry_use_case, CreateEntryError,
};
use millani_artes::domain::Context;
use millani_artes::infrastructure::movements::AdapterSqliteMovementStore;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub(crate) struct EntrySetupState {
    store: Mutex<AdapterSqliteMovementStore>,
    app_data_dir: PathBuf,
}

impl EntrySetupState {
    pub(crate) fn new(store: AdapterSqliteMovementStore, app_data_dir: PathBuf) -> Self {
        Self {
            store: Mutex::new(store),
            app_data_dir,
        }
    }

    fn create_entry(
        &self,
        account_id: [u8; 16],
        amount_cents: i64,
        context: Context,
    ) -> Result<(), &'static str> {
        let mut store = self.store.lock().map_err(|_| "ENTRY_CREATE_FAILED")?;
        let mut ids = super::account_setup::SystemUuidGenerator;

        match create_entry_use_case(
            account_id,
            amount_cents,
            context,
            &mut ids,
            &super::account_setup::SystemClock,
            &mut *store,
        ) {
            Ok(_) => Ok(()),
            Err(CreateEntryError::InvalidInput) => Err("ENTRY_INVALID"),
            Err(CreateEntryError::StoreFailed) => {
                if let Ok(recovered) = initialize_entry_store_at(&self.app_data_dir) {
                    *store = recovered;
                }
                Err("ENTRY_CREATE_FAILED")
            }
            Err(CreateEntryError::IdGenerationFailed | CreateEntryError::ClockFailed) => {
                Err("ENTRY_CREATE_FAILED")
            }
        }
    }
}

pub(crate) fn initialize_entry_store_at(
    app_data_dir: &Path,
) -> Result<AdapterSqliteMovementStore, super::account_setup::StartupError> {
    super::account_setup::initialize_database_at(app_data_dir).map(AdapterSqliteMovementStore::new)
}

pub(crate) fn list_accounts_for(
    state: &super::account_setup::AccountSetupState,
) -> Result<Vec<([u8; 16], String)>, &'static str> {
    let accounts = state.read_accounts_for_entry_form()?;
    Ok(accounts
        .into_iter()
        .map(|account| (account.id, account.name))
        .collect())
}

fn parse_account_id(account_id: Vec<u8>) -> Result<[u8; 16], &'static str> {
    account_id.try_into().map_err(|_| "ENTRY_INVALID")
}

fn parse_amount_cents(amount_cents: &str) -> Result<i64, &'static str> {
    if amount_cents.is_empty() || !amount_cents.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("ENTRY_INVALID");
    }

    let amount = amount_cents.parse::<i64>().map_err(|_| "ENTRY_INVALID")?;
    if amount.to_string() != amount_cents {
        return Err("ENTRY_INVALID");
    }
    Ok(amount)
}

pub(crate) fn create_entry_for(
    state: &EntrySetupState,
    write_lock: &super::AppDatabaseWriteLock,
    account_id: Vec<u8>,
    amount_cents: String,
    context: String,
) -> Result<(), &'static str> {
    let account_id = parse_account_id(account_id)?;
    let amount_cents = parse_amount_cents(&amount_cents)?;
    let context = Context::try_from(context.as_str()).map_err(|_| "ENTRY_INVALID")?;
    let _write_guard = write_lock.lock().map_err(|_| "ENTRY_CREATE_FAILED")?;
    state.create_entry(account_id, amount_cents, context)
}

#[tauri::command]
pub(crate) fn list_accounts(
    state: tauri::State<'_, super::account_setup::AccountSetupState>,
) -> Result<Vec<([u8; 16], String)>, String> {
    list_accounts_for(&state).map_err(str::to_owned)
}

#[tauri::command]
pub(crate) fn create_entry(
    state: tauri::State<'_, EntrySetupState>,
    write_lock: tauri::State<'_, super::AppDatabaseWriteLock>,
    account_id: Vec<u8>,
    amount_cents: String,
    context: String,
) -> Result<(), String> {
    create_entry_for(&state, &write_lock, account_id, amount_cents, context).map_err(str::to_owned)
}
