pub(crate) mod account_setup;
pub(crate) mod category_setup;
pub(crate) mod entry_setup;
pub(crate) mod exit_setup;
pub(crate) mod transfer_setup;

use std::io::Write;
use std::sync::{LockResult, Mutex, MutexGuard};

use crate::application::Failure;
use tauri::Manager;

#[derive(Default)]
pub(crate) struct AppDatabaseWriteLock(Mutex<()>);

impl AppDatabaseWriteLock {
    pub(crate) fn lock(&self) -> LockResult<MutexGuard<'_, ()>> {
        self.0.lock()
    }
}

pub(crate) fn startup() -> tauri::Result<()> {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app
                .path()
                .app_local_data_dir()
                .map_err(|_| account_setup::StartupError)?;
            let account_store = account_setup::initialize_account_store_at(&data_dir)?;
            let category_store = category_setup::initialize_category_store_at(&data_dir)?;
            let entry_store = entry_setup::initialize_entry_store_at(&data_dir)?;
            let exit_store = exit_setup::initialize_exit_store_at(&data_dir)?;
            let transfer_store = transfer_setup::initialize_transfer_store_at(&data_dir)?;
            app.manage(AppDatabaseWriteLock::default());
            app.manage(account_setup::AccountSetupState::new(
                account_store,
                data_dir.clone(),
            ));
            app.manage(category_setup::CategorySetupState::new(
                category_store,
                data_dir.clone(),
            ));
            app.manage(entry_setup::EntrySetupState::new(
                entry_store,
                data_dir.clone(),
            ));
            app.manage(exit_setup::ExitSetupState::new(
                exit_store,
                data_dir.clone(),
            ));
            app.manage(transfer_setup::TransferSetupState::new(
                transfer_store,
                data_dir,
            ));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            account_setup::create_account,
            category_setup::create_category,
            entry_setup::list_accounts,
            entry_setup::create_entry,
            exit_setup::create_exit,
            transfer_setup::create_transfer
        ])
        .run(tauri::generate_context!())
}

pub(crate) fn emit_failure(failure: Failure) {
    let record: &[u8] = match failure {
        Failure::Startup => {
            b"{\"level\":\"error\",\"operation\":\"startup\",\"code\":\"TAURI_STARTUP_FAILED\"}\n"
        }
        Failure::Panic => {
            b"{\"level\":\"error\",\"operation\":\"runtime\",\"code\":\"INTERNAL_PANIC\"}\n"
        }
    };
    // A missing sink discards the safe record; never print the original error.
    let _ = std::io::stderr().lock().write_all(record);
}

#[cfg(test)]
#[path = "exit_setup_tests.rs"]
mod exit_setup_tests;

#[cfg(test)]
#[path = "transfer_setup_tests.rs"]
mod transfer_setup_tests;
