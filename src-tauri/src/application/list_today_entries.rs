use crate::domain::ports::Clock;
use crate::domain::{brasilia_day_window_ms, sum_positive_movement_amounts_cents};
use std::time::UNIX_EPOCH;

#[derive(Debug)]
pub struct ListTodayEntriesError;

impl std::fmt::Display for ListTodayEntriesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "today's entries could not be read")
    }
}

impl std::error::Error for ListTodayEntriesError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub trait TodayEntryReadStore {
    type Error;

    fn read_entry_amounts_created_between(
        &self,
        start_ms: i64,
        end_ms: i64,
    ) -> Result<Vec<i64>, Self::Error>;
}

pub fn list_today_entry_cents<C: Clock, S: TodayEntryReadStore>(
    clock: &C,
    store: &S,
) -> Result<i64, ListTodayEntriesError> {
    let now_ms = i64::try_from(
        clock
            .now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| ListTodayEntriesError)?
            .as_millis(),
    )
    .map_err(|_| ListTodayEntriesError)?;
    let (start_ms, end_ms) = brasilia_day_window_ms(now_ms).map_err(|_| ListTodayEntriesError)?;
    let amounts = store
        .read_entry_amounts_created_between(start_ms, end_ms)
        .map_err(|_| ListTodayEntriesError)?;

    sum_positive_movement_amounts_cents(&amounts).map_err(|_| ListTodayEntriesError)
}

#[cfg(test)]
#[path = "list_today_entries_tests.rs"]
mod tests;
