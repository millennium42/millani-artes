//! Pure business rules belong here; financial rules are not implemented yet.

pub mod ports;

/// Account identity metadata; financial fields belong to later tasks.
///
/// ```compile_fail
/// fn require_debug<T: std::fmt::Debug>() {}
/// require_debug::<millani_artes::domain::Account>();
/// ```
///
/// ```compile_fail
/// fn require_display<T: std::fmt::Display>() {}
/// require_display::<millani_artes::domain::Account>();
/// ```
pub struct Account {
    pub id: [u8; 16],
    pub name: String,
}

impl Account {
    pub fn validate_name(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            Err(DomainError::InvalidInput)
        } else {
            Ok(())
        }
    }
}

pub type Result<T> = std::result::Result<T, DomainError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainError {
    InvalidInput,
}

impl std::fmt::Display for DomainError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("DOMAIN_INVALID_INPUT")
    }
}

impl std::error::Error for DomainError {}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod ports_tests;
