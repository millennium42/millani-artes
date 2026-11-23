pub(crate) mod account_setup;

use std::io::Write;

use crate::application::Failure;
use tauri::Manager;

pub(crate) fn startup() -> tauri::Result<()> {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app
                .path()
                .app_local_data_dir()
                .map_err(|_| account_setup::StartupError)?;
            let store = account_setup::initialize_account_store_at(&data_dir)?;
            app.manage(account_setup::AccountSetupState::new(store, data_dir));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![account_setup::create_account])
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
