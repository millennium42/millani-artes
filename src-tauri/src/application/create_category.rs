use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::Category;
use std::fmt;
use std::time::UNIX_EPOCH;

pub trait CategoryCreationStore {
    type Error;

    fn persist_category_with_audit(
        &mut self,
        category: &Category,
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<(), Self::Error>;
}

#[derive(PartialEq, Eq)]
pub enum CreateCategoryError {
    InvalidName,
    IdGenerationFailed,
    ClockFailed,
    StoreFailed,
}

impl fmt::Debug for CreateCategoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidName => "InvalidName",
            Self::IdGenerationFailed => "IdGenerationFailed",
            Self::ClockFailed => "ClockFailed",
            Self::StoreFailed => "StoreFailed",
        })
    }
}

impl fmt::Display for CreateCategoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidName => "category name is invalid",
            Self::IdGenerationFailed => "category identifiers could not be generated",
            Self::ClockFailed => "category timestamp is invalid",
            Self::StoreFailed => "category could not be stored",
        })
    }
}

impl std::error::Error for CreateCategoryError {}

pub fn create_category<S: CategoryCreationStore, I: UuidGenerator, C: Clock>(
    name: String,
    ids: &mut I,
    clock: &C,
    store: &mut S,
) -> Result<Category, CreateCategoryError> {
    let mut category = Category {
        id: [0; 16],
        name,
        is_active: true,
    };
    category
        .validate_name()
        .map_err(|_| CreateCategoryError::InvalidName)?;

    category.id = ids
        .generate()
        .map_err(|_| CreateCategoryError::IdGenerationFailed)?;
    let event_id = ids
        .generate()
        .map_err(|_| CreateCategoryError::IdGenerationFailed)?;
    let occurred_at = clock
        .now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| CreateCategoryError::ClockFailed)?;
    let occurred_at_ms = duration_to_epoch_millis(occurred_at)?;

    store
        .persist_category_with_audit(&category, event_id, occurred_at_ms)
        .map_err(|_| CreateCategoryError::StoreFailed)?;
    Ok(category)
}

fn duration_to_epoch_millis(duration: std::time::Duration) -> Result<i64, CreateCategoryError> {
    i64::try_from(duration.as_millis()).map_err(|_| CreateCategoryError::ClockFailed)
}

#[cfg(test)]
#[path = "create_category_tests.rs"]
mod tests;
