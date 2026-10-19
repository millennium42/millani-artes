use super::{list_movements, ListMovementsError, MovementReadStore};
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
#[test]
fn empty_history_uses_port_once() {
    let store = FakeStore::new(Ok(vec![]));
    assert!(list_movements(&store).unwrap().is_empty());
    assert_eq!(store.calls.get(), 1);
}
#[test]
fn all_kinds_and_contexts_are_returned_without_reordering_or_mutation() {
    let store = FakeStore::new(Ok(vec![
        Movement {
            id: [3; 16],
            amount_cents: i64::MAX,
            context: Context::Home,
            kind: MovementKind::Entry {
                account_id: [1; 16],
            },
        },
        Movement {
            id: [1; 16],
            amount_cents: 1,
            context: Context::Factory,
            kind: MovementKind::Exit {
                account_id: [2; 16],
            },
        },
        Movement {
            id: [2; 16],
            amount_cents: 27,
            context: Context::Home,
            kind: MovementKind::Transfer {
                source_account_id: [1; 16],
                destination_account_id: [2; 16],
            },
        },
    ]));
    let movements: Vec<Movement> = list_movements(&store).unwrap();
    assert_eq!(store.calls.get(), 1);
    assert_eq!(movements.len(), 3);
    assert_eq!(
        movements.iter().map(|m| m.id).collect::<Vec<_>>(),
        vec![[3; 16], [1; 16], [2; 16]]
    );
    assert_eq!(
        movements.iter().map(|m| m.amount_cents).collect::<Vec<_>>(),
        vec![i64::MAX, 1, 27]
    );
    assert!(matches!(movements[0].context, Context::Home));
    assert!(matches!(movements[1].context, Context::Factory));
    assert!(matches!(movements[2].context, Context::Home));
    assert!(
        matches!(movements[0].kind, MovementKind::Entry { account_id } if account_id == [1;16])
    );
    assert!(matches!(movements[1].kind, MovementKind::Exit { account_id } if account_id == [2;16]));
    assert!(
        matches!(movements[2].kind, MovementKind::Transfer { source_account_id, destination_account_id } if source_account_id == [1;16] && destination_account_id == [2;16])
    );
}
#[test]
fn private_store_error_is_not_exposed_or_retried() {
    let store = FakeStore::new(Err("PRIVATE_FIXTURE SQL C:/synthetic/db 12345"));
    let error = match list_movements(&store) {
        Err(e) => e,
        Ok(_) => panic!("expected failure"),
    };
    assert!(matches!(error, ListMovementsError));
    assert_eq!(format!("{error:?}"), "ListMovementsError");
    assert_eq!(format!("{error}"), "movements could not be read");
    assert!(std::error::Error::source(&error).is_none());
    assert_eq!(store.calls.get(), 1);
}
