#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::ExitCode;

mod application;
mod domain;
mod infrastructure;

fn main() -> ExitCode {
    application::run_startup(infrastructure::startup, infrastructure::emit_failure)
}

#[cfg(test)]
mod security_tests;

#[cfg(all(test, windows))]
mod startup_tests;
