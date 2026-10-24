use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::Customer;
use std::fmt;
use std::time::UNIX_EPOCH;

pub trait CustomerCreationStore {
    type Error;

    fn persist_customer_with_audit(
        &mut self,
        customer: &Customer,
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<(), Self::Error>;
}

#[derive(PartialEq, Eq)]
pub enum CreateCustomerError {
    InvalidName,
    InvalidPhone,
    IdGenerationFailed,
    ClockFailed,
    StoreFailed,
}

impl fmt::Debug for CreateCustomerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidName => "InvalidName",
            Self::InvalidPhone => "InvalidPhone",
            Self::IdGenerationFailed => "IdGenerationFailed",
            Self::ClockFailed => "ClockFailed",
            Self::StoreFailed => "StoreFailed",
        })
    }
}

impl fmt::Display for CreateCustomerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidName => "customer name is invalid",
            Self::InvalidPhone => "customer phone is invalid",
            Self::IdGenerationFailed => "customer identifiers could not be generated",
            Self::ClockFailed => "customer timestamp is invalid",
            Self::StoreFailed => "customer could not be stored",
        })
    }
}

impl std::error::Error for CreateCustomerError {}

pub fn create_customer<S: CustomerCreationStore, I: UuidGenerator, C: Clock>(
    name: String,
    phone: Option<String>,
    ids: &mut I,
    clock: &C,
    store: &mut S,
) -> Result<Customer, CreateCustomerError> {
    let mut customer = Customer {
        id: [0; 16],
        name,
        phone,
    };
    customer
        .validate_name()
        .map_err(|_| CreateCustomerError::InvalidName)?;
    customer
        .validate_phone()
        .map_err(|_| CreateCustomerError::InvalidPhone)?;

    customer.id = ids
        .generate()
        .map_err(|_| CreateCustomerError::IdGenerationFailed)?;
    let event_id = ids
        .generate()
        .map_err(|_| CreateCustomerError::IdGenerationFailed)?;
    let occurred_at = clock
        .now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| CreateCustomerError::ClockFailed)?;
    let occurred_at_ms = duration_to_epoch_millis(occurred_at)?;

    store
        .persist_customer_with_audit(&customer, event_id, occurred_at_ms)
        .map_err(|_| CreateCustomerError::StoreFailed)?;
    Ok(customer)
}

fn duration_to_epoch_millis(duration: std::time::Duration) -> Result<i64, CreateCustomerError> {
    i64::try_from(duration.as_millis()).map_err(|_| CreateCustomerError::ClockFailed)
}

#[cfg(test)]
#[path = "create_customer_tests.rs"]
mod tests;
