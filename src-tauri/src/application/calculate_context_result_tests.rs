use super::{calculate_context_result, CalculateContextResultError};
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
fn context(which: u8) -> Context {
    if which == 0 {
        Context::Home
    } else {
        Context::Factory
    }
}
fn movement(id: u8, kind: u8, amount: i64, which: u8) -> Movement {
    Movement {
        id: [id; 16],
        amount_cents: amount,
        context: context(which),
        kind: match kind {
            0 => MovementKind::Entry {
                account_id: [1; 16],
            },
            1 => MovementKind::Exit {
                account_id: [2; 16],
            },
            _ => MovementKind::Transfer {
                source_account_id: if which == 0 { [1; 16] } else { [2; 16] },
                destination_account_id: if which == 0 { [2; 16] } else { [1; 16] },
            },
        },
    }
}
fn assert_error(result: Result<i64, CalculateContextResultError>) {
    let e = result.unwrap_err();
    assert_eq!(format!("{e:?}"), "CalculateContextResultError");
    assert_eq!(format!("{e}"), "context result could not be calculated");
    assert!(std::error::Error::source(&e).is_none());
}
#[test]
fn empty_and_transfer_only_histories_return_zero_with_one_read() {
    for wanted in 0..2 {
        let empty = FakeStore::new(Ok(vec![]));
        assert_eq!(
            calculate_context_result(&context(wanted), &empty).unwrap(),
            0
        );
        assert_eq!(empty.calls.get(), 1);
        for amount in [1, i64::MAX] {
            let store = FakeStore::new(Ok(vec![
                movement(1, 2, amount, 0),
                movement(2, 2, amount, 1),
            ]));
            assert_eq!(
                calculate_context_result(&context(wanted), &store).unwrap(),
                0
            );
            assert_eq!(store.calls.get(), 1);
        }
    }
}
#[test]
fn mixed_contexts_and_accounts_use_only_selected_context_contributions() {
    for (wanted, expected) in [(0, 60), (1, 70)] {
        let store = FakeStore::new(Ok(vec![
            movement(1, 0, 100, 0),
            movement(2, 1, 40, 0),
            movement(3, 2, i64::MAX, 0),
            movement(4, 0, 75, 1),
            movement(5, 1, 5, 1),
            movement(6, 2, i64::MAX, 1),
        ]));
        assert_eq!(
            calculate_context_result(&context(wanted), &store).unwrap(),
            expected
        );
        assert_eq!(store.calls.get(), 1);
        let other_overflow = FakeStore::new(Ok(vec![
            movement(1, 0, 3, wanted),
            movement(2, 0, i64::MAX, 1 - wanted),
            movement(3, 0, i64::MAX, 1 - wanted),
        ]));
        assert_eq!(
            calculate_context_result(&context(wanted), &other_overflow).unwrap(),
            3
        );
        assert_eq!(other_overflow.calls.get(), 1);
    }
}
#[test]
fn widened_sum_checks_final_bounds_and_allows_intermediate_cancellation() {
    let cases = [
        (vec![(0, i64::MAX)], Some(i64::MAX)),
        (vec![(1, i64::MAX), (1, 1)], Some(i64::MIN)),
        (vec![(0, i64::MAX), (0, 1)], None),
        (vec![(1, i64::MAX), (1, 2)], None),
        (
            vec![(0, i64::MAX), (0, i64::MAX), (1, i64::MAX)],
            Some(i64::MAX),
        ),
        (
            vec![(1, i64::MAX), (1, i64::MAX), (0, i64::MAX)],
            Some(-i64::MAX),
        ),
    ];
    for wanted in 0..2 {
        for (items, expected) in &cases {
            let data = items
                .iter()
                .enumerate()
                .map(|(i, (kind, amount))| movement((i + 1) as u8, *kind, *amount, wanted))
                .collect();
            let store = FakeStore::new(Ok(data));
            let result = calculate_context_result(&context(wanted), &store);
            match expected {
                Some(value) => assert_eq!(result.unwrap(), *value),
                None => assert_error(result),
            }
            assert_eq!(store.calls.get(), 1);
        }
    }
}
#[test]
fn invalid_movements_are_rejected_before_context_filtering() {
    for wanted in 0..2 {
        for actual in 0..2 {
            for kind in 0..3 {
                for amount in [0, -1, i64::MIN] {
                    let store = FakeStore::new(Ok(vec![movement(1, kind, amount, actual)]));
                    assert_error(calculate_context_result(&context(wanted), &store));
                    assert_eq!(store.calls.get(), 1);
                }
            }
            let mut invalid = movement(1, 2, 1, actual);
            invalid.kind = MovementKind::Transfer {
                source_account_id: [1; 16],
                destination_account_id: [1; 16],
            };
            let store = FakeStore::new(Ok(vec![invalid]));
            assert_error(calculate_context_result(&context(wanted), &store));
            assert_eq!(store.calls.get(), 1);
        }
    }
}
#[test]
fn store_failure_is_sanitized_and_never_retried() {
    let store = FakeStore::new(Err("PRIVATE_FIXTURE SQL C:/synthetic/db amount12345"));
    assert_error(calculate_context_result(&Context::Home, &store));
    assert_eq!(store.calls.get(), 1);
}
