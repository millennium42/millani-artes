use crate::database::{append_audit_event, AuditEvent};
use crate::domain::Receivable;
use rusqlite::{params, OptionalExtension, Transaction};
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum ReceivableStoreError {
    InvalidAmount,
    SaleNotFinalized,
    StoreFailed,
}

impl fmt::Display for ReceivableStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidAmount => "receivable amount is invalid",
            Self::SaleNotFinalized => "sale is not finalized",
            Self::StoreFailed => "receivable could not be stored",
        })
    }
}

impl Error for ReceivableStoreError {}

pub fn persist_receivable_in_transaction(
    transaction: &Transaction<'_>,
    receivable: &Receivable,
    event_id: [u8; 16],
    occurred_at_ms: i64,
) -> Result<(), ReceivableStoreError> {
    if receivable.pending.amount_cents <= 0 {
        return Err(ReceivableStoreError::InvalidAmount);
    }

    let sale_status = transaction
        .query_row(
            "SELECT status FROM main.sales WHERE id = ?1",
            [receivable.sale_id.as_slice()],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|_| ReceivableStoreError::StoreFailed)?;

    match sale_status.as_deref() {
        Some("finalized") => {}
        Some("open") | None => return Err(ReceivableStoreError::SaleNotFinalized),
        Some(_) => return Err(ReceivableStoreError::StoreFailed),
    }

    let inserted = transaction
        .execute(
            "INSERT INTO main.receivables (id, sale_id, amount_cents, customer_id, due_at_ms) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                receivable.id.as_slice(),
                receivable.sale_id.as_slice(),
                receivable.pending.amount_cents,
                receivable.pending.customer_id.as_slice(),
                receivable.pending.due_at_ms,
            ],
        )
        .map_err(|_| ReceivableStoreError::StoreFailed)?;
    if inserted != 1 {
        return Err(ReceivableStoreError::StoreFailed);
    }

    append_audit_event(
        transaction,
        &AuditEvent {
            event_id,
            occurred_at_ms,
            operation: "receivable_create",
            entity: "receivable",
            entity_id: receivable.id,
        },
    )
    .map_err(|_| ReceivableStoreError::StoreFailed)
}
