use crate::application::record_receivable_receipt::{
    ReceivableReceiptRecordingStore, ReceivableReceiptWrite, RecordReceivableReceiptError,
};
use crate::database::execute_write_transaction;
use crate::database::{append_audit_event, AuditEvent};
use crate::domain::{Context, MovementKind, Receivable, ReceivableReceipt, SalePending};
use crate::infrastructure::movements::persist_entry_in_transaction;
use rusqlite::{params, Connection, OptionalExtension, Transaction};
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

#[derive(Debug, PartialEq, Eq)]
pub enum ReceivableReceiptStoreError {
    InvalidAmount,
    ReceivableNotFound,
    AmountExceedsDue,
    StoreFailed,
}

impl fmt::Display for ReceivableReceiptStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidAmount => "receipt amount is invalid",
            Self::ReceivableNotFound => "receivable could not be found",
            Self::AmountExceedsDue => "receipt amount exceeds the due balance",
            Self::StoreFailed => "receipt could not be stored",
        })
    }
}

impl Error for ReceivableReceiptStoreError {}

/// Persists a receipt and its technical audit event in the caller's write transaction.
/// It deliberately does not create the financial entry or control commit; RCV-007 composes
/// both writes in one application-owned transaction.
pub fn persist_receivable_receipt_in_transaction(
    transaction: &Transaction<'_>,
    receivable_id: [u8; 16],
    receipt_id: [u8; 16],
    receipt: &ReceivableReceipt,
    event_id: [u8; 16],
    occurred_at_ms: i64,
) -> Result<(), ReceivableReceiptStoreError> {
    if receipt.amount_cents <= 0 {
        return Err(ReceivableReceiptStoreError::InvalidAmount);
    }

    let (sale_id, principal_cents, customer_id, due_at_ms, applied_cents) = transaction
        .query_row(
            "SELECT r.sale_id, r.amount_cents, r.customer_id, r.due_at_ms, COALESCE(SUM(receipt.amount_cents), 0) FROM main.receivables AS r LEFT JOIN main.receipts AS receipt ON receipt.receivable_id = r.id WHERE r.id = ?1 GROUP BY r.id, r.sale_id, r.amount_cents, r.customer_id, r.due_at_ms",
            [receivable_id.as_slice()],
            |row| {
                Ok((
                    row.get::<_, Vec<u8>>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            },
        )
        .optional()
        .map_err(|_| ReceivableReceiptStoreError::StoreFailed)?
        .ok_or(ReceivableReceiptStoreError::ReceivableNotFound)?;

    let sale_id: [u8; 16] = sale_id
        .try_into()
        .map_err(|_| ReceivableReceiptStoreError::StoreFailed)?;
    let customer_id: [u8; 16] = customer_id
        .try_into()
        .map_err(|_| ReceivableReceiptStoreError::StoreFailed)?;
    let receivable = Receivable {
        id: receivable_id,
        sale_id,
        pending: SalePending {
            amount_cents: principal_cents,
            customer_id,
            due_at_ms,
        },
    };
    let due_cents = receivable
        .remaining_due_cents(applied_cents)
        .map_err(|_| ReceivableReceiptStoreError::StoreFailed)?;
    if receipt.amount_cents > due_cents {
        return Err(ReceivableReceiptStoreError::AmountExceedsDue);
    }

    let context = match &receipt.context {
        Context::Home => "home",
        Context::Factory => "factory",
    };
    let inserted = transaction
        .execute(
            "INSERT INTO main.receipts (id, receivable_id, account_id, amount_cents, context) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                receipt_id.as_slice(),
                receivable_id.as_slice(),
                receipt.account_id.as_slice(),
                receipt.amount_cents,
                context,
            ],
        )
        .map_err(|_| ReceivableReceiptStoreError::StoreFailed)?;
    if inserted != 1 {
        return Err(ReceivableReceiptStoreError::StoreFailed);
    }

    append_audit_event(
        transaction,
        &AuditEvent {
            event_id,
            occurred_at_ms,
            operation: "receivable_receipt_create",
            entity: "receipt",
            entity_id: receipt_id,
        },
    )
    .map_err(|_| ReceivableReceiptStoreError::StoreFailed)
}

pub struct AdapterSqliteReceivableReceiptStore {
    connection: Option<Connection>,
}

impl AdapterSqliteReceivableReceiptStore {
    pub fn new(connection: Connection) -> Self {
        Self {
            connection: Some(connection),
        }
    }
}

impl ReceivableReceiptRecordingStore for AdapterSqliteReceivableReceiptStore {
    fn persist_receipt_and_entry(
        &mut self,
        write: ReceivableReceiptWrite<'_>,
    ) -> Result<(), RecordReceivableReceiptError> {
        let account_matches = match &write.movement.kind {
            MovementKind::Entry { account_id } => account_id == &write.receipt.account_id,
            _ => false,
        };
        let context_matches = matches!(
            (&write.receipt.context, &write.movement.context),
            (Context::Home, Context::Home) | (Context::Factory, Context::Factory)
        );
        if !account_matches
            || write.movement.amount_cents != write.receipt.amount_cents
            || !context_matches
        {
            return Err(RecordReceivableReceiptError::InvalidInput);
        }

        let connection = self
            .connection
            .take()
            .ok_or(RecordReceivableReceiptError::StoreFailed)?;
        let result =
            execute_write_transaction(
                connection,
                |transaction| -> Result<
                    Result<(), RecordReceivableReceiptError>,
                    RecordReceivableReceiptError,
                > {
                    match persist_receivable_receipt_in_transaction(
                        transaction,
                        write.receivable_id,
                        write.receipt_id,
                        write.receipt,
                        write.receipt_event_id,
                        write.occurred_at_ms,
                    ) {
                        Ok(()) => (),
                        Err(ReceivableReceiptStoreError::InvalidAmount) => {
                            return Ok(Err(RecordReceivableReceiptError::InvalidInput));
                        }
                        Err(ReceivableReceiptStoreError::ReceivableNotFound) => {
                            return Ok(Err(RecordReceivableReceiptError::ReceivableNotFound));
                        }
                        Err(ReceivableReceiptStoreError::AmountExceedsDue) => {
                            return Ok(Err(RecordReceivableReceiptError::AmountExceedsDue));
                        }
                        Err(ReceivableReceiptStoreError::StoreFailed) => {
                            return Err(RecordReceivableReceiptError::StoreFailed);
                        }
                    }

                    persist_entry_in_transaction(
                        transaction,
                        write.movement,
                        write.movement_event_id,
                        write.occurred_at_ms,
                    )
                    .map_err(|_| RecordReceivableReceiptError::StoreFailed)?;
                    Ok(Ok(()))
                },
            );

        match result {
            Ok((connection, outcome)) => {
                self.connection = Some(connection);
                outcome
            }
            Err(_) => Err(RecordReceivableReceiptError::StoreFailed),
        }
    }
}
