use crate::application::add_free_item::{FreeItemStore, FreeItemWrite};
use crate::application::add_product_item::{ProductItemWrite, SaleError, SaleStore};
use crate::application::finalize_sale::{FinalizeSaleWrite, FinalizedSale, SaleFinalizationStore};
use crate::application::remove_sale_item::{RemoveSaleItemWrite, SaleItemRemovalStore};
use crate::database::{append_audit_event, execute_write_transaction, AuditEvent};
use crate::domain::{
    create_free_item, create_product_item, DiscountInput, MovementKind, Product, Receivable, Sale,
    SaleItem, SalePending, SaleStatus,
};
use crate::infrastructure::movements::persist_entry_in_transaction;
use crate::infrastructure::receivables::persist_receivable_in_transaction;
use rusqlite::{params, Connection, OptionalExtension, Transaction};

enum ItemSource {
    Product([u8; 16]),
    Free(String),
}

struct PersistSaleItem {
    sale_id: [u8; 16],
    create_sale: bool,
    item_id: [u8; 16],
    source: ItemSource,
    quantity: i64,
    unit_price_cents: i64,
    discount: DiscountInput,
    event_id: [u8; 16],
    occurred_at_ms: i64,
}

pub struct AdapterSqliteSaleStore {
    connection: Option<Connection>,
}
impl AdapterSqliteSaleStore {
    pub fn new(connection: Connection) -> Self {
        Self {
            connection: Some(connection),
        }
    }

    fn persist_item_with_audit(&mut self, write: PersistSaleItem) -> Result<SaleItem, SaleError> {
        let connection = self.connection.take().ok_or(SaleError::StoreFailed)?;
        let result = execute_write_transaction(
            connection,
            |transaction| -> Result<Result<SaleItem, SaleError>, ()> {
                if !write.create_sale {
                    let status: Option<String> = transaction
                        .query_row(
                            "SELECT status FROM main.sales WHERE id=?1",
                            params![write.sale_id.as_slice()],
                            |r| r.get(0),
                        )
                        .optional()
                        .map_err(|_| ())?;
                    match status.as_deref() {
                        None => return Ok(Err(SaleError::SaleNotFound)),
                        Some("open") => (),
                        Some(_) => return Ok(Err(SaleError::SaleNotOpen)),
                    }
                }
                let item = match write.source {
                    ItemSource::Product(product_id) => {
                        let product = transaction
                            .query_row(
                                "SELECT id,name,description,suggested_price_cents,is_active FROM main.products WHERE id=?1",
                                params![product_id.as_slice()],
                                |r| Ok(Product {
                                    id: r.get(0)?,
                                    name: r.get(1)?,
                                    description: r.get(2)?,
                                    suggested_price_cents: r.get(3)?,
                                    is_active: r.get(4)?,
                                }),
                            )
                            .optional()
                            .map_err(|_| ())?;
                        let Some(product) = product else {
                            return Ok(Err(SaleError::ProductNotFound));
                        };
                        match create_product_item(
                            write.item_id,
                            &product,
                            write.quantity,
                            write.unit_price_cents,
                            write.discount,
                        ) {
                            Ok(item) => item,
                            Err(_) => return Ok(Err(SaleError::InvalidItem)),
                        }
                    }
                    ItemSource::Free(description) => match create_free_item(
                        write.item_id,
                        description,
                        write.quantity,
                        write.unit_price_cents,
                        write.discount,
                    ) {
                        Ok(item) => item,
                        Err(_) => return Ok(Err(SaleError::InvalidItem)),
                    },
                };
                let last: Option<i64> = transaction
                    .query_row(
                        "SELECT MAX(position) FROM main.sale_items WHERE sale_id=?1",
                        params![write.sale_id.as_slice()],
                        |r| r.get(0),
                    )
                    .map_err(|_| ())?;
                let position = match last {
                    None => 0,
                    Some(last) => match last.checked_add(1) {
                        Some(next) => next,
                        None => return Ok(Err(SaleError::PositionExhausted)),
                    },
                };
                if write.create_sale
                    && transaction
                        .execute(
                            "INSERT INTO main.sales(id,status) VALUES(?1,'open')",
                            params![write.sale_id.as_slice()],
                        )
                        .map_err(|_| ())?
                        != 1
                {
                    return Err(());
                }
                if transaction.execute(
                    "INSERT INTO main.sale_items(id,sale_id,position,product_id,description,quantity,unit_price_cents,discount_cents,total_cents) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                    params![
                        item.id.as_slice(),
                        write.sale_id.as_slice(),
                        position,
                        item.product_id.as_ref().map(|id| id.as_slice()),
                        &item.description,
                        item.quantity,
                        item.unit_price_cents,
                        item.discount_cents,
                        item.total_cents
                    ],
                ).map_err(|_| ())? != 1 {
                    return Err(());
                }
                append_audit_event(
                    transaction,
                    &AuditEvent {
                        event_id: write.event_id,
                        occurred_at_ms: write.occurred_at_ms,
                        operation: "sale_item_add",
                        entity: "sale_item",
                        entity_id: item.id,
                    },
                )
                .map_err(|_| ())?;
                Ok(Ok(item))
            },
        );
        match result {
            Ok((connection, outcome)) => {
                self.connection = Some(connection);
                outcome
            }
            Err(_) => Err(SaleError::StoreFailed),
        }
    }
}

fn load_sale_in_transaction(
    transaction: &Transaction<'_>,
    id: [u8; 16],
) -> Result<Option<Sale>, ()> {
    let status: Option<String> = transaction
        .query_row(
            "SELECT status FROM main.sales WHERE id=?1",
            params![id.as_slice()],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| ())?;
    let status = match status.as_deref() {
        Some("open") => SaleStatus::Open,
        Some("finalized") => SaleStatus::Finalized,
        None => return Ok(None),
        Some(_) => return Err(()),
    };
    let items = {
        let mut statement = transaction
            .prepare(
                "SELECT id,product_id,description,quantity,unit_price_cents,discount_cents,total_cents,position FROM main.sale_items WHERE sale_id=?1 ORDER BY position",
            )
            .map_err(|_| ())?;
        let rows = statement
            .query_map(params![id.as_slice()], |row| {
                if row.get::<_, i64>(7)? < 0 {
                    return Err(rusqlite::Error::InvalidQuery);
                }
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
            .map_err(|_| ())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|_| ())?
    };
    for item in &items {
        if item.calculate_total().map_err(|_| ())? != item.total_cents {
            return Err(());
        }
    }
    Ok(Some(Sale { id, status, items }))
}

fn finalize_sale_in_transaction(
    transaction: &Transaction<'_>,
    write: &FinalizeSaleWrite,
) -> Result<Result<FinalizedSale, SaleError>, ()> {
    let Some(sale) = load_sale_in_transaction(transaction, write.sale_id)? else {
        return Ok(Err(SaleError::SaleNotFound));
    };
    if !matches!(&sale.status, SaleStatus::Open) {
        return Ok(Err(SaleError::SaleNotOpen));
    }
    if sale.items.is_empty() {
        return Ok(Err(SaleError::EmptySale));
    }
    let total_cents = match sale.calculate_total() {
        Ok(total) => total,
        Err(_) => return Ok(Err(SaleError::InvalidSale)),
    };
    let paid_cents = match write.payments.iter().try_fold(0_i64, |total, payment| {
        if !matches!(&payment.movement.kind, MovementKind::Entry { .. })
            || payment.movement.validate_entry().is_err()
        {
            return None;
        }
        total.checked_add(payment.movement.amount_cents)
    }) {
        Some(total) => total,
        None => return Ok(Err(SaleError::InvalidPayment)),
    };
    if paid_cents > total_cents {
        return Ok(Err(SaleError::PaymentExceedsTotal));
    }
    let pending_cents = total_cents.checked_sub(paid_cents).ok_or(())?;
    match (pending_cents > 0, write.pending.is_some()) {
        (true, false) => return Ok(Err(SaleError::PendingTermsRequired)),
        (false, true) => return Ok(Err(SaleError::PendingTermsUnexpected)),
        _ => (),
    }

    if transaction
        .execute(
            "UPDATE main.sales SET status='finalized' WHERE id=?1 AND status='open'",
            params![write.sale_id.as_slice()],
        )
        .map_err(|_| ())?
        != 1
    {
        return Err(());
    }
    append_audit_event(
        transaction,
        &AuditEvent {
            event_id: write.sale_event_id,
            occurred_at_ms: write.occurred_at_ms,
            operation: "sale_finalize",
            entity: "sale",
            entity_id: write.sale_id,
        },
    )
    .map_err(|_| ())?;
    for payment in &write.payments {
        persist_entry_in_transaction(
            transaction,
            &payment.movement,
            payment.event_id,
            write.occurred_at_ms,
        )?;
    }
    if pending_cents > 0 {
        let terms = write.pending.as_ref().ok_or(())?;
        persist_receivable_in_transaction(
            transaction,
            &Receivable {
                id: terms.receivable_id,
                sale_id: write.sale_id,
                pending: SalePending {
                    amount_cents: pending_cents,
                    customer_id: terms.customer_id,
                    due_at_ms: terms.due_at_ms,
                },
            },
            terms.event_id,
            write.occurred_at_ms,
        )
        .map_err(|_| ())?;
    }
    Ok(Ok(FinalizedSale {
        sale_id: write.sale_id,
        total_cents,
        paid_cents,
        pending_cents,
    }))
}
impl SaleStore for AdapterSqliteSaleStore {
    fn persist_product_item_with_audit(
        &mut self,
        write: ProductItemWrite,
    ) -> Result<SaleItem, SaleError> {
        self.persist_item_with_audit(PersistSaleItem {
            sale_id: write.sale_id,
            create_sale: write.create_sale,
            item_id: write.item_id,
            source: ItemSource::Product(write.product_id),
            quantity: write.quantity,
            unit_price_cents: write.unit_price_cents,
            discount: write.discount,
            event_id: write.event_id,
            occurred_at_ms: write.occurred_at_ms,
        })
    }
    fn load_sale(&mut self, id: [u8; 16]) -> Result<Option<Sale>, SaleError> {
        let connection = self.connection.as_mut().ok_or(SaleError::StoreFailed)?;
        let transaction = connection
            .transaction()
            .map_err(|_| SaleError::StoreFailed)?;
        let sale =
            load_sale_in_transaction(&transaction, id).map_err(|_| SaleError::StoreFailed)?;
        transaction.commit().map_err(|_| SaleError::StoreFailed)?;
        Ok(sale)
    }
}

impl SaleFinalizationStore for AdapterSqliteSaleStore {
    fn finalize_sale_with_audit(
        &mut self,
        write: FinalizeSaleWrite,
    ) -> Result<FinalizedSale, SaleError> {
        let connection = self.connection.take().ok_or(SaleError::StoreFailed)?;
        let result = execute_write_transaction(connection, |transaction| {
            finalize_sale_in_transaction(transaction, &write)
        });
        match result {
            Ok((connection, outcome)) => {
                self.connection = Some(connection);
                outcome
            }
            Err(_) => Err(SaleError::StoreFailed),
        }
    }
}

impl FreeItemStore for AdapterSqliteSaleStore {
    fn persist_free_item_with_audit(
        &mut self,
        write: FreeItemWrite,
    ) -> Result<SaleItem, SaleError> {
        self.persist_item_with_audit(PersistSaleItem {
            sale_id: write.sale_id,
            create_sale: write.create_sale,
            item_id: write.item_id,
            source: ItemSource::Free(write.description),
            quantity: write.quantity,
            unit_price_cents: write.unit_price_cents,
            discount: write.discount,
            event_id: write.event_id,
            occurred_at_ms: write.occurred_at_ms,
        })
    }
}

impl SaleItemRemovalStore for AdapterSqliteSaleStore {
    fn remove_sale_item_with_audit(&mut self, write: RemoveSaleItemWrite) -> Result<(), SaleError> {
        let connection = self.connection.take().ok_or(SaleError::StoreFailed)?;
        let result = execute_write_transaction(
            connection,
            |transaction| -> Result<Result<(), SaleError>, ()> {
                let status: Option<String> = transaction
                    .query_row(
                        "SELECT status FROM main.sales WHERE id=?1",
                        params![write.sale_id.as_slice()],
                        |row| row.get(0),
                    )
                    .optional()
                    .map_err(|_| ())?;
                match status.as_deref() {
                    None => return Ok(Err(SaleError::SaleNotFound)),
                    Some("open") => (),
                    Some(_) => return Ok(Err(SaleError::SaleNotOpen)),
                }

                let item_exists: Option<i64> = transaction
                    .query_row(
                        "SELECT 1 FROM main.sale_items WHERE sale_id=?1 AND id=?2",
                        params![write.sale_id.as_slice(), write.item_id.as_slice()],
                        |row| row.get(0),
                    )
                    .optional()
                    .map_err(|_| ())?;
                if item_exists.is_none() {
                    return Ok(Err(SaleError::SaleItemNotFound));
                }
                if transaction
                    .execute(
                        "DELETE FROM main.sale_items WHERE sale_id=?1 AND id=?2",
                        params![write.sale_id.as_slice(), write.item_id.as_slice()],
                    )
                    .map_err(|_| ())?
                    != 1
                {
                    return Err(());
                }
                append_audit_event(
                    transaction,
                    &AuditEvent {
                        event_id: write.event_id,
                        occurred_at_ms: write.occurred_at_ms,
                        operation: "sale_item_remove",
                        entity: "sale_item",
                        entity_id: write.item_id,
                    },
                )
                .map_err(|_| ())?;
                Ok(Ok(()))
            },
        );
        match result {
            Ok((connection, outcome)) => {
                self.connection = Some(connection);
                outcome
            }
            Err(_) => Err(SaleError::StoreFailed),
        }
    }
}
