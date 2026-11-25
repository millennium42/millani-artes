use millani_artes::application::create_product::{
    create_product as create_product_use_case, CreateProductError,
};
use millani_artes::infrastructure::products::AdapterSqliteProductStore;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub(crate) struct ProductSetupState {
    store: Mutex<AdapterSqliteProductStore>,
    app_data_dir: PathBuf,
}

impl ProductSetupState {
    pub(crate) fn new(store: AdapterSqliteProductStore, app_data_dir: PathBuf) -> Self {
        Self {
            store: Mutex::new(store),
            app_data_dir,
        }
    }

    fn create_product(
        &self,
        name: String,
        description: String,
        suggested_price_cents: i64,
    ) -> Result<(), &'static str> {
        let mut store = self.store.lock().map_err(|_| "PRODUCT_CREATE_FAILED")?;
        let mut ids = super::account_setup::SystemUuidGenerator;

        match create_product_use_case(
            name,
            description,
            suggested_price_cents,
            &mut ids,
            &super::account_setup::SystemClock,
            &mut *store,
        ) {
            Ok(_) => Ok(()),
            Err(CreateProductError::InvalidName) => Err("PRODUCT_NAME_INVALID"),
            Err(CreateProductError::InvalidSuggestedPrice) => Err("PRODUCT_PRICE_INVALID"),
            Err(CreateProductError::StoreFailed) => {
                if let Ok(recovered) = initialize_product_store_at(&self.app_data_dir) {
                    *store = recovered;
                }
                Err("PRODUCT_CREATE_FAILED")
            }
            Err(CreateProductError::IdGenerationFailed | CreateProductError::ClockFailed) => {
                Err("PRODUCT_CREATE_FAILED")
            }
        }
    }
}

pub(crate) fn initialize_product_store_at(
    app_data_dir: &Path,
) -> Result<AdapterSqliteProductStore, super::account_setup::StartupError> {
    super::account_setup::initialize_database_at(app_data_dir).map(AdapterSqliteProductStore::new)
}

struct ProductRequestBody {
    name: String,
    description: String,
    suggested_price_cents: i64,
}

fn parse_suggested_price_cents(value: &str) -> Result<i64, &'static str> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("PRODUCT_PRICE_INVALID");
    }

    let cents = value.parse::<i64>().map_err(|_| "PRODUCT_PRICE_INVALID")?;
    if cents.to_string() != value {
        return Err("PRODUCT_PRICE_INVALID");
    }
    Ok(cents)
}

fn parse_product_request_body(
    body: &tauri::ipc::InvokeBody,
) -> Result<ProductRequestBody, &'static str> {
    let tauri::ipc::InvokeBody::Json(value) = body else {
        return Err("PRODUCT_INVALID");
    };
    let arguments = value.as_object().ok_or("PRODUCT_INVALID")?;
    if arguments.len() != 3 {
        return Err("PRODUCT_INVALID");
    }

    let name = arguments
        .get("name")
        .and_then(|value| value.as_str())
        .ok_or("PRODUCT_INVALID")?
        .to_owned();
    let description = arguments
        .get("description")
        .and_then(|value| value.as_str())
        .ok_or("PRODUCT_INVALID")?
        .to_owned();
    let suggested_price_cents = arguments
        .get("suggestedPriceCents")
        .and_then(|value| value.as_str())
        .ok_or("PRODUCT_INVALID")
        .and_then(parse_suggested_price_cents)?;

    Ok(ProductRequestBody {
        name,
        description,
        suggested_price_cents,
    })
}

pub(crate) fn create_product_from_request_for(
    state: &ProductSetupState,
    write_lock: &super::AppDatabaseWriteLock,
    body: &tauri::ipc::InvokeBody,
) -> Result<(), &'static str> {
    let request = parse_product_request_body(body)?;
    let _write_guard = write_lock.lock().map_err(|_| "PRODUCT_CREATE_FAILED")?;
    state.create_product(
        request.name,
        request.description,
        request.suggested_price_cents,
    )
}

#[tauri::command]
pub(crate) fn create_product(
    state: tauri::State<'_, ProductSetupState>,
    write_lock: tauri::State<'_, super::AppDatabaseWriteLock>,
    request: tauri::ipc::Request<'_>,
) -> Result<(), String> {
    create_product_from_request_for(&state, &write_lock, request.body()).map_err(str::to_owned)
}

#[cfg(test)]
#[path = "product_setup_tests.rs"]
mod tests;
