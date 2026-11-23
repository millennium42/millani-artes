use millani_artes::application::create_exit::{
    create_exit as create_exit_use_case, CreateExitError,
};
use millani_artes::domain::Context;
use millani_artes::infrastructure::movements::AdapterSqliteMovementStore;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub(crate) struct ExitSetupState {
    store: Mutex<AdapterSqliteMovementStore>,
    app_data_dir: PathBuf,
}

impl ExitSetupState {
    pub(crate) fn new(store: AdapterSqliteMovementStore, app_data_dir: PathBuf) -> Self {
        Self {
            store: Mutex::new(store),
            app_data_dir,
        }
    }

    fn create_exit(
        &self,
        account_id: [u8; 16],
        amount_cents: i64,
        context: Context,
    ) -> Result<(), &'static str> {
        let mut store = self.store.lock().map_err(|_| "EXIT_CREATE_FAILED")?;
        let mut ids = super::account_setup::SystemUuidGenerator;

        match create_exit_use_case(
            account_id,
            amount_cents,
            context,
            &mut ids,
            &super::account_setup::SystemClock,
            &mut *store,
        ) {
            Ok(_) => Ok(()),
            Err(CreateExitError::InvalidInput) => Err("EXIT_INVALID"),
            Err(CreateExitError::StoreFailed) => {
                if let Ok(recovered) = initialize_exit_store_at(&self.app_data_dir) {
                    *store = recovered;
                }
                Err("EXIT_CREATE_FAILED")
            }
            Err(CreateExitError::IdGenerationFailed | CreateExitError::ClockFailed) => {
                Err("EXIT_CREATE_FAILED")
            }
        }
    }
}

pub(crate) fn initialize_exit_store_at(
    app_data_dir: &Path,
) -> Result<AdapterSqliteMovementStore, super::account_setup::StartupError> {
    super::account_setup::initialize_database_at(app_data_dir).map(AdapterSqliteMovementStore::new)
}

fn parse_account_id(account_id: Vec<u8>) -> Result<[u8; 16], &'static str> {
    account_id.try_into().map_err(|_| "EXIT_INVALID")
}

fn parse_amount_cents(amount_cents: &str) -> Result<i64, &'static str> {
    if amount_cents.is_empty() || !amount_cents.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("EXIT_INVALID");
    }

    let amount = amount_cents.parse::<i64>().map_err(|_| "EXIT_INVALID")?;
    if amount.to_string() != amount_cents {
        return Err("EXIT_INVALID");
    }
    Ok(amount)
}

fn parse_exit_request_body(
    body: &tauri::ipc::InvokeBody,
) -> Result<(Vec<u8>, String, String), &'static str> {
    let tauri::ipc::InvokeBody::Json(value) = body else {
        return Err("EXIT_INVALID");
    };
    let arguments = value.as_object().ok_or("EXIT_INVALID")?;
    if arguments.len() != 3 {
        return Err("EXIT_INVALID");
    }

    let account_id = arguments
        .get("accountId")
        .and_then(|value| value.as_array())
        .ok_or("EXIT_INVALID")?
        .iter()
        .map(|byte| {
            byte.as_u64()
                .and_then(|value| u8::try_from(value).ok())
                .ok_or("EXIT_INVALID")
        })
        .collect::<Result<Vec<_>, _>>()?;
    let amount_cents = arguments
        .get("amountCents")
        .and_then(|value| value.as_str())
        .ok_or("EXIT_INVALID")?
        .to_owned();
    let context = arguments
        .get("context")
        .and_then(|value| value.as_str())
        .ok_or("EXIT_INVALID")?
        .to_owned();

    Ok((account_id, amount_cents, context))
}

pub(crate) fn create_exit_for(
    state: &ExitSetupState,
    write_lock: &super::AppDatabaseWriteLock,
    account_id: Vec<u8>,
    amount_cents: String,
    context: String,
) -> Result<(), &'static str> {
    let account_id = parse_account_id(account_id)?;
    let amount_cents = parse_amount_cents(&amount_cents)?;
    let context = Context::try_from(context.as_str()).map_err(|_| "EXIT_INVALID")?;
    let _write_guard = write_lock.lock().map_err(|_| "EXIT_CREATE_FAILED")?;
    state.create_exit(account_id, amount_cents, context)
}

pub(crate) fn create_exit_from_request_for(
    state: &ExitSetupState,
    write_lock: &super::AppDatabaseWriteLock,
    body: &tauri::ipc::InvokeBody,
) -> Result<(), &'static str> {
    let (account_id, amount_cents, context) = parse_exit_request_body(body)?;
    create_exit_for(state, write_lock, account_id, amount_cents, context)
}

#[tauri::command]
pub(crate) fn create_exit(
    state: tauri::State<'_, ExitSetupState>,
    write_lock: tauri::State<'_, super::AppDatabaseWriteLock>,
    request: tauri::ipc::Request<'_>,
) -> Result<(), String> {
    create_exit_from_request_for(&state, &write_lock, request.body()).map_err(str::to_owned)
}
