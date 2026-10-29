use super::add_product_item::SaleError;
use crate::domain::ports::{Clock, UuidGenerator};
use std::time::UNIX_EPOCH;

pub struct RemoveSaleItemInput {
    pub sale_id: [u8; 16],
    pub item_id: [u8; 16],
}

pub struct RemoveSaleItemWrite {
    pub sale_id: [u8; 16],
    pub item_id: [u8; 16],
    pub event_id: [u8; 16],
    pub occurred_at_ms: i64,
}

pub trait SaleItemRemovalStore {
    fn remove_sale_item_with_audit(&mut self, write: RemoveSaleItemWrite) -> Result<(), SaleError>;
}

pub fn remove_sale_item<S: SaleItemRemovalStore, I: UuidGenerator, C: Clock>(
    input: RemoveSaleItemInput,
    ids: &mut I,
    clock: &C,
    store: &mut S,
) -> Result<(), SaleError> {
    let event_id = ids.generate().map_err(|_| SaleError::IdGenerationFailed)?;
    let occurred_at = clock
        .now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| SaleError::ClockFailed)?;
    let occurred_at_ms = duration_to_epoch_millis(occurred_at)?;
    store.remove_sale_item_with_audit(RemoveSaleItemWrite {
        sale_id: input.sale_id,
        item_id: input.item_id,
        event_id,
        occurred_at_ms,
    })
}

fn duration_to_epoch_millis(duration: std::time::Duration) -> Result<i64, SaleError> {
    i64::try_from(duration.as_millis()).map_err(|_| SaleError::ClockFailed)
}

#[cfg(test)]
#[path = "remove_sale_item_tests.rs"]
mod tests;
