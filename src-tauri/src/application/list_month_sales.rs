use crate::domain::ports::Clock;
use crate::domain::{brasilia_month_window_ms, sum_nonnegative_sale_amounts_cents};
use std::time::UNIX_EPOCH;

#[derive(Debug)]
pub struct ListMonthSalesError;

impl std::fmt::Display for ListMonthSalesError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("month's sales could not be read")
    }
}

impl std::error::Error for ListMonthSalesError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub trait MonthSaleReadStore {
    type Error;

    fn read_finalized_sale_item_amounts_created_between(
        &self,
        start_ms: i64,
        end_ms: i64,
    ) -> Result<Vec<i64>, Self::Error>;
}

pub fn list_month_sale_cents<C: Clock, S: MonthSaleReadStore>(
    clock: &C,
    store: &S,
) -> Result<i64, ListMonthSalesError> {
    let now_ms = i64::try_from(
        clock
            .now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| ListMonthSalesError)?
            .as_millis(),
    )
    .map_err(|_| ListMonthSalesError)?;
    let (start_ms, end_ms) = brasilia_month_window_ms(now_ms).map_err(|_| ListMonthSalesError)?;
    let amounts = store
        .read_finalized_sale_item_amounts_created_between(start_ms, end_ms)
        .map_err(|_| ListMonthSalesError)?;

    sum_nonnegative_sale_amounts_cents(&amounts).map_err(|_| ListMonthSalesError)
}

#[cfg(test)]
#[path = "list_month_sales_tests.rs"]
mod tests;
