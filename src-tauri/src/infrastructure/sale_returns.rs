use crate::application::record_sale_return::{
    RecordSaleReturnError, RecordSaleReturnWrite, SaleReturnRecordingStore,
};
use crate::database::{append_audit_event, execute_write_transaction, AuditEvent};
use crate::domain::{Receivable, SaleItem, SalePending, SaleReturnItem};
use rusqlite::{params, Connection, OptionalExtension};

pub struct AdapterSqliteSaleReturnStore {
    connection: Option<Connection>,
}

impl AdapterSqliteSaleReturnStore {
    pub fn new(connection: Connection) -> Self {
        Self {
            connection: Some(connection),
        }
    }
}

impl SaleReturnRecordingStore for AdapterSqliteSaleReturnStore {
    fn persist_return_with_audit(
        &mut self,
        write: RecordSaleReturnWrite,
    ) -> Result<(), RecordSaleReturnError> {
        if write.quantity <= 0 {
            return Err(RecordSaleReturnError::InvalidInput);
        }

        let connection = self
            .connection
            .take()
            .ok_or(RecordSaleReturnError::StoreFailed)?;
        let result = execute_write_transaction(
            connection,
            |transaction| -> Result<Result<(), RecordSaleReturnError>, RecordSaleReturnError> {
                // Validation exits precede every INSERT; this no-write commit is state-equivalent to rollback and preserves the connection.
                let item = transaction
                    .query_row(
                        "SELECT id,sale_id,product_id,description,quantity,unit_price_cents,discount_cents,total_cents FROM main.sale_items WHERE id=?1",
                        params![write.sale_item_id.as_slice()],
                        |row| {
                            Ok((
                                row.get::<_, Vec<u8>>(1)?,
                                SaleItem {
                                    id: row.get(0)?,
                                    product_id: row.get(2)?,
                                    description: row.get(3)?,
                                    quantity: row.get(4)?,
                                    unit_price_cents: row.get(5)?,
                                    discount_cents: row.get(6)?,
                                    total_cents: row.get(7)?,
                                },
                            ))
                        },
                    )
                    .optional()
                    .map_err(|_| RecordSaleReturnError::StoreFailed)?;
                let Some((sale_id, item)) = item else {
                    return Ok(Err(RecordSaleReturnError::SaleItemNotFound));
                };
                let sale_id: [u8; 16] = sale_id
                    .try_into()
                    .map_err(|_| RecordSaleReturnError::StoreFailed)?;

                let returns = {
                    let mut statement = transaction
                        .prepare(
                            "SELECT sale_item_id,quantity FROM main.sale_return_items WHERE sale_item_id=?1 ORDER BY id",
                        )
                        .map_err(|_| RecordSaleReturnError::StoreFailed)?;
                    let rows = statement
                        .query_map(params![write.sale_item_id.as_slice()], |row| {
                            Ok(SaleReturnItem {
                                sale_item_id: row.get(0)?,
                                quantity: row.get(1)?,
                            })
                        })
                        .map_err(|_| RecordSaleReturnError::StoreFailed)?;
                    rows.collect::<rusqlite::Result<Vec<_>>>()
                        .map_err(|_| RecordSaleReturnError::StoreFailed)?
                };
                if item
                    .validate_return_quantity(write.quantity, &returns)
                    .is_err()
                {
                    return Ok(Err(RecordSaleReturnError::QuantityExceedsAvailable));
                }

                let prior_returned_quantity = item.quantity
                    - item
                        .available_return_quantity(&returns)
                        .map_err(|_| RecordSaleReturnError::StoreFailed)?;
                let return_amount_cents = item
                    .cumulative_return_amount_cents(prior_returned_quantity, write.quantity)
                    .map_err(|_| RecordSaleReturnError::StoreFailed)?;
                let debt_reduction_cents = transaction
                    .query_row(
                        "SELECT r.id,r.amount_cents,r.customer_id,r.due_at_ms,COALESCE((SELECT SUM(receipt.amount_cents) FROM main.receipts AS receipt WHERE receipt.receivable_id=r.id),0),COALESCE((SELECT SUM(returned.debt_reduction_cents) FROM main.sale_return_items AS returned JOIN main.sale_items AS item ON item.id=returned.sale_item_id WHERE item.sale_id=r.sale_id),0) FROM main.receivables AS r WHERE r.sale_id=?1",
                        params![sale_id.as_slice()],
                        |row| {
                            Ok((
                                row.get::<_, Vec<u8>>(0)?,
                                row.get::<_, i64>(1)?,
                                row.get::<_, Vec<u8>>(2)?,
                                row.get::<_, i64>(3)?,
                                row.get::<_, i64>(4)?,
                                row.get::<_, i64>(5)?,
                            ))
                        },
                    )
                    .optional()
                    .map_err(|_| RecordSaleReturnError::StoreFailed)?
                    .map(|(receivable_id, principal_cents, customer_id, due_at_ms, receipts_cents, prior_reductions_cents)| {
                        let applied_cents = receipts_cents
                            .checked_add(prior_reductions_cents)
                            .ok_or(RecordSaleReturnError::StoreFailed)?;
                        let receivable = Receivable {
                            id: receivable_id
                                .try_into()
                                .map_err(|_| RecordSaleReturnError::StoreFailed)?,
                            sale_id,
                            pending: SalePending {
                                amount_cents: principal_cents,
                                customer_id: customer_id
                                    .try_into()
                                    .map_err(|_| RecordSaleReturnError::StoreFailed)?,
                                due_at_ms,
                            },
                        };
                        let due_cents = receivable
                            .remaining_due_cents(applied_cents)
                            .map_err(|_| RecordSaleReturnError::StoreFailed)?;
                        Ok::<_, RecordSaleReturnError>(return_amount_cents.min(due_cents))
                    })
                    .transpose()?
                    .unwrap_or(0);

                if transaction
                    .execute(
                        "INSERT INTO main.sale_return_items(id,sale_item_id,quantity,return_amount_cents,debt_reduction_cents) VALUES(?1,?2,?3,?4,?5)",
                        params![
                            write.return_id.as_slice(),
                            write.sale_item_id.as_slice(),
                            write.quantity,
                            return_amount_cents,
                            debt_reduction_cents,
                        ],
                    )
                    .map_err(|_| RecordSaleReturnError::StoreFailed)?
                    != 1
                {
                    return Err(RecordSaleReturnError::StoreFailed);
                }
                append_audit_event(
                    transaction,
                    &AuditEvent {
                        event_id: write.audit_event_id,
                        occurred_at_ms: write.occurred_at_ms,
                        operation: "sale_return_item_create",
                        entity: "sale_return_item",
                        entity_id: write.return_id,
                    },
                )
                .map_err(|_| RecordSaleReturnError::StoreFailed)?;
                Ok(Ok(()))
            },
        );

        match result {
            Ok((connection, outcome)) => {
                self.connection = Some(connection);
                outcome
            }
            Err(_) => Err(RecordSaleReturnError::StoreFailed),
        }
    }
}
