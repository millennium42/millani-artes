use crate::application::list_receivables::ReceivableDueReadStore;
use crate::domain::{Receivable, SalePending};
use rusqlite::Connection;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ReceivableDueReadError;

impl fmt::Display for ReceivableDueReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("receivable due history could not be read")
    }
}

impl Error for ReceivableDueReadError {}

pub struct AdapterSqliteReceivableDueReadStore<'a> {
    connection: &'a Connection,
}

impl<'a> AdapterSqliteReceivableDueReadStore<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }
}

impl ReceivableDueReadStore for AdapterSqliteReceivableDueReadStore<'_> {
    type Error = ReceivableDueReadError;

    fn read_remaining_receivable_due_cents(&self) -> Result<Vec<i64>, Self::Error> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT r.id,r.sale_id,r.amount_cents,r.customer_id,r.due_at_ms, \
                 COALESCE((SELECT SUM(receipt.amount_cents) FROM main.receipts AS receipt WHERE receipt.receivable_id=r.id),0) + \
                 COALESCE((SELECT SUM(returned.debt_reduction_cents) FROM main.sale_return_items AS returned \
                 JOIN main.sale_items AS item ON item.id=returned.sale_item_id WHERE item.sale_id=r.sale_id),0) \
                 FROM main.receivables AS r \
                 JOIN main.sales AS sale ON sale.id=r.sale_id \
                 WHERE sale.status='finalized' \
                 ORDER BY r.id",
            )
            .map_err(|_| ReceivableDueReadError)?;
        let rows = statement
            .query_map([], |row| {
                let receivable_id: [u8; 16] = row
                    .get::<_, Vec<u8>>(0)?
                    .try_into()
                    .map_err(|_| rusqlite::Error::InvalidQuery)?;
                let sale_id: [u8; 16] = row
                    .get::<_, Vec<u8>>(1)?
                    .try_into()
                    .map_err(|_| rusqlite::Error::InvalidQuery)?;
                let principal_cents = row.get(2)?;
                let customer_id: [u8; 16] = row
                    .get::<_, Vec<u8>>(3)?
                    .try_into()
                    .map_err(|_| rusqlite::Error::InvalidQuery)?;
                let due_at_ms = row.get(4)?;
                let applied_cents = row.get(5)?;
                let receivable = Receivable {
                    id: receivable_id,
                    sale_id,
                    pending: SalePending {
                        amount_cents: principal_cents,
                        customer_id,
                        due_at_ms,
                    },
                };
                receivable
                    .remaining_due_cents(applied_cents)
                    .map_err(|_| rusqlite::Error::InvalidQuery)
            })
            .map_err(|_| ReceivableDueReadError)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|_| ReceivableDueReadError)
    }
}
