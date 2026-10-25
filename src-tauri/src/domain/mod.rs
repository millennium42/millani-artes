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
    Overdue {
        amount_cents: i64,
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
            Self::Overdue { amount_cents } => {
                if *amount_cents > 0 {
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
            Self::Overdue { amount_cents } => *amount_cents,
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

    pub fn mark_overdue(
        &self,
        due_at: std::time::SystemTime,
        clock: &dyn ports::Clock,
    ) -> Result<Self> {
        self.validate()?;
        let amount_cents = match self {
            Self::Payable { amount_cents } => *amount_cents,
            _ => return Err(DomainError::InvalidInput),
        };
        if clock.now() <= due_at {
            return Err(DomainError::InvalidInput);
        }
        Ok(Self::Overdue { amount_cents })
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

    pub fn generate_obligation(
        &self,
        id: [u8; 16],
        period: (u16, u8),
    ) -> Result<RecurrenceObligation> {
        let state = self.initial_obligation_state()?;
        let due_at_ms = recurrence_due_at_ms(period.0, period.1, self.due_day)?;
        Ok(RecurrenceObligation {
            id,
            recurrence_id: self.id,
            period_year: period.0,
            period_month: period.1,
            due_at_ms,
            state,
            context: match &self.context {
                Context::Home => Context::Home,
                Context::Factory => Context::Factory,
            },
        })
    }
}

/// Identity, period, due date, state, and context for a recurrence obligation.
///
/// ```compile_fail
/// fn require_debug<T: std::fmt::Debug>() {}
/// require_debug::<millani_artes::domain::RecurrenceObligation>();
/// ```
///
/// ```compile_fail
/// fn require_display<T: std::fmt::Display>() {}
/// require_display::<millani_artes::domain::RecurrenceObligation>();
/// ```
pub struct RecurrenceObligation {
    pub id: [u8; 16],
    pub recurrence_id: [u8; 16],
    pub period_year: u16,
    pub period_month: u8,
    pub due_at_ms: i64,
    pub state: ObligationState,
    pub context: Context,
}

fn recurrence_due_at_ms(year: u16, month: u8, due_day: u8) -> Result<i64> {
    if !(1..=9999).contains(&year) || !(1..=12).contains(&month) {
        return Err(DomainError::InvalidInput);
    }
    let year_i64 = i64::from(year);
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let months: [i64; 12] = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let month_idx = usize::from(month - 1);
    let day = i64::from(due_day).min(months[month_idx]);
    let y = year_i64 - 1;
    let days = 365 * y + y / 4 - y / 100 + y / 400 + months[..month_idx].iter().sum::<i64>() + day
        - 1
        - 719162;
    Ok((days + 1) * 86400000 + 10800000 - 1)
}

/// Reusable product template; sale items preserve their own snapshots.
///
/// ```compile_fail
/// fn require_debug<T: std::fmt::Debug>() {}
/// require_debug::<millani_artes::domain::Product>();
/// ```
///
/// ```compile_fail
/// fn require_display<T: std::fmt::Display>() {}
/// require_display::<millani_artes::domain::Product>();
/// ```
pub struct Product {
    pub id: [u8; 16],
    pub name: String,
    pub description: String,
    pub suggested_price_cents: i64,
    pub is_active: bool,
}

impl Product {
    pub fn validate_name(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            Err(DomainError::InvalidInput)
        } else {
            Ok(())
        }
    }

    pub fn validate_suggested_price(&self) -> Result<()> {
        if self.suggested_price_cents < 0 {
            Err(DomainError::InvalidInput)
        } else {
            Ok(())
        }
    }
}

/// Customer identity with optional phone metadata.
///
/// ```compile_fail
/// fn require_debug<T: std::fmt::Debug>() {}
/// require_debug::<millani_artes::domain::Customer>();
/// ```
///
/// ```compile_fail
/// fn require_display<T: std::fmt::Display>() {}
/// require_display::<millani_artes::domain::Customer>();
/// ```
pub struct Customer {
    pub id: [u8; 16],
    pub name: String,
    pub phone: Option<String>,
}

impl Customer {
    pub fn validate_name(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            Err(DomainError::InvalidInput)
        } else {
            Ok(())
        }
    }

    pub fn validate_phone(&self) -> Result<()> {
        let Some(phone) = &self.phone else {
            return Ok(());
        };
        let bytes = phone.as_bytes();
        if bytes.len() != 15
            || bytes[0] != b'('
            || bytes[3] != b')'
            || bytes[4] != b' '
            || bytes[5] != b'9'
            || bytes[10] != b'-'
            || !bytes[1..3]
                .iter()
                .chain(&bytes[5..10])
                .chain(&bytes[11..15])
                .all(u8::is_ascii_digit)
        {
            return Err(DomainError::InvalidInput);
        }
        let ddd = (bytes[1] - b'0') * 10 + (bytes[2] - b'0');
        if matches!(
            ddd,
            11..=19 | 21 | 22 | 24 | 27 | 28 | 31..=35 | 37 | 38 | 41..=49 | 51
                | 53..=55 | 61..=69 | 71 | 73..=75 | 77 | 79 | 81..=89 | 91..=99
        ) {
            Ok(())
        } else {
            Err(DomainError::InvalidInput)
        }
    }
}

/// Sale item values owned independently of the optional product template.
///
/// ```compile_fail
/// fn require_debug<T: std::fmt::Debug>() {}
/// require_debug::<millani_artes::domain::SaleItem>();
/// ```
///
/// ```compile_fail
/// fn require_display<T: std::fmt::Display>() {}
/// require_display::<millani_artes::domain::SaleItem>();
/// ```
pub struct SaleItem {
    pub id: [u8; 16],
    pub product_id: Option<[u8; 16]>,
    pub description: String,
    pub quantity: i64,
    pub unit_price_cents: i64,
    pub discount_cents: i64,
    pub total_cents: i64,
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

#[cfg(test)]
mod overdue_state_tests;

#[cfg(test)]
mod fixed_generation_tests;

mod variable_generation;
#[cfg(test)]
mod variable_generation_tests;

mod obligation_amount;
#[cfg(test)]
mod obligation_amount_tests;

mod overdue_epoch;
#[cfg(test)]
mod overdue_epoch_tests;

#[cfg(test)]
mod product_tests;

#[cfg(test)]
mod customer_tests;

#[cfg(test)]
mod sale_item_tests;
