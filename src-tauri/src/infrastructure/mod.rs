use std::io::Write;

use crate::application::Failure;

pub(crate) fn startup() -> tauri::Result<()> {
    tauri::Builder::default().run(tauri::generate_context!())
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
