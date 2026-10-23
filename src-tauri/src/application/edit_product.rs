use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::Product;
use std::fmt;
use std::time::UNIX_EPOCH;

pub trait ProductEditingStore {
    type Error;

    fn edit_product_with_audit(
        &mut self,
        id: [u8; 16],
        name: &str,
        description: &str,
        suggested_price_cents: i64,
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<Option<Product>, Self::Error>;
}

#[derive(PartialEq, Eq)]
pub enum EditProductError {
    InvalidName,
    InvalidSuggestedPrice,
    IdGenerationFailed,
    ClockFailed,
    NotFound,
    StoreFailed,
}

impl fmt::Debug for EditProductError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidName => "InvalidName",
            Self::InvalidSuggestedPrice => "InvalidSuggestedPrice",
            Self::IdGenerationFailed => "IdGenerationFailed",
            Self::ClockFailed => "ClockFailed",
            Self::NotFound => "NotFound",
            Self::StoreFailed => "StoreFailed",
        })
    }
}

impl fmt::Display for EditProductError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidName => "product name is invalid",
            Self::InvalidSuggestedPrice => "product suggested price is invalid",
            Self::IdGenerationFailed => "product audit identifier could not be generated",
            Self::ClockFailed => "product timestamp is invalid",
            Self::NotFound => "product was not found",
            Self::StoreFailed => "product could not be edited",
        })
    }
}

impl std::error::Error for EditProductError {}

pub fn edit_product<S: ProductEditingStore, I: UuidGenerator, C: Clock>(
    id: [u8; 16],
    name: String,
    description: String,
    suggested_price_cents: i64,
    ids: &mut I,
    clock: &C,
    store: &mut S,
) -> Result<Product, EditProductError> {
    let temp_product = Product {
        id,
        name,
        description,
        suggested_price_cents,
        is_active: true,
    };
    temp_product
        .validate_name()
        .map_err(|_| EditProductError::InvalidName)?;
    temp_product
        .validate_suggested_price()
        .map_err(|_| EditProductError::InvalidSuggestedPrice)?;

    let event_id = ids
        .generate()
        .map_err(|_| EditProductError::IdGenerationFailed)?;
    let occurred_at = clock
        .now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| EditProductError::ClockFailed)?;
    let occurred_at_ms = duration_to_epoch_millis(occurred_at)?;

    let result = store
        .edit_product_with_audit(
            id,
            &temp_product.name,
            &temp_product.description,
            temp_product.suggested_price_cents,
            event_id,
            occurred_at_ms,
        )
        .map_err(|_| EditProductError::StoreFailed)?;
    match result {
        Some(product) => Ok(product),
        None => Err(EditProductError::NotFound),
    }
}

fn duration_to_epoch_millis(duration: std::time::Duration) -> Result<i64, EditProductError> {
    i64::try_from(duration.as_millis()).map_err(|_| EditProductError::ClockFailed)
}

#[cfg(test)]
#[path = "edit_product_tests.rs"]
mod tests;
