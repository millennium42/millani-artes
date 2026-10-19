use super::{duration_to_epoch_millis, edit_movement, EditMovementError, MovementEditingStore};
use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::{Context, Movement, MovementKind};
use std::cell::Cell;
use std::error::Error;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

struct Ids {
    calls: usize,
    fail: bool,
}
impl UuidGenerator for Ids {
    type Error = &'static str;
    fn generate(&mut self) -> Result<[u8; 16], Self::Error> {
        self.calls += 1;
        if self.fail {
            Err("PRIVATE_ID")
        } else {
            Ok([9; 16])
        }
    }
}
struct Time {
    calls: Cell<usize>,
    before: bool,
}
impl Clock for Time {
    fn now(&self) -> SystemTime {
        self.calls.set(self.calls.get() + 1);
        if self.before {
            UNIX_EPOCH - Duration::from_millis(1)
        } else {
            UNIX_EPOCH + Duration::from_millis(1234)
        }
    }
}
type PersistedEdit = ([u8; 16], i64, u8, u8, [u8; 16], [u8; 16], [u8; 16], i64);
#[derive(Default)]
struct Store {
    calls: usize,
    fail: bool,
    fields: Option<PersistedEdit>,
}
impl MovementEditingStore for Store {
    type Error = &'static str;
    fn persist_movement_edit_with_audit(
        &mut self,
        m: &Movement,
        event: [u8; 16],
        ms: i64,
    ) -> Result<(), Self::Error> {
        self.calls += 1;
        let (kind, a, b) = match &m.kind {
            MovementKind::Entry { account_id } => (0, *account_id, [0; 16]),
            MovementKind::Exit { account_id } => (1, *account_id, [0; 16]),
            MovementKind::Transfer {
                source_account_id,
                destination_account_id,
            } => (2, *source_account_id, *destination_account_id),
        };
        self.fields = Some((
            m.id,
            m.amount_cents,
            if matches!(&m.context, Context::Home) {
                0
            } else {
                1
            },
            kind,
            a,
            b,
            event,
            ms,
        ));
        if self.fail {
            Err("PRIVATE_STORE")
        } else {
            Ok(())
        }
    }
}
fn movement(kind: u8, amount: i64, ctx: u8) -> Movement {
    Movement {
        id: [7; 16],
        amount_cents: amount,
        context: if ctx == 0 {
            Context::Home
        } else {
            Context::Factory
        },
        kind: match kind {
            0 => MovementKind::Entry {
                account_id: [3; 16],
            },
            1 => MovementKind::Exit {
                account_id: [4; 16],
            },
            _ => MovementKind::Transfer {
                source_account_id: [3; 16],
                destination_account_id: [4; 16],
            },
        },
    }
}
fn deps() -> (Ids, Time, Store) {
    (
        Ids {
            calls: 0,
            fail: false,
        },
        Time {
            calls: Cell::new(0),
            before: false,
        },
        Store::default(),
    )
}

#[test]
fn edits_preserve_identity_and_forward_all_fields_with_one_audit_id() {
    for kind in 0..3 {
        for ctx in 0..2 {
            for amount in [1, i64::MAX] {
                let (mut ids, time, mut store) = deps();
                let m = edit_movement(movement(kind, amount, ctx), &mut ids, &time, &mut store)
                    .unwrap();
                let (a, b) = if kind == 1 {
                    ([4; 16], [0; 16])
                } else if kind == 0 {
                    ([3; 16], [0; 16])
                } else {
                    ([3; 16], [4; 16])
                };
                assert_eq!(
                    store.fields,
                    Some(([7; 16], amount, ctx, kind, a, b, [9; 16], 1234))
                );
                assert_eq!(m.id, [7; 16]);
                assert_eq!(m.amount_cents, amount);
                assert_eq!((ids.calls, time.calls.get(), store.calls), (1, 1, 1));
            }
        }
    }
}
#[test]
fn invalid_amounts_and_equal_transfer_accounts_stop_before_side_effects() {
    for kind in 0..3 {
        for ctx in 0..2 {
            for amount in [0, -1, i64::MIN] {
                let (mut ids, time, mut store) = deps();
                assert!(matches!(
                    edit_movement(movement(kind, amount, ctx), &mut ids, &time, &mut store),
                    Err(EditMovementError::InvalidInput)
                ));
                assert_eq!((ids.calls, time.calls.get(), store.calls), (0, 0, 0));
            }
        }
    }
    for ctx in 0..2 {
        let (mut ids, time, mut store) = deps();
        let mut m = movement(2, 1, ctx);
        m.kind = MovementKind::Transfer {
            source_account_id: [3; 16],
            destination_account_id: [3; 16],
        };
        assert!(matches!(
            edit_movement(m, &mut ids, &time, &mut store),
            Err(EditMovementError::InvalidInput)
        ));
        assert_eq!((ids.calls, time.calls.get(), store.calls), (0, 0, 0));
    }
}
#[test]
fn id_failure_stops_before_clock_and_store() {
    let (mut ids, time, mut store) = deps();
    ids.fail = true;
    assert!(matches!(
        edit_movement(movement(0, 1, 0), &mut ids, &time, &mut store),
        Err(EditMovementError::IdGenerationFailed)
    ));
    assert_eq!((ids.calls, time.calls.get(), store.calls), (1, 0, 0));
}
#[test]
fn pre_epoch_clock_stops_before_store() {
    let (mut ids, mut time, mut store) = deps();
    time.before = true;
    assert!(matches!(
        edit_movement(movement(0, 1, 0), &mut ids, &time, &mut store),
        Err(EditMovementError::ClockFailed)
    ));
    assert_eq!((ids.calls, time.calls.get(), store.calls), (1, 1, 0));
}
#[test]
fn store_errors_are_opaque_and_do_not_retry() {
    let (mut ids, time, mut store) = deps();
    store.fail = true;
    let err = edit_movement(movement(0, 1, 0), &mut ids, &time, &mut store)
        .err()
        .unwrap();
    assert_eq!((ids.calls, time.calls.get(), store.calls), (1, 1, 1));
    for e in [
        EditMovementError::InvalidInput,
        EditMovementError::IdGenerationFailed,
        EditMovementError::ClockFailed,
        err,
    ] {
        assert!(!e.to_string().is_empty());
        assert!(!format!("{e:?}").contains("PRIVATE"));
        assert!(!e.to_string().contains("PRIVATE"));
        assert!(e.source().is_none());
    }
}
#[test]
fn duration_conversion_checks_i64_boundary() {
    assert_eq!(
        duration_to_epoch_millis(Duration::from_millis(i64::MAX as u64)),
        Ok(i64::MAX)
    );
    assert!(matches!(
        duration_to_epoch_millis(Duration::from_millis(i64::MAX as u64 + 1)),
        Err(EditMovementError::ClockFailed)
    ));
}
