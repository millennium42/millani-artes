use crate::application::list_payables::PayableObligationsReadStore;
use rusqlite::Connection;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct PayableObligationsReadError;

impl fmt::Display for PayableObligationsReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("payable obligation history could not be read")
    }
}

impl Error for PayableObligationsReadError {}

pub struct AdapterSqlitePayableObligationsReadStore<'a> {
    connection: &'a Connection,
}

impl<'a> AdapterSqlitePayableObligationsReadStore<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }
}

impl PayableObligationsReadStore for AdapterSqlitePayableObligationsReadStore<'_> {
    type Error = PayableObligationsReadError;

    fn read_open_recurrence_obligation_amounts_cents(&self) -> Result<Vec<i64>, Self::Error> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT state,amount_cents,movement_id \
                 FROM main.recurrence_obligations ORDER BY id",
            )
            .map_err(|_| PayableObligationsReadError)?;
        let rows = statement
            .query_map([], |row| {
                let state: String = row.get(0)?;
                let amount_cents: Option<i64> = row.get(1)?;
                let movement_id: Option<Vec<u8>> = row.get(2)?;
                match (state.as_str(), amount_cents, movement_id) {
                    ("awaiting_value", None, None) => Ok(None),
                    ("payable" | "overdue", Some(amount_cents), None) if amount_cents > 0 => {
                        Ok(Some(amount_cents))
                    }
                    ("paid", Some(amount_cents), Some(movement_id))
                        if amount_cents > 0
                            && movement_id.len() == 16
                            && movement_id.iter().any(|byte| *byte != 0) =>
                    {
                        Ok(None)
                    }
                    _ => Err(rusqlite::Error::InvalidQuery),
                }
            })
            .map_err(|_| PayableObligationsReadError)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map(|amounts| amounts.into_iter().flatten().collect())
            .map_err(|_| PayableObligationsReadError)
    }
}
