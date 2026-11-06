use super::{list_receivable_due_cents, ListReceivableDueError, ReceivableDueReadStore};
use std::cell::{Cell, RefCell};

struct FakeStore {
    result: RefCell<Option<Result<Vec<i64>, &'static str>>>,
    calls: Cell<usize>,
}

impl FakeStore {
    fn new(result: Result<Vec<i64>, &'static str>) -> Self {
        Self {
            result: RefCell::new(Some(result)),
            calls: Cell::new(0),
        }
    }
}

impl ReceivableDueReadStore for FakeStore {
    type Error = &'static str;

    fn read_remaining_receivable_due_cents(&self) -> Result<Vec<i64>, Self::Error> {
        self.calls.set(self.calls.get() + 1);
        self.result.borrow_mut().take().expect("one store read")
    }
}

fn assert_error(result: Result<i64, ListReceivableDueError>) {
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("expected opaque dashboard error"),
    };
    assert_eq!(format!("{error:?}"), "ListReceivableDueError");
    assert_eq!(format!("{error}"), "receivable total could not be read");
    assert!(std::error::Error::source(&error).is_none());
}

#[test]
fn list_reads_store_once_and_sums_current_due_amounts() {
    let store = FakeStore::new(Ok(vec![0, 1_500, 3_000]));

    assert_eq!(list_receivable_due_cents(&store).unwrap(), 4_500);
    assert_eq!(store.calls.get(), 1);
}

#[test]
fn empty_receivables_project_zero() {
    let store = FakeStore::new(Ok(vec![]));

    assert_eq!(list_receivable_due_cents(&store).unwrap(), 0);
    assert_eq!(store.calls.get(), 1);
}

#[test]
fn store_invalid_amount_and_overflow_errors_are_opaque_without_partial_total() {
    let store = FakeStore::new(Err("private SQL detail"));
    assert_error(list_receivable_due_cents(&store));

    for amounts in [vec![1_500, -1, 3_000], vec![i64::MAX, 1]] {
        let store = FakeStore::new(Ok(amounts));
        assert_error(list_receivable_due_cents(&store));
    }
}
