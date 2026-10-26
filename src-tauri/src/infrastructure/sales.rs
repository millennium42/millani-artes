use crate::application::add_product_item::{ProductItemWrite, SaleError, SaleStore};
use crate::database::{append_audit_event, execute_write_transaction, AuditEvent};
use crate::domain::{create_product_item, Product, Sale, SaleItem, SaleStatus};
use rusqlite::{params, Connection, OptionalExtension};

pub struct AdapterSqliteSaleStore {
    connection: Option<Connection>,
}
impl AdapterSqliteSaleStore {
    pub fn new(connection: Connection) -> Self {
        Self {
            connection: Some(connection),
        }
    }
}
impl SaleStore for AdapterSqliteSaleStore {
    fn persist_product_item_with_audit(
        &mut self,
        write: ProductItemWrite,
    ) -> Result<SaleItem, SaleError> {
        let connection = self.connection.take().ok_or(SaleError::StoreFailed)?;
        let result = execute_write_transaction(
            connection,
            |transaction| -> Result<Result<SaleItem, SaleError>, ()> {
                // Business rejections happen before writes, so their empty transaction can commit.
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
                let product = transaction.query_row(
                "SELECT id,name,description,suggested_price_cents,is_active FROM main.products WHERE id=?1",
                params![write.product_id.as_slice()],
                |r| Ok(Product { id:r.get(0)?,name:r.get(1)?,description:r.get(2)?,suggested_price_cents:r.get(3)?,is_active:r.get(4)? }),
            ).optional().map_err(|_| ())?;
                let Some(product) = product else {
                    return Ok(Err(SaleError::ProductNotFound));
                };
                let item = match create_product_item(
                    write.item_id,
                    &product,
                    write.quantity,
                    write.unit_price_cents,
                    write.discount,
                ) {
                    Ok(item) => item,
                    Err(_) => return Ok(Err(SaleError::InvalidItem)),
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
                params![item.id.as_slice(),write.sale_id.as_slice(),position,item.product_id.as_ref().map(|id|id.as_slice()),&item.description,item.quantity,item.unit_price_cents,item.discount_cents,item.total_cents],
            ).map_err(|_| ())? != 1 { return Err(()); }
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
    fn load_sale(&mut self, id: [u8; 16]) -> Result<Option<Sale>, SaleError> {
        let connection = self.connection.as_mut().ok_or(SaleError::StoreFailed)?;
        let transaction = connection
            .transaction()
            .map_err(|_| SaleError::StoreFailed)?;
        let status: Option<String> = transaction
            .query_row(
                "SELECT status FROM main.sales WHERE id=?1",
                params![id.as_slice()],
                |r| r.get(0),
            )
            .optional()
            .map_err(|_| SaleError::StoreFailed)?;
        let status = match status.as_deref() {
            Some("open") => SaleStatus::Open,
            Some("finalized") => SaleStatus::Finalized,
            None => {
                transaction.commit().map_err(|_| SaleError::StoreFailed)?;
                return Ok(None);
            }
            Some(_) => return Err(SaleError::StoreFailed),
        };
        let items = {
            let mut statement=transaction.prepare(
                "SELECT id,product_id,description,quantity,unit_price_cents,discount_cents,total_cents,position FROM main.sale_items WHERE sale_id=?1 ORDER BY position",
            ).map_err(|_| SaleError::StoreFailed)?;
            let rows = statement
                .query_map(params![id.as_slice()], |r| {
                    if r.get::<_, i64>(7)? < 0 {
                        return Err(rusqlite::Error::InvalidQuery);
                    }
                    Ok(SaleItem {
                        id: r.get(0)?,
                        product_id: r.get(1)?,
                        description: r.get(2)?,
                        quantity: r.get(3)?,
                        unit_price_cents: r.get(4)?,
                        discount_cents: r.get(5)?,
                        total_cents: r.get(6)?,
                    })
                })
                .map_err(|_| SaleError::StoreFailed)?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|_| SaleError::StoreFailed)?
        };
        for item in &items {
            if item.calculate_total().map_err(|_| SaleError::StoreFailed)? != item.total_cents {
                return Err(SaleError::StoreFailed);
            }
        }
        transaction.commit().map_err(|_| SaleError::StoreFailed)?;
        Ok(Some(Sale { id, status, items }))
    }
}
