//! Pure business rules belong here.

pub mod ports;

/// A Casa/Fábrica classification for movements, separate from an account.
pub enum Context {
    Home,
    Factory,
}

impl std::convert::TryFrom<&str> for Context {
    type Error = DomainError;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "home" => Ok(Self::Home),
            "factory" => Ok(Self::Factory),
            _ => Err(DomainError::InvalidInput),
        }
    }
}

pub enum AccountType {
    Cash,
    BankPix,
    Card,
    Other,
}

/// Account identity metadata and opening balance in cents.
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
    pub account_type: AccountType,
    pub initial_balance_cents: i64,
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

/// Category identity and active-state metadata.
///
/// ```compile_fail
/// fn require_debug<T: std::fmt::Debug>() {}
/// require_debug::<millani_artes::domain::Category>();
/// ```
///
/// ```compile_fail
/// fn require_display<T: std::fmt::Display>() {}
/// require_display::<millani_artes::domain::Category>();
/// ```
pub struct Category {
    pub id: [u8; 16],
    pub name: String,
    pub is_active: bool,
}

impl Category {
    pub fn validate_name(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            Err(DomainError::InvalidInput)
        } else {
            Ok(())
        }
    }
}

/// The account association carried by a movement.
///
/// ```compile_fail
/// fn require_debug<T: std::fmt::Debug>() {}
/// require_debug::<millani_artes::domain::MovementKind>();
/// ```
///
/// ```compile_fail
/// fn require_display<T: std::fmt::Display>() {}
/// require_display::<millani_artes::domain::MovementKind>();
/// ```
pub enum MovementKind {
    Entry {
        account_id: [u8; 16],
    },
    Exit {
        account_id: [u8; 16],
    },
    Transfer {
        source_account_id: [u8; 16],
        destination_account_id: [u8; 16],
    },
}

/// Identity, kind, raw amount in cents, and context for a movement.
///
/// ```compile_fail
/// fn require_debug<T: std::fmt::Debug>() {}
/// require_debug::<millani_artes::domain::Movement>();
/// ```
///
/// ```compile_fail
/// fn require_display<T: std::fmt::Display>() {}
/// require_display::<millani_artes::domain::Movement>();
/// ```
pub struct Movement {
    pub id: [u8; 16],
    pub kind: MovementKind,
    pub amount_cents: i64,
    pub context: Context,
}

impl Movement {
    pub fn validate_entry(&self) -> Result<()> {
        match &self.kind {
            MovementKind::Entry { .. } if self.amount_cents > 0 => Ok(()),
            _ => Err(DomainError::InvalidInput),
        }
    }

    pub fn validate_exit(&self) -> Result<()> {
        match &self.kind {
            MovementKind::Exit { .. } if self.amount_cents > 0 => Ok(()),
            _ => Err(DomainError::InvalidInput),
        }
    }

    pub fn validate_transfer(&self) -> Result<()> {
        match &self.kind {
            MovementKind::Transfer {
                source_account_id,
                destination_account_id,
            } if self.amount_cents > 0 && source_account_id != destination_account_id => Ok(()),
            _ => Err(DomainError::InvalidInput),
        }
    }

    pub fn result_effect_cents(&self) -> Result<i64> {
        match &self.kind {
            MovementKind::Entry { .. } => {
                self.validate_entry()?;
                Ok(self.amount_cents)
            }
            MovementKind::Exit { .. } => {
                self.validate_exit()?;
                Ok(-self.amount_cents)
            }
            MovementKind::Transfer { .. } => {
                self.validate_transfer()?;
                Ok(0)
            }
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

#[cfg(test)]
mod entry_tests;

#[cfg(test)]
mod exit_tests;

#[cfg(test)]
mod transfer_tests;

#[cfg(test)]
mod result_effect_tests;
