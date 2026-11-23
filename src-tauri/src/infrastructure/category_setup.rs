use millani_artes::application::create_category::{self, CreateCategoryError};
use millani_artes::infrastructure::categories::AdapterSqliteCategoryStore;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub(crate) struct CategorySetupState {
    store: Mutex<AdapterSqliteCategoryStore>,
    app_data_dir: PathBuf,
}

impl CategorySetupState {
    pub(crate) fn new(store: AdapterSqliteCategoryStore, app_data_dir: PathBuf) -> Self {
        Self {
            store: Mutex::new(store),
            app_data_dir,
        }
    }

    fn create_category(&self, name: String) -> Result<(), &'static str> {
        let mut store = self.store.lock().map_err(|_| "CATEGORY_CREATE_FAILED")?;
        let mut ids = super::account_setup::SystemUuidGenerator;

        match create_category::create_category(
            name,
            &mut ids,
            &super::account_setup::SystemClock,
            &mut *store,
        ) {
            Ok(_) => Ok(()),
            Err(CreateCategoryError::InvalidName) => Err("CATEGORY_NAME_INVALID"),
            Err(CreateCategoryError::StoreFailed) => {
                if let Ok(connection) =
                    super::account_setup::initialize_database_at(&self.app_data_dir)
                {
                    *store = AdapterSqliteCategoryStore::new(connection);
                }
                Err("CATEGORY_CREATE_FAILED")
            }
            Err(CreateCategoryError::IdGenerationFailed | CreateCategoryError::ClockFailed) => {
                Err("CATEGORY_CREATE_FAILED")
            }
        }
    }
}

pub(crate) fn initialize_category_store_at(
    app_data_dir: &Path,
) -> Result<AdapterSqliteCategoryStore, super::account_setup::StartupError> {
    super::account_setup::initialize_database_at(app_data_dir).map(AdapterSqliteCategoryStore::new)
}

#[tauri::command]
pub(crate) fn create_category(
    state: tauri::State<'_, CategorySetupState>,
    write_lock: tauri::State<'_, super::AppDatabaseWriteLock>,
    name: String,
) -> Result<(), String> {
    let _write_guard = write_lock
        .lock()
        .map_err(|_| "CATEGORY_CREATE_FAILED".to_owned())?;
    state.create_category(name).map_err(str::to_owned)
}

#[cfg(test)]
#[path = "category_setup_tests.rs"]
mod tests;
