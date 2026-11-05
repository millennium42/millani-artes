use super::{list_today_exit_cents, ListTodayExitsError, TodayExitReadStore};
use crate::domain::ports::Clock;
use std::cell::{Cell, RefCell};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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

impl TodayExitReadStore for FakeStore {
    type Error = &'static str;

    fn read_exit_amounts_created_between(
        &self,
        start_ms: i64,
        end_ms: i64,
    ) -> Result<Vec<i64>, Self::Error> {
        self.calls.set(self.calls.get() + 1);
        self.range.set(Some((start_ms, end_ms)));
        self.result.borrow_mut().take().expect("one store read")
    }
}

fn assert_error(result: Result<i64, ListTodayExitsError>) {
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("expected opaque dashboard error"),
    };
    assert_eq!(format!("{error:?}"), "ListTodayExitsError");
    assert_eq!(format!("{error}"), "today's exits could not be read");
    assert!(std::error::Error::source(&error).is_none());
}

#[test]
fn list_reads_clock_and_store_once_and_forwards_the_fixed_brasilia_window() {
    let clock = TestClock::at(UNIX_EPOCH + Duration::from_millis(10_800_000));
    let store = FakeStore::new(Ok(vec![125, 300]));

    assert_eq!(list_today_exit_cents(&clock, &store).unwrap(), 425);

    assert_eq!(clock.calls.get(), 1);
    assert_eq!(store.calls.get(), 1);
    assert_eq!(store.range.get(), Some((10_800_000, 97_200_000)));
}

#[test]
fn empty_store_result_projects_zero() {
    let clock = TestClock::at(UNIX_EPOCH + Duration::from_millis(10_800_000));
    let store = FakeStore::new(Ok(vec![]));

    assert_eq!(list_today_exit_cents(&clock, &store).unwrap(), 0);
    assert_eq!(store.calls.get(), 1);
}

#[test]
fn clock_before_epoch_fails_before_reading_store() {
    let clock = TestClock::at(UNIX_EPOCH - Duration::from_millis(1));
    let store = FakeStore::new(Ok(vec![1]));

    assert_error(list_today_exit_cents(&clock, &store));
    assert_eq!(clock.calls.get(), 1);
    assert_eq!(store.calls.get(), 0);
}

#[test]
fn store_failures_are_opaque() {
    let clock = TestClock::at(UNIX_EPOCH + Duration::from_millis(10_800_000));
    let store = FakeStore::new(Err("private SQL detail"));

    assert_error(list_today_exit_cents(&clock, &store));
    assert_eq!(store.calls.get(), 1);
}

#[test]
fn invalid_or_unrepresentable_exits_return_no_partial_total() {
    for amounts in [vec![0], vec![-1], vec![i64::MAX, 1]] {
        let clock = TestClock::at(UNIX_EPOCH + Duration::from_millis(10_800_000));
        let store = FakeStore::new(Ok(amounts));

        assert_error(list_today_exit_cents(&clock, &store));
        assert_eq!(store.calls.get(), 1);
    }
}
