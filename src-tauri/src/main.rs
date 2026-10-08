#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{io::Write, process::ExitCode};

enum Failure {
    Startup,
    Panic,
}

fn emit_failure(failure: Failure) {
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

fn run_startup(startup: impl FnOnce() -> tauri::Result<()>) -> ExitCode {
    std::panic::set_hook(Box::new(|_| emit_failure(Failure::Panic)));
    match startup() {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => {
            emit_failure(Failure::Startup);
            ExitCode::FAILURE
        }
    }
}

fn main() -> ExitCode {
    run_startup(|| tauri::Builder::default().run(tauri::generate_context!()))
}

#[cfg(test)]
mod security_tests;

#[cfg(all(test, windows))]
mod startup_tests;
