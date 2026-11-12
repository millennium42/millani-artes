use super::{calculate_home_expenses, CalculateHomeExpensesError};
use crate::application::list_movements::MovementReadStore;
use crate::domain::{Context, Movement, MovementKind};
use std::cell::{Cell, RefCell};

struct FakeStore {
    result: RefCell<Option<Result<Vec<Movement>, &'static str>>>,
    calls: Cell<usize>,
}

impl FakeStore {
    fn new(result: Result<Vec<Movement>, &'static str>) -> Self {
        Self {
            result: RefCell::new(Some(result)),
            calls: Cell::new(0),
        }
    }
}

impl MovementReadStore for FakeStore {
    type Error = &'static str;

    fn read_all_movements(&self) -> Result<Vec<Movement>, Self::Error> {
        self.calls.set(self.calls.get() + 1);
        self.result.borrow_mut().take().expect("one port call")
    }
}

fn context(factory: bool) -> Context {
    if factory {
        Context::Factory
    } else {
        Context::Home
    }
}

fn movement(id: u8, kind: u8, amount: i64, factory: bool) -> Movement {
    Movement {
        id: [id; 16],
        amount_cents: amount,
        context: context(factory),
        kind: match kind {
            0 => MovementKind::Entry {
                account_id: [1; 16],
            },
            1 => MovementKind::Exit {
                account_id: [2; 16],
            },
            _ => MovementKind::Transfer {
                source_account_id: [1; 16],
                destination_account_id: [2; 16],
            },
        },
    }
}

fn assert_error(result: Result<i64, CalculateHomeExpensesError>) {
    let error = result.unwrap_err();
    assert_eq!(format!("{error:?}"), "CalculateHomeExpensesError");
    assert_eq!(format!("{error}"), "home expenses could not be calculated");
    assert!(std::error::Error::source(&error).is_none());
}

#[test]
fn empty_history_returns_zero_with_one_read() {
    let store = FakeStore::new(Ok(vec![]));
    assert_eq!(calculate_home_expenses(&store).unwrap(), 0);
    assert_eq!(store.calls.get(), 1);
}

#[test]
fn sums_only_home_exits_and_ignores_other_valid_movements() {
    let store = FakeStore::new(Ok(vec![
        movement(1, 0, 1_000, false),
        movement(2, 1, 40, false),
        movement(3, 1, 700, true),
        movement(4, 2, 300, false),
        movement(5, 1, 60, false),
    ]));
    assert_eq!(calculate_home_expenses(&store).unwrap(), 100);
    assert_eq!(store.calls.get(), 1);
}

#[test]
fn sum_is_checked_and_other_context_total_does_not_overflow_selected_context() {
    let maximum = FakeStore::new(Ok(vec![movement(1, 1, i64::MAX, false)]));
    assert_eq!(calculate_home_expenses(&maximum).unwrap(), i64::MAX);
    assert_eq!(maximum.calls.get(), 1);

    let overflow = FakeStore::new(Ok(vec![
        movement(1, 1, i64::MAX, false),
        movement(2, 1, 1, false),
    ]));
    assert_error(calculate_home_expenses(&overflow));
    assert_eq!(overflow.calls.get(), 1);

    let other_context = FakeStore::new(Ok(vec![
        movement(1, 1, 1, false),
        movement(2, 1, i64::MAX, true),
        movement(3, 1, i64::MAX, true),
    ]));
    assert_eq!(calculate_home_expenses(&other_context).unwrap(), 1);
    assert_eq!(other_context.calls.get(), 1);
}

#[test]
fn invalid_movements_are_rejected_before_context_filtering() {
    for factory in [false, true] {
        for kind in 0..3 {
            for amount in [0, -1, i64::MIN] {
                let store = FakeStore::new(Ok(vec![movement(1, kind, amount, factory)]));
                assert_error(calculate_home_expenses(&store));
                assert_eq!(store.calls.get(), 1);
            }
        }
        let mut invalid = movement(1, 2, 1, factory);
        invalid.kind = MovementKind::Transfer {
            source_account_id: [1; 16],
            destination_account_id: [1; 16],
        };
        let store = FakeStore::new(Ok(vec![invalid]));
        assert_error(calculate_home_expenses(&store));
        assert_eq!(store.calls.get(), 1);
    }
}

#[test]
fn store_failure_is_sanitized_and_never_retried() {
    let store = FakeStore::new(Err("PRIVATE_FIXTURE SQL C:/synthetic/db amount12345"));
    assert_error(calculate_home_expenses(&store));
    assert_eq!(store.calls.get(), 1);
}
