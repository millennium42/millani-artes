use super::{list_month_sale_cents, ListMonthSalesError, MonthSaleReadStore};
use crate::domain::ports::Clock;
use std::cell::{Cell, RefCell};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const JANUARY_2024_START_MS: i64 = 1_704_078_000_000;
const FEBRUARY_2024_START_MS: i64 = 1_706_756_400_000;

struct TestClock {
    now: SystemTime,
    calls: Cell<usize>,
}

impl TestClock {
    fn at(now: SystemTime) -> Self {
        Self {
            now,
            calls: Cell::new(0),
        }
    }
}

impl Clock for TestClock {
    fn now(&self) -> SystemTime {
        self.calls.set(self.calls.get() + 1);
        self.now
    }
}

struct FakeStore {
    result: RefCell<Option<Result<Vec<i64>, &'static str>>>,
    calls: Cell<usize>,
    range: Cell<Option<(i64, i64)>>,
}

impl FakeStore {
    fn new(result: Result<Vec<i64>, &'static str>) -> Self {
        Self {
            result: RefCell::new(Some(result)),
            calls: Cell::new(0),
            range: Cell::new(None),
        }
    }
}

impl MonthSaleReadStore for FakeStore {
    type Error = &'static str;

    fn read_finalized_sale_item_amounts_created_between(
        &self,
        start_ms: i64,
        end_ms: i64,
    ) -> Result<Vec<i64>, Self::Error> {
        self.calls.set(self.calls.get() + 1);
        self.range.set(Some((start_ms, end_ms)));
        self.result.borrow_mut().take().expect("one store read")
    }
}

fn assert_error(result: Result<i64, ListMonthSalesError>) {
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("expected opaque dashboard error"),
    };
    assert_eq!(format!("{error:?}"), "ListMonthSalesError");
    assert_eq!(format!("{error}"), "month's sales could not be read");
    assert!(std::error::Error::source(&error).is_none());
}

#[test]
fn list_reads_clock_and_store_once_and_forwards_month_window() {
    let clock = TestClock::at(UNIX_EPOCH + Duration::from_millis(JANUARY_2024_START_MS as u64));
    let store = FakeStore::new(Ok(vec![0, 125, 300]));

    assert_eq!(list_month_sale_cents(&clock, &store).unwrap(), 425);
    assert_eq!(clock.calls.get(), 1);
    assert_eq!(store.calls.get(), 1);
    assert_eq!(
        store.range.get(),
        Some((JANUARY_2024_START_MS, FEBRUARY_2024_START_MS))
    );
}

#[test]
fn empty_month_projects_zero() {
    let clock = TestClock::at(UNIX_EPOCH + Duration::from_millis(JANUARY_2024_START_MS as u64));
    let store = FakeStore::new(Ok(vec![]));

    assert_eq!(list_month_sale_cents(&clock, &store).unwrap(), 0);
}

#[test]
fn invalid_clock_fails_before_store_read() {
    let clock = TestClock::at(UNIX_EPOCH - Duration::from_millis(1));
    let store = FakeStore::new(Ok(vec![1]));

    assert_error(list_month_sale_cents(&clock, &store));
    assert_eq!(clock.calls.get(), 1);
    assert_eq!(store.calls.get(), 0);
}

#[test]
fn store_and_invalid_amount_errors_are_opaque_without_partial_total() {
    let clock = TestClock::at(UNIX_EPOCH + Duration::from_millis(JANUARY_2024_START_MS as u64));
    let store = FakeStore::new(Err("private SQL detail"));
    assert_error(list_month_sale_cents(&clock, &store));

    for amounts in [vec![-1], vec![i64::MAX, 1]] {
        let clock = TestClock::at(UNIX_EPOCH + Duration::from_millis(JANUARY_2024_START_MS as u64));
        let store = FakeStore::new(Ok(amounts));
        assert_error(list_month_sale_cents(&clock, &store));
    }
}
