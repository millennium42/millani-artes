use super::{
    duration_to_epoch_millis, mark_obligation_overdue, MarkObligationOverdueError,
    OverdueObligationStore,
};
use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::{Context, ObligationState, RecurrenceObligation};
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
    instant: SystemTime,
}
impl Clock for Time {
    fn now(&self) -> SystemTime {
        self.calls.set(self.calls.get() + 1);
        self.instant
    }
}
type Fields = (
    [u8; 16],
    [u8; 16],
    u16,
    u8,
    i64,
    i64,
    u8,
    [u8; 16],
    SystemTime,
);
#[derive(Default)]
struct Store {
    calls: usize,
    fail: bool,
    fields: Option<Fields>,
}
impl OverdueObligationStore for Store {
    type Error = &'static str;
    fn persist_overdue_obligation_with_audit(
        &mut self,
        o: &RecurrenceObligation,
        event: [u8; 16],
        observed_at: SystemTime,
    ) -> Result<(), Self::Error> {
        self.calls += 1;
        let amount = match &o.state {
            ObligationState::Overdue { amount_cents } => *amount_cents,
            _ => panic!("nonoverdue forwarded"),
        };
        self.fields = Some((
            o.id,
            o.recurrence_id,
            o.period_year,
            o.period_month,
            o.due_at_ms,
            amount,
            if matches!(&o.context, Context::Home) {
                0
            } else {
                1
            },
            event,
            observed_at,
        ));
        if self.fail {
            Err("PRIVATE_STORE")
        } else {
            Ok(())
        }
    }
}
fn pending(ctx: u8) -> RecurrenceObligation {
    RecurrenceObligation {
        id: [7; 16],
        recurrence_id: [3; 16],
        period_year: 2026,
        period_month: 2,
        due_at_ms: 1000,
        state: ObligationState::Payable { amount_cents: 1000 },
        context: if ctx == 0 {
            Context::Home
        } else {
            Context::Factory
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
            instant: UNIX_EPOCH + Duration::from_millis(1234),
        },
        Store::default(),
    )
}

#[test]
fn marks_payable_overdue_preserving_metadata_and_exact_observation() {
    for ctx in 0..2 {
        for amount in [1, i64::MAX] {
            let (mut ids, time, mut store) = deps();
            let mut o = pending(ctx);
            o.state = ObligationState::Payable {
                amount_cents: amount,
            };
            let o = mark_obligation_overdue(o, &mut ids, &time, &mut store).unwrap();
            assert_eq!(
                store.fields,
                Some((
                    [7; 16],
                    [3; 16],
                    2026,
                    2,
                    1000,
                    amount,
                    ctx,
                    [9; 16],
                    time.instant
                ))
            );
            assert_eq!(
                (
                    o.id,
                    o.recurrence_id,
                    o.period_year,
                    o.period_month,
                    o.due_at_ms
                ),
                ([7; 16], [3; 16], 2026, 2, 1000)
            );
            assert!(
                matches!(o.state,ObligationState::Overdue{amount_cents}if amount_cents==amount)
            );
            assert!(matches!(
                (o.context, ctx),
                (Context::Home, 0) | (Context::Factory, 1)
            ));
            assert_eq!((ids.calls, time.calls.get(), store.calls), (1, 1, 1));
        }
    }
}
#[test]
fn strict_boundary_before_equal_and_one_windows_tick_after_preserves_observation() {
    let due = UNIX_EPOCH + Duration::from_millis(1000);
    for ctx in 0..2 {
        for at in [
            due - Duration::from_nanos(100),
            due,
            due + Duration::from_nanos(100),
        ] {
            let (mut ids, mut time, mut store) = deps();
            time.instant = at;
            let result = mark_obligation_overdue(pending(ctx), &mut ids, &time, &mut store);
            if at > due {
                assert!(matches!(
                    result,
                    Ok(RecurrenceObligation {
                        state: ObligationState::Overdue { amount_cents: 1000 },
                        ..
                    })
                ));
                assert_eq!(store.fields.as_ref().unwrap().8, at);
                assert_eq!((ids.calls, time.calls.get(), store.calls), (1, 1, 1));
            } else {
                assert!(matches!(
                    result,
                    Err(MarkObligationOverdueError::InvalidInput)
                ));
                assert_eq!((ids.calls, time.calls.get(), store.calls), (0, 1, 0));
            }
        }
    }
}
#[test]
fn invalid_states_including_nonpositive_payable_reject_before_clock_id_store() {
    for ctx in 0..2 {
        for amount in [0, -1, i64::MIN] {
            let (mut ids, time, mut store) = deps();
            let mut o = pending(ctx);
            o.state = ObligationState::Payable {
                amount_cents: amount,
            };
            assert!(matches!(
                mark_obligation_overdue(o, &mut ids, &time, &mut store),
                Err(MarkObligationOverdueError::InvalidInput)
            ));
            assert_eq!((ids.calls, time.calls.get(), store.calls), (0, 0, 0));
        }
        for state in [
            ObligationState::AwaitingValue,
            ObligationState::Overdue { amount_cents: 7 },
            ObligationState::Paid {
                amount_cents: 7,
                movement_id: [1; 16],
            },
        ] {
            let (mut ids, time, mut store) = deps();
            let mut o = pending(ctx);
            o.state = state;
            assert!(matches!(
                mark_obligation_overdue(o, &mut ids, &time, &mut store),
                Err(MarkObligationOverdueError::InvalidInput)
            ));
            assert_eq!((ids.calls, time.calls.get(), store.calls), (0, 0, 0));
        }
    }
}
#[test]
fn pre_epoch_clock_and_due_i64_min_expired_makes_clock_failed_before_id() {
    let (mut ids, mut time, mut store) = deps();
    time.instant = UNIX_EPOCH - Duration::from_millis(1);
    let mut o = pending(0);
    o.due_at_ms = i64::MIN;
    assert!(matches!(
        mark_obligation_overdue(o, &mut ids, &time, &mut store),
        Err(MarkObligationOverdueError::ClockFailed)
    ));
    assert_eq!((ids.calls, time.calls.get(), store.calls), (0, 1, 0));
}
#[test]
fn id_failure_follows_one_clock_read_and_stops_before_store() {
    let (mut ids, time, mut store) = deps();
    ids.fail = true;
    assert!(matches!(
        mark_obligation_overdue(pending(0), &mut ids, &time, &mut store),
        Err(MarkObligationOverdueError::IdGenerationFailed)
    ));
    assert_eq!((ids.calls, time.calls.get(), store.calls), (1, 1, 0));
}
#[test]
fn store_errors_are_opaque_and_do_not_retry() {
    let (mut ids, time, mut store) = deps();
    store.fail = true;
    let err = mark_obligation_overdue(pending(0), &mut ids, &time, &mut store)
        .err()
        .unwrap();
    assert_eq!((ids.calls, time.calls.get(), store.calls), (1, 1, 1));
    for e in [
        MarkObligationOverdueError::InvalidInput,
        MarkObligationOverdueError::IdGenerationFailed,
        MarkObligationOverdueError::ClockFailed,
        err,
    ] {
        assert!(!e.to_string().is_empty());
        assert!(!format!("{e:?}/{e}").contains("PRIVATE"));
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
        Err(MarkObligationOverdueError::ClockFailed)
    ));
}
