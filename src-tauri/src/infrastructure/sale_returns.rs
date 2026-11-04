use crate::application::cancel_paid_sale::{
    CancelPaidSaleError, SaleCancellationPreview, SaleCancellationRefundPlan,
    SaleCancellationRefundTarget, SaleCancellationReturnPlan, SaleCancellationStore,
    SaleCancellationWrite,
};
use crate::application::record_sale_refund::{
    RecordSaleRefundError, SaleRefundRecordingStore, SaleRefundWrite,
};
use crate::application::record_sale_return::{
    RecordSaleReturnError, RecordSaleReturnWrite, SaleReturnRecordingStore,
};
use crate::database::{append_audit_event, execute_write_transaction, AuditEvent};
use crate::domain::{Context, MovementKind, Receivable, SaleItem, SalePending, SaleReturnItem};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::{HashMap, HashSet};

pub struct AdapterSqliteSaleCancellationStore {
    connection: Option<Connection>,
}

impl AdapterSqliteSaleCancellationStore {
    pub fn new(connection: Connection) -> Self {
        Self {
            connection: Some(connection),
        }
    }
}

impl SaleCancellationStore for AdapterSqliteSaleCancellationStore {
    fn preview_sale_cancellation(
        &mut self,
        sale_id: [u8; 16],
    ) -> Result<SaleCancellationPreview, CancelPaidSaleError> {
        let mut connection = self
            .connection
            .take()
            .ok_or(CancelPaidSaleError::StoreFailed)?;
        let result = match connection.transaction() {
            Ok(transaction) => {
                let preview = build_cancellation_preview(&transaction, sale_id);
                if transaction.commit().is_err() {
                    Err(CancelPaidSaleError::StoreFailed)
                } else {
                    preview
                }
            }
            Err(_) => Err(CancelPaidSaleError::StoreFailed),
        };
        self.connection = Some(connection);
        result
    }

    fn persist_paid_sale_cancellation(
        &mut self,
        write: SaleCancellationWrite,
    ) -> Result<(), CancelPaidSaleError> {
        if write.occurred_at_ms < 0
            || write.cancellation_audit_event_id == [0; 16]
            || write
                .returns
                .iter()
                .any(|item| item.id == [0; 16] || item.audit_event_id == [0; 16])
            || write
                .refunds
                .iter()
                .any(|refund| refund.movement_id == [0; 16] || refund.audit_event_id == [0; 16])
        {
            return Err(CancelPaidSaleError::InvalidInput);
        }

        let connection = self
            .connection
            .take()
            .ok_or(CancelPaidSaleError::StoreFailed)?;
        let result = execute_write_transaction(connection, |transaction| {
            let current = match build_cancellation_preview(transaction, write.preview.sale_id) {
                Ok(current) => current,
                Err(error) => return Ok(Err(error)),
            };
            if current != write.preview || !cancellation_writes_match(&write) {
                return Ok(Err(CancelPaidSaleError::InvalidInput));
            }
            for refund in &write.refunds {
                let exists: bool = transaction
                    .query_row(
                        "SELECT EXISTS(SELECT 1 FROM main.accounts WHERE id=?1)",
                        params![refund.account_id.as_slice()],
                        |row| row.get(0),
                    )
                    .map_err(|_| CancelPaidSaleError::StoreFailed)?;
                if !exists {
                    return Ok(Err(CancelPaidSaleError::AccountNotFound));
                }
            }

            for item in &write.returns {
                if transaction.execute(
                    "INSERT INTO main.sale_return_items(id,sale_item_id,quantity,return_amount_cents,debt_reduction_cents) VALUES(?1,?2,?3,?4,0)",
                    params![item.id.as_slice(),item.sale_item_id.as_slice(),item.quantity,item.return_amount_cents],
                ).map_err(|_| CancelPaidSaleError::StoreFailed)? != 1 {
                    return Err(CancelPaidSaleError::StoreFailed);
                }
                append_cancellation_audit(
                    transaction,
                    item.audit_event_id,
                    write.occurred_at_ms,
                    "sale_return_item_create",
                    "sale_return_item",
                    item.id,
                )?;
            }

            for refund in &write.refunds {
                let context = match refund.context {
                    Context::Home => "home",
                    Context::Factory => "factory",
                };
                if transaction.execute(
                    "INSERT INTO main.movements(id,kind,amount_cents,context,account_id) VALUES(?1,'exit',?2,?3,?4)",
                    params![refund.movement_id.as_slice(),refund.amount_cents,context,refund.account_id.as_slice()],
                ).map_err(|_| CancelPaidSaleError::StoreFailed)? != 1 {
                    return Err(CancelPaidSaleError::StoreFailed);
                }
                if transaction.execute(
                    "INSERT INTO main.sale_return_refunds(movement_id,sale_return_item_id) VALUES(?1,?2)",
                    params![refund.movement_id.as_slice(),refund.return_item_id.as_slice()],
                ).map_err(|_| CancelPaidSaleError::StoreFailed)? != 1 {
                    return Err(CancelPaidSaleError::StoreFailed);
                }
                append_cancellation_audit(
                    transaction,
                    refund.audit_event_id,
                    write.occurred_at_ms,
                    "sale_return_refund_create",
                    "movement",
                    refund.movement_id,
                )?;
            }

            if transaction
                .execute(
                    "INSERT INTO main.sale_cancellations(sale_id,cancelled_at_ms) VALUES(?1,?2)",
                    params![write.preview.sale_id.as_slice(), write.occurred_at_ms],
                )
                .map_err(|_| CancelPaidSaleError::StoreFailed)?
                != 1
            {
                return Err(CancelPaidSaleError::StoreFailed);
            }
            append_cancellation_audit(
                transaction,
                write.cancellation_audit_event_id,
                write.occurred_at_ms,
                "sale_cancelled",
                "sale",
                write.preview.sale_id,
            )?;
            Ok(Ok(()))
        });

        match result {
            Ok((connection, outcome)) => {
                self.connection = Some(connection);
                outcome
            }
            Err(_) => Err(CancelPaidSaleError::StoreFailed),
        }
    }
}

fn cancellation_writes_match(write: &SaleCancellationWrite) -> bool {
    if write.returns.len() != write.preview.returns.len()
        || write.refunds.len() != write.preview.refunds.len()
    {
        return false;
    }
    let mut return_ids = HashMap::with_capacity(write.returns.len());
    for (item, expected) in write.returns.iter().zip(&write.preview.returns) {
        if item.sale_item_id != expected.sale_item_id
            || item.quantity != expected.quantity
            || item.return_amount_cents != expected.return_amount_cents
            || return_ids.insert(item.sale_item_id, item.id).is_some()
        {
            return false;
        }
    }
    let mut targets = HashSet::with_capacity(write.refunds.len());
    for (refund, expected) in write.refunds.iter().zip(&write.preview.refunds) {
        let expected_return_item_id = match expected.target {
            SaleCancellationRefundTarget::ExistingReturnItem(id) => id,
            SaleCancellationRefundTarget::CancellationReturnForSaleItem(sale_item_id) => {
                let Some(id) = return_ids.get(&sale_item_id) else {
                    return false;
                };
                *id
            }
        };
        if refund.target != expected.target
            || refund.return_item_id != expected_return_item_id
            || refund.amount_cents != expected.amount_cents
            || refund.amount_cents <= 0
            || !targets.insert(refund.target)
        {
            return false;
        }
    }
    true
}

fn build_cancellation_preview(
    connection: &Connection,
    sale_id: [u8; 16],
) -> Result<SaleCancellationPreview, CancelPaidSaleError> {
    let status: Option<(String, bool)> = connection
        .query_row(
            "SELECT status,EXISTS(SELECT 1 FROM main.sale_cancellations WHERE sale_id=sales.id) FROM main.sales WHERE id=?1",
            params![sale_id.as_slice()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|_| CancelPaidSaleError::StoreFailed)?;
    let Some((status, cancelled)) = status else {
        return Err(CancelPaidSaleError::SaleNotFound);
    };
    if cancelled {
        return Err(CancelPaidSaleError::AlreadyCancelled);
    }
    if status != "finalized" {
        return Err(CancelPaidSaleError::SaleNotFinalized);
    }

    let receivable = connection
        .query_row(
            "SELECT r.id,r.amount_cents,r.customer_id,r.due_at_ms,COALESCE((SELECT SUM(receipt.amount_cents) FROM main.receipts AS receipt WHERE receipt.receivable_id=r.id),0),COALESCE((SELECT SUM(returned.debt_reduction_cents) FROM main.sale_return_items AS returned JOIN main.sale_items AS item ON item.id=returned.sale_item_id WHERE item.sale_id=r.sale_id),0) FROM main.receivables AS r WHERE r.sale_id=?1",
            params![sale_id.as_slice()],
            |row| Ok((
                row.get::<_, Vec<u8>>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, Vec<u8>>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
            )),
        )
        .optional()
        .map_err(|_| CancelPaidSaleError::StoreFailed)?;
    if let Some((id, amount_cents, customer_id, due_at_ms, receipts, reductions)) = receivable {
        let applied = receipts
            .checked_add(reductions)
            .ok_or(CancelPaidSaleError::StoreFailed)?;
        let receivable = Receivable {
            id: id
                .try_into()
                .map_err(|_| CancelPaidSaleError::StoreFailed)?,
            sale_id,
            pending: SalePending {
                amount_cents,
                customer_id: customer_id
                    .try_into()
                    .map_err(|_| CancelPaidSaleError::StoreFailed)?,
                due_at_ms,
            },
        };
        if receivable
            .remaining_due_cents(applied)
            .map_err(|_| CancelPaidSaleError::StoreFailed)?
            != 0
        {
            return Err(CancelPaidSaleError::ReceivableNotSettled);
        }
    }

    let items = {
        let mut statement = connection
            .prepare("SELECT id,product_id,description,quantity,unit_price_cents,discount_cents,total_cents FROM main.sale_items WHERE sale_id=?1 ORDER BY position")
            .map_err(|_| CancelPaidSaleError::StoreFailed)?;
        let rows = statement
            .query_map(params![sale_id.as_slice()], |row| {
                Ok(SaleItem {
                    id: row.get(0)?,
                    product_id: row.get(1)?,
                    description: row.get(2)?,
                    quantity: row.get(3)?,
                    unit_price_cents: row.get(4)?,
                    discount_cents: row.get(5)?,
                    total_cents: row.get(6)?,
                })
            })
            .map_err(|_| CancelPaidSaleError::StoreFailed)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|_| CancelPaidSaleError::StoreFailed)?
    };
    if items.is_empty() {
        return Err(CancelPaidSaleError::StoreFailed);
    }

    let returned = {
        let mut statement = connection
            .prepare(
                "SELECT returned.id,returned.sale_item_id,returned.quantity,returned.return_amount_cents,returned.debt_reduction_cents,COALESCE((SELECT SUM(movement.amount_cents) FROM main.sale_return_refunds AS refund JOIN main.movements AS movement ON movement.id=refund.movement_id WHERE refund.sale_return_item_id=returned.id),0) FROM main.sale_return_items AS returned JOIN main.sale_items AS item ON item.id=returned.sale_item_id WHERE item.sale_id=?1 ORDER BY returned.rowid",
            )
            .map_err(|_| CancelPaidSaleError::StoreFailed)?;
        let rows = statement
            .query_map(params![sale_id.as_slice()], |row| {
                Ok((
                    row.get::<_, [u8; 16]>(0)?,
                    row.get::<_, [u8; 16]>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                ))
            })
            .map_err(|_| CancelPaidSaleError::StoreFailed)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|_| CancelPaidSaleError::StoreFailed)?
    };

    let mut return_plans = Vec::new();
    let mut refund_plans = Vec::new();
    for item in &items {
        let item_returns = returned
            .iter()
            .filter(|(_, sale_item_id, ..)| *sale_item_id == item.id)
            .map(|(_, sale_item_id, quantity, ..)| SaleReturnItem {
                sale_item_id: *sale_item_id,
                quantity: *quantity,
            })
            .collect::<Vec<_>>();
        let available = item
            .available_return_quantity(&item_returns)
            .map_err(|_| CancelPaidSaleError::StoreFailed)?;
        if available > 0 {
            let previously_returned = item
                .quantity
                .checked_sub(available)
                .ok_or(CancelPaidSaleError::StoreFailed)?;
            let return_amount_cents = item
                .cumulative_return_amount_cents(previously_returned, available)
                .map_err(|_| CancelPaidSaleError::StoreFailed)?;
            return_plans.push(SaleCancellationReturnPlan {
                sale_item_id: item.id,
                quantity: available,
                return_amount_cents,
            });
        }
        for (id, _, _, return_amount, debt_reduction, refunded) in returned
            .iter()
            .filter(|(_, sale_item_id, ..)| *sale_item_id == item.id)
        {
            let available_refund = return_amount
                .checked_sub(*debt_reduction)
                .and_then(|reserved| reserved.checked_sub(*refunded))
                .ok_or(CancelPaidSaleError::StoreFailed)?;
            if available_refund > 0 {
                refund_plans.push(SaleCancellationRefundPlan {
                    target: SaleCancellationRefundTarget::ExistingReturnItem(*id),
                    amount_cents: available_refund,
                });
            }
        }
        if let Some(plan) = return_plans
            .last()
            .filter(|plan| plan.sale_item_id == item.id)
        {
            if plan.return_amount_cents > 0 {
                refund_plans.push(SaleCancellationRefundPlan {
                    target: SaleCancellationRefundTarget::CancellationReturnForSaleItem(item.id),
                    amount_cents: plan.return_amount_cents,
                });
            }
        }
    }
    Ok(SaleCancellationPreview {
        sale_id,
        returns: return_plans,
        refunds: refund_plans,
    })
}

fn append_cancellation_audit(
    transaction: &rusqlite::Transaction<'_>,
    event_id: [u8; 16],
    occurred_at_ms: i64,
    operation: &'static str,
    entity: &'static str,
    entity_id: [u8; 16],
) -> Result<(), CancelPaidSaleError> {
    append_audit_event(
        transaction,
        &AuditEvent {
            event_id,
            occurred_at_ms,
            operation,
            entity,
            entity_id,
        },
    )
    .map_err(|_| CancelPaidSaleError::StoreFailed)
}

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

pub struct AdapterSqliteSaleRefundStore {
    connection: Option<Connection>,
}

impl AdapterSqliteSaleRefundStore {
    pub fn new(connection: Connection) -> Self {
        Self {
            connection: Some(connection),
        }
    }
}

impl SaleRefundRecordingStore for AdapterSqliteSaleRefundStore {
    fn persist_refund_with_audit(
        &mut self,
        write: SaleRefundWrite<'_>,
    ) -> Result<(), RecordSaleRefundError> {
        let account_id = match &write.movement.kind {
            MovementKind::Exit { account_id } => *account_id,
            _ => return Err(RecordSaleRefundError::InvalidInput),
        };
        write
            .movement
            .validate_exit()
            .map_err(|_| RecordSaleRefundError::InvalidInput)?;
        let connection = self
            .connection
            .take()
            .ok_or(RecordSaleRefundError::StoreFailed)?;
        let result = execute_write_transaction(connection, |transaction| {
            let reservation = transaction
                .query_row(
                    "SELECT returned.return_amount_cents,returned.debt_reduction_cents,COALESCE((SELECT SUM(movement.amount_cents) FROM main.sale_return_refunds AS refund JOIN main.movements AS movement ON movement.id=refund.movement_id WHERE refund.sale_return_item_id=returned.id),0) FROM main.sale_return_items AS returned WHERE returned.id=?1",
                    params![write.return_item_id.as_slice()],
                    |row| Ok((row.get::<_, i64>(0)?,row.get::<_, i64>(1)?,row.get::<_, i64>(2)?)),
                )
                .optional()
                .map_err(|_| RecordSaleRefundError::StoreFailed)?;
            let Some((return_amount_cents, debt_reduction_cents, refunded_cents)) = reservation
            else {
                return Ok(Err(RecordSaleRefundError::ReturnItemNotFound));
            };
            let reserved_cents = return_amount_cents
                .checked_sub(debt_reduction_cents)
                .ok_or(RecordSaleRefundError::StoreFailed)?;
            let available_cents = reserved_cents.saturating_sub(refunded_cents).max(0);
            if write.movement.amount_cents > available_cents {
                return Ok(Err(RecordSaleRefundError::AmountExceedsAvailable));
            }

            let context = match &write.movement.context {
                Context::Home => "home",
                Context::Factory => "factory",
            };
            if transaction.execute(
                "INSERT INTO main.movements(id,kind,amount_cents,context,account_id) VALUES(?1,'exit',?2,?3,?4)",
                params![write.movement.id.as_slice(),write.movement.amount_cents,context,account_id.as_slice()],
            ).map_err(|_| RecordSaleRefundError::StoreFailed)? != 1 {
                return Err(RecordSaleRefundError::StoreFailed);
            }
            if transaction.execute(
                "INSERT INTO main.sale_return_refunds(movement_id,sale_return_item_id) VALUES(?1,?2)",
                params![write.movement.id.as_slice(),write.return_item_id.as_slice()],
            ).map_err(|_| RecordSaleRefundError::StoreFailed)? != 1 {
                return Err(RecordSaleRefundError::StoreFailed);
            }
            append_audit_event(
                transaction,
                &AuditEvent {
                    event_id: write.audit_event_id,
                    occurred_at_ms: write.occurred_at_ms,
                    operation: "sale_return_refund_create",
                    entity: "movement",
                    entity_id: write.movement.id,
                },
            )
            .map_err(|_| RecordSaleRefundError::StoreFailed)?;
            Ok(Ok(()))
        });

        match result {
            Ok((connection, outcome)) => {
                self.connection = Some(connection);
                outcome
            }
            Err(_) => Err(RecordSaleRefundError::StoreFailed),
        }
    }
}
