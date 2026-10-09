use std::process::ExitCode;

pub(crate) enum Failure {
    Startup,
    Panic,
}

pub(crate) fn run_startup<E>(
    startup: impl FnOnce() -> Result<(), E>,
    emit_failure: fn(Failure),
) -> ExitCode {
    std::panic::set_hook(Box::new(move |_| emit_failure(Failure::Panic)));
    match startup() {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => {
            emit_failure(Failure::Startup);
            ExitCode::FAILURE
        }
    }
}
