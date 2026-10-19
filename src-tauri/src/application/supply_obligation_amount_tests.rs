use super::{
    duration_to_epoch_millis, supply_obligation_amount, ObligationAmountStore,
    SupplyObligationAmountError,
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
type Fields = ([u8; 16], [u8; 16], u16, u8, i64, i64, u8, [u8; 16], i64);
#[derive(Default)]
struct Store {
    calls: usize,
    fail: bool,
    fields: Option<Fields>,
}
impl ObligationAmountStore for Store {
    type Error = &'static str;
    fn persist_obligation_amount_with_audit(
        &mut self,
        o: &RecurrenceObligation,
        event: [u8; 16],
        ms: i64,
    ) -> Result<(), Self::Error> {
        self.calls += 1;
        let amount = match &o.state {
            ObligationState::Payable { amount_cents } => *amount_cents,
            _ => panic!("nonpayable forwarded"),
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
            ms,
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
        due_at_ms: 1772420399999,
        state: ObligationState::AwaitingValue,
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
            before: false,
        },
        Store::default(),
    )
}

#[test]
fn supplied_amount_preserves_all_metadata_and_uses_one_audit_id() {
    for ctx in 0..2 {
        for amount in [1, i64::MAX] {
            let (mut ids, time, mut store) = deps();
            let o = supply_obligation_amount(pending(ctx), amount, &mut ids, &time, &mut store)
                .unwrap();
            assert_eq!(
                store.fields,
                Some((
                    [7; 16],
                    [3; 16],
                    2026,
                    2,
                    1772420399999,
                    amount,
                    ctx,
                    [9; 16],
                    1234
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
                ([7; 16], [3; 16], 2026, 2, 1772420399999)
            );
            assert!(
                matches!(o.state,ObligationState::Payable{amount_cents}if amount_cents==amount)
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
fn invalid_amounts_and_nonpending_states_stop_before_side_effects() {
    for ctx in 0..2 {
        for amount in [0, -1, i64::MIN] {
            let (mut ids, time, mut store) = deps();
            assert!(matches!(
                supply_obligation_amount(pending(ctx), amount, &mut ids, &time, &mut store),
                Err(SupplyObligationAmountError::InvalidInput)
            ));
            assert_eq!((ids.calls, time.calls.get(), store.calls), (0, 0, 0));
        }
        for state in [
            ObligationState::Payable { amount_cents: 7 },
            ObligationState::Payable { amount_cents: 0 },
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
                supply_obligation_amount(o, 1, &mut ids, &time, &mut store),
                Err(SupplyObligationAmountError::InvalidInput)
            ));
            assert_eq!((ids.calls, time.calls.get(), store.calls), (0, 0, 0));
        }
    }
}
#[test]
fn id_failure_stops_before_clock_and_store() {
    let (mut ids, time, mut store) = deps();
    ids.fail = true;
    assert!(matches!(
        supply_obligation_amount(pending(0), 1, &mut ids, &time, &mut store),
        Err(SupplyObligationAmountError::IdGenerationFailed)
    ));
    assert_eq!((ids.calls, time.calls.get(), store.calls), (1, 0, 0));
}
#[test]
fn pre_epoch_clock_stops_before_store() {
    let (mut ids, mut time, mut store) = deps();
    time.before = true;
    assert!(matches!(
        supply_obligation_amount(pending(0), 1, &mut ids, &time, &mut store),
        Err(SupplyObligationAmountError::ClockFailed)
    ));
    assert_eq!((ids.calls, time.calls.get(), store.calls), (1, 1, 0));
}
#[test]
fn store_errors_are_opaque_and_do_not_retry() {
    let (mut ids, time, mut store) = deps();
    store.fail = true;
    let err = supply_obligation_amount(pending(0), 1, &mut ids, &time, &mut store)
        .err()
        .unwrap();
    assert_eq!((ids.calls, time.calls.get(), store.calls), (1, 1, 1));
    for e in [
        SupplyObligationAmountError::InvalidInput,
        SupplyObligationAmountError::IdGenerationFailed,
        SupplyObligationAmountError::ClockFailed,
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
        Err(SupplyObligationAmountError::ClockFailed)
    ));
}
