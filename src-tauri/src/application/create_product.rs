use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::Product;
use std::fmt;
use std::time::UNIX_EPOCH;

pub trait ProductCreationStore {
    type Error;

    fn persist_product_with_audit(
        &mut self,
        product: &Product,
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<(), Self::Error>;
}

#[derive(PartialEq, Eq)]
pub enum CreateProductError {
    InvalidName,
    InvalidSuggestedPrice,
    IdGenerationFailed,
    ClockFailed,
    StoreFailed,
}

impl fmt::Debug for CreateProductError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidName => "InvalidName",
            Self::InvalidSuggestedPrice => "InvalidSuggestedPrice",
            Self::IdGenerationFailed => "IdGenerationFailed",
            Self::ClockFailed => "ClockFailed",
            Self::StoreFailed => "StoreFailed",
        })
    }
}

impl fmt::Display for CreateProductError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidName => "product name is invalid",
            Self::InvalidSuggestedPrice => "product suggested price is invalid",
            Self::IdGenerationFailed => "product identifiers could not be generated",
            Self::ClockFailed => "product timestamp is invalid",
            Self::StoreFailed => "product could not be stored",
        })
    }
}

impl std::error::Error for CreateProductError {}

pub fn create_product<S: ProductCreationStore, I: UuidGenerator, C: Clock>(
    name: String,
    description: String,
    suggested_price_cents: i64,
    ids: &mut I,
    clock: &C,
    store: &mut S,
) -> Result<Product, CreateProductError> {
    let mut product = Product {
        id: [0; 16],
        name,
        description,
        suggested_price_cents,
        is_active: true,
    };
    product
        .validate_name()
        .map_err(|_| CreateProductError::InvalidName)?;
    product
        .validate_suggested_price()
        .map_err(|_| CreateProductError::InvalidSuggestedPrice)?;

    product.id = ids
        .generate()
        .map_err(|_| CreateProductError::IdGenerationFailed)?;
    let event_id = ids
        .generate()
        .map_err(|_| CreateProductError::IdGenerationFailed)?;
    let occurred_at = clock
        .now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| CreateProductError::ClockFailed)?;
    let occurred_at_ms = duration_to_epoch_millis(occurred_at)?;

    store
        .persist_product_with_audit(&product, event_id, occurred_at_ms)
        .map_err(|_| CreateProductError::StoreFailed)?;
    Ok(product)
}

fn duration_to_epoch_millis(duration: std::time::Duration) -> Result<i64, CreateProductError> {
    i64::try_from(duration.as_millis()).map_err(|_| CreateProductError::ClockFailed)
}

#[cfg(test)]
#[path = "create_product_tests.rs"]
mod tests;
