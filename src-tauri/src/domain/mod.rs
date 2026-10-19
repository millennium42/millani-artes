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

    pub fn balance_cents(&self, movements: &[Movement]) -> Result<i64> {
        let mut balance = i128::from(self.initial_balance_cents);

        for m in movements {
            let delta = match &m.kind {
                MovementKind::Entry { account_id } => {
                    m.validate_entry()?;
                    if account_id == &self.id {
                        i128::from(m.amount_cents)
                    } else {
                        0
                    }
                }
                MovementKind::Exit { account_id } => {
                    m.validate_exit()?;
                    if account_id == &self.id {
                        -i128::from(m.amount_cents)
                    } else {
                        0
                    }
                }
                MovementKind::Transfer {
                    source_account_id,
                    destination_account_id,
                } => {
                    m.validate_transfer()?;
                    if source_account_id == &self.id {
                        -i128::from(m.amount_cents)
                    } else if destination_account_id == &self.id {
                        i128::from(m.amount_cents)
                    } else {
                        0
                    }
                }
            };
            balance = balance
                .checked_add(delta)
                .ok_or(DomainError::InvalidInput)?;
        }

        i64::try_from(balance).map_err(|_| DomainError::InvalidInput)
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

/// The financial state of a recurrence obligation.
///
/// ```compile_fail
/// fn amount(state: &millani_artes::domain::ObligationState) -> i64 {
///     state.amount_cents
/// }
/// ```
///
/// ```compile_fail
/// fn require_debug<T: std::fmt::Debug>() {}
/// require_debug::<millani_artes::domain::ObligationState>();
/// ```
///
/// ```compile_fail
/// fn require_display<T: std::fmt::Display>() {}
/// require_display::<millani_artes::domain::ObligationState>();
/// ```
pub enum ObligationState {
    AwaitingValue,
    Payable {
        amount_cents: i64,
    },
    Paid {
        amount_cents: i64,
        movement_id: [u8; 16],
    },
}

impl ObligationState {
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::AwaitingValue => Ok(()),
            Self::Payable { amount_cents } => {
                if *amount_cents > 0 {
                    Ok(())
                } else {
                    Err(DomainError::InvalidInput)
                }
            }
            Self::Paid {
                amount_cents,
                movement_id,
            } => {
                if *amount_cents > 0 && *movement_id != [0; 16] {
                    Ok(())
                } else {
                    Err(DomainError::InvalidInput)
                }
            }
        }
    }

    pub fn mark_paid(&self, payment: &Movement) -> Result<Self> {
        self.validate()?;
        let amount_cents = match self {
            Self::Payable { amount_cents } => *amount_cents,
            _ => return Err(DomainError::InvalidInput),
        };
        payment.validate_exit()?;
        if payment.amount_cents != amount_cents {
            return Err(DomainError::InvalidInput);
        }
        let paid = Self::Paid {
            amount_cents,
            movement_id: payment.id,
        };
        paid.validate()?;
        Ok(paid)
    }
}

/// Monthly variable recurrence configuration; amount is supplied per obligation.
///
/// ```compile_fail
/// fn require_debug<T: std::fmt::Debug>() {}
/// require_debug::<millani_artes::domain::VariableRecurrence>();
/// ```
///
/// ```compile_fail
/// fn require_display<T: std::fmt::Display>() {}
/// require_display::<millani_artes::domain::VariableRecurrence>();
/// ```
///
/// ```compile_fail
/// fn preset_amount(rule: &millani_artes::domain::VariableRecurrence) -> i64 {
///     rule.amount_cents
/// }
/// ```
pub struct VariableRecurrence {
    pub id: [u8; 16],
    pub name: String,
    pub context: Context,
    pub due_day: u8,
}

impl VariableRecurrence {
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() || !(1..=31).contains(&self.due_day) {
            Err(DomainError::InvalidInput)
        } else {
            Ok(())
        }
    }

    pub fn initial_obligation_state(&self) -> Result<ObligationState> {
        self.validate()?;
        Ok(ObligationState::AwaitingValue)
    }
}

/// Monthly fixed recurrence configuration; obligations and payments are separate operations.
///
/// ```compile_fail
/// fn require_debug<T: std::fmt::Debug>() {}
/// require_debug::<millani_artes::domain::FixedRecurrence>();
/// ```
///
/// ```compile_fail
/// fn require_display<T: std::fmt::Display>() {}
/// require_display::<millani_artes::domain::FixedRecurrence>();
/// ```
pub struct FixedRecurrence {
    pub id: [u8; 16],
    pub name: String,
    pub amount_cents: i64,
    pub context: Context,
    pub due_day: u8,
}

impl FixedRecurrence {
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty()
            || self.amount_cents <= 0
            || !(1..=31).contains(&self.due_day)
        {
            Err(DomainError::InvalidInput)
        } else {
            Ok(())
        }
    }

    pub fn initial_obligation_state(&self) -> Result<ObligationState> {
        self.validate()?;
        Ok(ObligationState::Payable {
            amount_cents: self.amount_cents,
        })
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

#[cfg(test)]
mod balance_tests;

#[cfg(test)]
mod fixed_recurrence_tests;

#[cfg(test)]
mod variable_recurrence_tests;

#[cfg(test)]
mod awaiting_value_tests;

#[cfg(test)]
mod payable_state_tests;

#[cfg(test)]
mod paid_state_tests;
