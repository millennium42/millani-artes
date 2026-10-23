use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::Product;
use std::fmt;
use std::time::UNIX_EPOCH;

pub trait ProductDeactivationStore {
    type Error;

    fn deactivate_product_with_audit(
        &mut self,
        id: [u8; 16],
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<Option<Product>, Self::Error>;
}

#[derive(PartialEq, Eq)]
pub enum DeactivateProductError {
    IdGenerationFailed,
    ClockFailed,
    NotFound,
    StoreFailed,
}

impl fmt::Debug for DeactivateProductError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::IdGenerationFailed => "IdGenerationFailed",
            Self::ClockFailed => "ClockFailed",
            Self::NotFound => "NotFound",
            Self::StoreFailed => "StoreFailed",
        })
    }
}

impl fmt::Display for DeactivateProductError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::IdGenerationFailed => "product audit identifier could not be generated",
            Self::ClockFailed => "product timestamp is invalid",
            Self::NotFound => "product was not found",
            Self::StoreFailed => "product could not be deactivated",
        })
    }
}

impl std::error::Error for DeactivateProductError {}

pub fn deactivate_product<S: ProductDeactivationStore, I: UuidGenerator, C: Clock>(
    id: [u8; 16],
    ids: &mut I,
    clock: &C,
    store: &mut S,
) -> Result<Product, DeactivateProductError> {
    let event_id = ids
        .generate()
        .map_err(|_| DeactivateProductError::IdGenerationFailed)?;

    let occurred_at = clock
        .now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| DeactivateProductError::ClockFailed)?;
    let occurred_at_ms = duration_to_epoch_millis(occurred_at)?;

    let result = store
        .deactivate_product_with_audit(id, event_id, occurred_at_ms)
        .map_err(|_| DeactivateProductError::StoreFailed)?;

    match result {
        Some(product) => Ok(product),
        None => Err(DeactivateProductError::NotFound),
    }
}

fn duration_to_epoch_millis(duration: std::time::Duration) -> Result<i64, DeactivateProductError> {
    i64::try_from(duration.as_millis()).map_err(|_| DeactivateProductError::ClockFailed)
}

#[cfg(test)]
#[path = "deactivate_product_tests.rs"]
mod tests;
