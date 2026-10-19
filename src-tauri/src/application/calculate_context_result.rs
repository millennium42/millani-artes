use crate::application::list_movements::{list_movements, MovementReadStore};
use crate::domain::Context;

#[derive(Debug)]
pub struct CalculateContextResultError;

impl std::fmt::Display for CalculateContextResultError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "context result could not be calculated")
    }
}

impl std::error::Error for CalculateContextResultError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub fn calculate_context_result(
    context: &Context,
    store: &impl MovementReadStore,
) -> Result<i64, CalculateContextResultError> {
    let movements = list_movements(store).map_err(|_| CalculateContextResultError)?;

    let mut result = 0_i128;

    for movement in movements {
        let delta = movement
            .result_effect_cents()
            .map_err(|_| CalculateContextResultError)?;

        if matches!(
            (&movement.context, context),
            (Context::Home, Context::Home) | (Context::Factory, Context::Factory)
        ) {
            result = result
                .checked_add(i128::from(delta))
                .ok_or(CalculateContextResultError)?;
        }
    }

    i64::try_from(result).map_err(|_| CalculateContextResultError)
}

#[cfg(test)]
#[path = "calculate_context_result_tests.rs"]
mod tests;
