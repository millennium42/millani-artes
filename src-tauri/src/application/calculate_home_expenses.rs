use crate::application::list_movements::{list_movements, MovementReadStore};
use crate::domain::{Context, MovementKind};

#[derive(Debug)]
pub struct CalculateHomeExpensesError;

impl std::fmt::Display for CalculateHomeExpensesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "home expenses could not be calculated")
    }
}

impl std::error::Error for CalculateHomeExpensesError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub fn calculate_home_expenses(
    store: &impl MovementReadStore,
) -> Result<i64, CalculateHomeExpensesError> {
    let movements = list_movements(store).map_err(|_| CalculateHomeExpensesError)?;
    let mut total = 0_i128;

    for movement in movements {
        let effect = movement
            .result_effect_cents()
            .map_err(|_| CalculateHomeExpensesError)?;

        if matches!(
            (&movement.kind, &movement.context),
            (MovementKind::Exit { .. }, Context::Home)
        ) {
            let outflow = effect.checked_neg().ok_or(CalculateHomeExpensesError)?;
            total = total
                .checked_add(i128::from(outflow))
                .ok_or(CalculateHomeExpensesError)?;
        }
    }

    i64::try_from(total).map_err(|_| CalculateHomeExpensesError)
}

#[cfg(test)]
#[path = "calculate_home_expenses_tests.rs"]
mod tests;
