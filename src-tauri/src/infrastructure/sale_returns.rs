use crate::application::record_sale_return::{
    RecordSaleReturnError, RecordSaleReturnWrite, SaleReturnRecordingStore,
};
use crate::database::{append_audit_event, execute_write_transaction, AuditEvent};
use crate::domain::{SaleItem, SaleReturnItem};
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
                        "SELECT id,product_id,description,quantity,unit_price_cents,discount_cents,total_cents FROM main.sale_items WHERE id=?1",
                        params![write.sale_item_id.as_slice()],
                        |row| {
                            Ok(SaleItem {
                                id: row.get(0)?,
                                product_id: row.get(1)?,
                                description: row.get(2)?,
                                quantity: row.get(3)?,
                                unit_price_cents: row.get(4)?,
                                discount_cents: row.get(5)?,
                                total_cents: row.get(6)?,
                            })
                        },
                    )
                    .optional()
                    .map_err(|_| RecordSaleReturnError::StoreFailed)?;
                let Some(item) = item else {
                    return Ok(Err(RecordSaleReturnError::SaleItemNotFound));
                };

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

                if transaction
                    .execute(
                        "INSERT INTO main.sale_return_items(id,sale_item_id,quantity) VALUES(?1,?2,?3)",
                        params![
                            write.return_id.as_slice(),
                            write.sale_item_id.as_slice(),
                            write.quantity
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
