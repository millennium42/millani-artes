use crate::domain::sum_open_recurrence_obligation_amounts_cents;
use std::fmt;

#[derive(Debug)]
pub struct ListPayablesError;

impl fmt::Display for ListPayablesError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("payable total could not be read")
    }
}

impl std::error::Error for ListPayablesError {}

pub trait PayableObligationsReadStore {
    type Error;

    fn read_open_recurrence_obligation_amounts_cents(&self) -> Result<Vec<i64>, Self::Error>;
}

pub fn list_payables_cents<S: PayableObligationsReadStore>(
    store: &S,
) -> Result<i64, ListPayablesError> {
    let amounts = store
        .read_open_recurrence_obligation_amounts_cents()
        .map_err(|_| ListPayablesError)?;

    sum_open_recurrence_obligation_amounts_cents(&amounts).map_err(|_| ListPayablesError)
}

#[cfg(test)]
#[path = "list_payables_tests.rs"]
mod tests;
