use crate::domain::ports::Clock;
use crate::domain::{brasilia_day_window_ms, sum_positive_movement_amounts_cents};
use std::time::UNIX_EPOCH;

#[derive(Debug)]
pub struct ListTodayExitsError;

impl std::fmt::Display for ListTodayExitsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "today's exits could not be read")
    }
}

impl std::error::Error for ListTodayExitsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub trait TodayExitReadStore {
    type Error;

    fn read_exit_amounts_created_between(
        &self,
        start_ms: i64,
        end_ms: i64,
    ) -> Result<Vec<i64>, Self::Error>;
}

pub fn list_today_exit_cents<C: Clock, S: TodayExitReadStore>(
    clock: &C,
    store: &S,
) -> Result<i64, ListTodayExitsError> {
    let now_ms = i64::try_from(
        clock
            .now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| ListTodayExitsError)?
            .as_millis(),
    )
    .map_err(|_| ListTodayExitsError)?;
    let (start_ms, end_ms) = brasilia_day_window_ms(now_ms).map_err(|_| ListTodayExitsError)?;
    let amounts = store
        .read_exit_amounts_created_between(start_ms, end_ms)
        .map_err(|_| ListTodayExitsError)?;

    sum_positive_movement_amounts_cents(&amounts).map_err(|_| ListTodayExitsError)
}

#[cfg(test)]
#[path = "list_today_exits_tests.rs"]
mod tests;
