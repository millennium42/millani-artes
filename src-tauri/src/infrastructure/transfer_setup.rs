use millani_artes::application::create_transfer::{
    create_transfer as create_transfer_use_case, CreateTransferError,
};
use millani_artes::domain::Context;
use millani_artes::infrastructure::movements::AdapterSqliteMovementStore;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub(crate) struct TransferSetupState {
    store: Mutex<AdapterSqliteMovementStore>,
    app_data_dir: PathBuf,
}

impl TransferSetupState {
    pub(crate) fn new(store: AdapterSqliteMovementStore, app_data_dir: PathBuf) -> Self {
        Self {
            store: Mutex::new(store),
            app_data_dir,
        }
    }

    fn create_transfer(
        &self,
        source_account_id: [u8; 16],
        destination_account_id: [u8; 16],
        amount_cents: i64,
        context: Context,
    ) -> Result<(), &'static str> {
        let mut store = self.store.lock().map_err(|_| "TRANSFER_CREATE_FAILED")?;
        let mut ids = super::account_setup::SystemUuidGenerator;

        match create_transfer_use_case(
            source_account_id,
            destination_account_id,
            amount_cents,
            context,
            &mut ids,
            &super::account_setup::SystemClock,
            &mut *store,
        ) {
            Ok(_) => Ok(()),
            Err(CreateTransferError::InvalidInput) => Err("TRANSFER_INVALID"),
            Err(CreateTransferError::StoreFailed) => {
                if let Ok(recovered) = initialize_transfer_store_at(&self.app_data_dir) {
                    *store = recovered;
                }
                Err("TRANSFER_CREATE_FAILED")
            }
            Err(CreateTransferError::IdGenerationFailed | CreateTransferError::ClockFailed) => {
                Err("TRANSFER_CREATE_FAILED")
            }
        }
    }
}

pub(crate) fn initialize_transfer_store_at(
    app_data_dir: &Path,
) -> Result<AdapterSqliteMovementStore, super::account_setup::StartupError> {
    super::account_setup::initialize_database_at(app_data_dir).map(AdapterSqliteMovementStore::new)
}

fn parse_account_id(account_id: Vec<u8>) -> Result<[u8; 16], &'static str> {
    account_id.try_into().map_err(|_| "TRANSFER_INVALID")
}

fn parse_amount_cents(amount_cents: &str) -> Result<i64, &'static str> {
    if amount_cents.is_empty() || !amount_cents.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("TRANSFER_INVALID");
    }

    let amount = amount_cents
        .parse::<i64>()
        .map_err(|_| "TRANSFER_INVALID")?;
    if amount.to_string() != amount_cents {
        return Err("TRANSFER_INVALID");
    }
    Ok(amount)
}

struct TransferRequestBody {
    source_account_id: Vec<u8>,
    destination_account_id: Vec<u8>,
    amount_cents: String,
    context: String,
}

fn parse_transfer_request_body(
    body: &tauri::ipc::InvokeBody,
) -> Result<TransferRequestBody, &'static str> {
    let tauri::ipc::InvokeBody::Json(value) = body else {
        return Err("TRANSFER_INVALID");
    };
    let arguments = value.as_object().ok_or("TRANSFER_INVALID")?;
    if arguments.len() != 4 {
        return Err("TRANSFER_INVALID");
    }

    let parse_id = |key: &str| {
        arguments
            .get(key)
            .and_then(|value| value.as_array())
            .ok_or("TRANSFER_INVALID")?
            .iter()
            .map(|byte| {
                byte.as_u64()
                    .and_then(|value| u8::try_from(value).ok())
                    .ok_or("TRANSFER_INVALID")
            })
            .collect::<Result<Vec<_>, _>>()
    };

    let source_account_id = parse_id("sourceAccountId")?;
    let destination_account_id = parse_id("destinationAccountId")?;
    let amount_cents = arguments
        .get("amountCents")
        .and_then(|value| value.as_str())
        .ok_or("TRANSFER_INVALID")?
        .to_owned();
    let context = arguments
        .get("context")
        .and_then(|value| value.as_str())
        .ok_or("TRANSFER_INVALID")?
        .to_owned();

    Ok(TransferRequestBody {
        source_account_id,
        destination_account_id,
        amount_cents,
        context,
    })
}

pub(crate) fn create_transfer_for(
    state: &TransferSetupState,
    write_lock: &super::AppDatabaseWriteLock,
    source_account_id: Vec<u8>,
    destination_account_id: Vec<u8>,
    amount_cents: String,
    context: String,
) -> Result<(), &'static str> {
    let source_account_id = parse_account_id(source_account_id)?;
    let destination_account_id = parse_account_id(destination_account_id)?;
    let amount_cents = parse_amount_cents(&amount_cents)?;
    let context = Context::try_from(context.as_str()).map_err(|_| "TRANSFER_INVALID")?;
    let _write_guard = write_lock.lock().map_err(|_| "TRANSFER_CREATE_FAILED")?;
    state.create_transfer(
        source_account_id,
        destination_account_id,
        amount_cents,
        context,
    )
}

pub(crate) fn create_transfer_from_request_for(
    state: &TransferSetupState,
    write_lock: &super::AppDatabaseWriteLock,
    body: &tauri::ipc::InvokeBody,
) -> Result<(), &'static str> {
    let request = parse_transfer_request_body(body)?;
    create_transfer_for(
        state,
        write_lock,
        request.source_account_id,
        request.destination_account_id,
        request.amount_cents,
        request.context,
    )
}

#[tauri::command]
pub(crate) fn create_transfer(
    state: tauri::State<'_, TransferSetupState>,
    write_lock: tauri::State<'_, super::AppDatabaseWriteLock>,
    request: tauri::ipc::Request<'_>,
) -> Result<(), String> {
    create_transfer_from_request_for(&state, &write_lock, request.body()).map_err(str::to_owned)
}
