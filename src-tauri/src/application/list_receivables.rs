use crate::domain::sum_nonnegative_receivable_due_cents;
use std::fmt;

#[derive(Debug)]
pub struct ListReceivableDueError;

impl fmt::Display for ListReceivableDueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("receivable total could not be read")
    }
}

impl std::error::Error for ListReceivableDueError {}

pub trait ReceivableDueReadStore {
    type Error;

    fn read_remaining_receivable_due_cents(&self) -> Result<Vec<i64>, Self::Error>;
}

pub fn list_receivable_due_cents<S: ReceivableDueReadStore>(
    store: &S,
) -> Result<i64, ListReceivableDueError> {
    let amounts = store
        .read_remaining_receivable_due_cents()
        .map_err(|_| ListReceivableDueError)?;

    sum_nonnegative_receivable_due_cents(&amounts).map_err(|_| ListReceivableDueError)
}

#[cfg(test)]
#[path = "list_receivables_tests.rs"]
mod tests;
