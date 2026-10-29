use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::{DiscountInput, Sale, SaleItem};
use std::{fmt, time::UNIX_EPOCH};

pub struct ProductItemInput {
    pub sale_id: Option<[u8; 16]>,
    pub product_id: [u8; 16],
    pub quantity: i64,
    pub unit_price_cents: i64,
    pub discount: DiscountInput,
}

pub struct ProductItemWrite {
    pub sale_id: [u8; 16],
    pub create_sale: bool,
    pub item_id: [u8; 16],
    pub product_id: [u8; 16],
    pub quantity: i64,
    pub unit_price_cents: i64,
    pub discount: DiscountInput,
    pub event_id: [u8; 16],
    pub occurred_at_ms: i64,
}

pub struct AddedProductItem {
    pub sale_id: [u8; 16],
    pub item: SaleItem,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SaleError {
    IdGenerationFailed,
    ClockFailed,
    SaleNotFound,
    SaleNotOpen,
    SaleItemNotFound,
    ProductNotFound,
    InvalidItem,
    PositionExhausted,
    StoreFailed,
}

impl fmt::Display for SaleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("sale operation failed")
    }
}
impl std::error::Error for SaleError {}

pub trait SaleStore {
    fn persist_product_item_with_audit(
        &mut self,
        write: ProductItemWrite,
    ) -> Result<SaleItem, SaleError>;
    fn load_sale(&mut self, id: [u8; 16]) -> Result<Option<Sale>, SaleError>;
}

pub fn add_product_item<S: SaleStore, I: UuidGenerator, C: Clock>(
    input: ProductItemInput,
    ids: &mut I,
    clock: &C,
    store: &mut S,
) -> Result<AddedProductItem, SaleError> {
    let create_sale = input.sale_id.is_none();
    let sale_id = match input.sale_id {
        Some(id) => id,
        None => ids.generate().map_err(|_| SaleError::IdGenerationFailed)?,
    };
    let item_id = ids.generate().map_err(|_| SaleError::IdGenerationFailed)?;
    let event_id = ids.generate().map_err(|_| SaleError::IdGenerationFailed)?;
    let occurred_at = clock
        .now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| SaleError::ClockFailed)?;
    let occurred_at_ms = duration_to_epoch_millis(occurred_at)?;
    let item = store.persist_product_item_with_audit(ProductItemWrite {
        sale_id,
        create_sale,
        item_id,
        product_id: input.product_id,
        quantity: input.quantity,
        unit_price_cents: input.unit_price_cents,
        discount: input.discount,
        event_id,
        occurred_at_ms,
    })?;
    Ok(AddedProductItem { sale_id, item })
}

pub fn read_sale<S: SaleStore>(store: &mut S, id: [u8; 16]) -> Result<Option<Sale>, SaleError> {
    store.load_sale(id)
}

fn duration_to_epoch_millis(duration: std::time::Duration) -> Result<i64, SaleError> {
    i64::try_from(duration.as_millis()).map_err(|_| SaleError::ClockFailed)
}

#[cfg(test)]
#[path = "add_product_item_tests.rs"]
mod tests;
