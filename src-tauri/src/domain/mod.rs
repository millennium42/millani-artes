//! Pure business rules belong here; financial rules are not implemented yet.

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
