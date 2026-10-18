use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::Category;
use std::fmt;
use std::time::UNIX_EPOCH;

pub trait CategoryRenamingStore {
    type Error;

    fn rename_category_with_audit(
        &mut self,
        id: [u8; 16],
        name: &str,
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<Option<Category>, Self::Error>;
}

#[derive(PartialEq, Eq)]
pub enum RenameCategoryError {
    InvalidName,
    IdGenerationFailed,
    ClockFailed,
    NotFound,
    StoreFailed,
}

impl fmt::Debug for RenameCategoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidName => "InvalidName",
            Self::IdGenerationFailed => "IdGenerationFailed",
            Self::ClockFailed => "ClockFailed",
            Self::NotFound => "NotFound",
            Self::StoreFailed => "StoreFailed",
        })
    }
}

impl fmt::Display for RenameCategoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidName => "category name is invalid",
            Self::IdGenerationFailed => "category audit identifier could not be generated",
            Self::ClockFailed => "category timestamp is invalid",
            Self::NotFound => "category was not found",
            Self::StoreFailed => "category could not be renamed",
        })
    }
}

impl std::error::Error for RenameCategoryError {}

pub fn rename_category<S: CategoryRenamingStore, I: UuidGenerator, C: Clock>(
    id: [u8; 16],
    name: String,
    ids: &mut I,
    clock: &C,
    store: &mut S,
) -> Result<Category, RenameCategoryError> {
    let temp_category = Category {
        id,
        name,
        is_active: true,
    };
    temp_category
        .validate_name()
        .map_err(|_| RenameCategoryError::InvalidName)?;

    let event_id = ids
        .generate()
        .map_err(|_| RenameCategoryError::IdGenerationFailed)?;

    let occurred_at = clock
        .now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| RenameCategoryError::ClockFailed)?;
    let occurred_at_ms = duration_to_epoch_millis(occurred_at)?;

    let result = store
        .rename_category_with_audit(id, &temp_category.name, event_id, occurred_at_ms)
        .map_err(|_| RenameCategoryError::StoreFailed)?;

    match result {
        Some(category) => Ok(category),
        None => Err(RenameCategoryError::NotFound),
    }
}

fn duration_to_epoch_millis(duration: std::time::Duration) -> Result<i64, RenameCategoryError> {
    i64::try_from(duration.as_millis()).map_err(|_| RenameCategoryError::ClockFailed)
}

#[cfg(test)]
#[path = "rename_category_tests.rs"]
mod tests;
