use super::add_product_item::SaleError;
use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::{DiscountInput, SaleItem};
use std::time::UNIX_EPOCH;

pub struct FreeItemInput {
    pub sale_id: Option<[u8; 16]>,
    pub description: String,
    pub quantity: i64,
    pub unit_price_cents: i64,
    pub discount: DiscountInput,
}

pub struct FreeItemWrite {
    pub sale_id: [u8; 16],
    pub create_sale: bool,
    pub item_id: [u8; 16],
    pub description: String,
    pub quantity: i64,
    pub unit_price_cents: i64,
    pub discount: DiscountInput,
    pub event_id: [u8; 16],
    pub occurred_at_ms: i64,
}

pub struct AddedFreeItem {
    pub sale_id: [u8; 16],
    pub item: SaleItem,
}

pub trait FreeItemStore {
    fn persist_free_item_with_audit(&mut self, write: FreeItemWrite)
        -> Result<SaleItem, SaleError>;
}

pub fn add_free_item<S: FreeItemStore, I: UuidGenerator, C: Clock>(
    input: FreeItemInput,
    ids: &mut I,
    clock: &C,
    store: &mut S,
) -> Result<AddedFreeItem, SaleError> {
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
    let item = store.persist_free_item_with_audit(FreeItemWrite {
        sale_id,
        create_sale,
        item_id,
        description: input.description,
        quantity: input.quantity,
        unit_price_cents: input.unit_price_cents,
        discount: input.discount,
        event_id,
        occurred_at_ms,
    })?;
    Ok(AddedFreeItem { sale_id, item })
}

fn duration_to_epoch_millis(duration: std::time::Duration) -> Result<i64, SaleError> {
    i64::try_from(duration.as_millis()).map_err(|_| SaleError::ClockFailed)
}

#[cfg(test)]
#[path = "add_free_item_tests.rs"]
mod tests;
