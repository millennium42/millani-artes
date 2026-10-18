use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::Category;
use std::fmt;
use std::time::UNIX_EPOCH;

pub trait CategoryDeactivationStore {
    type Error;

    fn deactivate_category_with_audit(
        &mut self,
        id: [u8; 16],
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<Option<Category>, Self::Error>;
}

#[derive(PartialEq, Eq)]
pub enum DeactivateCategoryError {
    IdGenerationFailed,
    ClockFailed,
    NotFound,
    StoreFailed,
}

impl fmt::Debug for DeactivateCategoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::IdGenerationFailed => "IdGenerationFailed",
            Self::ClockFailed => "ClockFailed",
            Self::NotFound => "NotFound",
            Self::StoreFailed => "StoreFailed",
        })
    }
}

impl fmt::Display for DeactivateCategoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::IdGenerationFailed => "category audit identifier could not be generated",
            Self::ClockFailed => "category timestamp is invalid",
            Self::NotFound => "category was not found",
            Self::StoreFailed => "category could not be deactivated",
        })
    }
}

impl std::error::Error for DeactivateCategoryError {}

pub fn deactivate_category<S: CategoryDeactivationStore, I: UuidGenerator, C: Clock>(
    id: [u8; 16],
    ids: &mut I,
    clock: &C,
    store: &mut S,
) -> Result<Category, DeactivateCategoryError> {
    let event_id = ids
        .generate()
        .map_err(|_| DeactivateCategoryError::IdGenerationFailed)?;

    let occurred_at = clock
        .now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| DeactivateCategoryError::ClockFailed)?;
    let occurred_at_ms = duration_to_epoch_millis(occurred_at)?;

    let result = store
        .deactivate_category_with_audit(id, event_id, occurred_at_ms)
        .map_err(|_| DeactivateCategoryError::StoreFailed)?;

    match result {
        Some(category) => Ok(category),
        None => Err(DeactivateCategoryError::NotFound),
    }
}

fn duration_to_epoch_millis(duration: std::time::Duration) -> Result<i64, DeactivateCategoryError> {
    i64::try_from(duration.as_millis()).map_err(|_| DeactivateCategoryError::ClockFailed)
}

#[cfg(test)]
#[path = "deactivate_category_tests.rs"]
mod tests;
