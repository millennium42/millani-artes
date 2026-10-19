use super::{
    duration_to_epoch_millis, generate_fixed_obligation, FixedObligationStore,
    GenerateFixedObligationError,
};
use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::{Context, FixedRecurrence, ObligationState, RecurrenceObligation};
use std::cell::Cell;
use std::error::Error;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
const SECRET_ID: &str = "PRIVATE_RECURRENCE_ID_72f9";
const SECRET_STORE: &str = "PRIVATE_RECURRENCE_STORE_91d2";
struct FixedClock {
    now: SystemTime,
    calls: Cell<usize>,
}
impl FixedClock {
    fn new(now: SystemTime) -> Self {
        Self {
            now,
            calls: Cell::new(0),
        }
    }
}
impl Clock for FixedClock {
    fn now(&self) -> SystemTime {
        self.calls.set(self.calls.get() + 1);
        self.now
    }
}
struct FixedIds {
    calls: usize,
    fail_at: Option<usize>,
}
impl FixedIds {
    fn new(fail_at: Option<usize>) -> Self {
        Self { calls: 0, fail_at }
    }
}
impl UuidGenerator for FixedIds {
    type Error = &'static str;
    fn generate(&mut self) -> Result<[u8; 16], Self::Error> {
        let n = self.calls;
        self.calls += 1;
        if self.fail_at == Some(n) {
            return Err(SECRET_ID);
        }
        match n {
            0 => Ok([1; 16]),
            1 => Ok([2; 16]),
            _ => Err(SECRET_ID),
        }
    }
}
type Stored = ([u8; 16], [u8; 16], u16, u8, i64, i64, u8, [u8; 16], i64);
#[derive(Default)]
struct FakeStore {
    calls: usize,
    fail: bool,
    stored: Option<Stored>,
}
impl FixedObligationStore for FakeStore {
    type Error = &'static str;
    fn persist_fixed_obligation_with_audit(
        &mut self,
        rule: &FixedRecurrence,
        o: &RecurrenceObligation,
        event: [u8; 16],
        ms: i64,
    ) -> Result<(), Self::Error> {
        self.calls += 1;
        assert_eq!(rule.id, o.recurrence_id);
        let amount = match &o.state {
            ObligationState::Payable { amount_cents } => *amount_cents,
            _ => panic!("expected payable"),
        };
        self.stored = Some((
            o.id,
            o.recurrence_id,
            o.period_year,
            o.period_month,
            o.due_at_ms,
            amount,
            match &o.context {
                Context::Home => 0,
                Context::Factory => 1,
            },
            event,
            ms,
        ));
        if self.fail {
            Err(SECRET_STORE)
        } else {
            Ok(())
        }
    }
}
fn rule(amount_cents: i64, day: u8, ctx: u8) -> FixedRecurrence {
    FixedRecurrence {
        id: [3; 16],
        name: "  Café\0  ".into(),
        amount_cents,
        context: if ctx == 0 {
            Context::Home
        } else {
            Context::Factory
        },
        due_day: day,
    }
}
fn err(
    r: Result<RecurrenceObligation, GenerateFixedObligationError>,
) -> GenerateFixedObligationError {
    match r {
        Err(e) => e,
        Ok(_) => panic!("expected error"),
    }
}

#[test]
fn fixed_obligation_uses_real_domain_and_exactly_two_ids_one_clock_one_store() {
    for amount in [1, i64::MAX] {
        for ctx in 0..2 {
            let rule = rule(amount, 31, ctx);
            let mut ids = FixedIds::new(None);
            let clock = FixedClock::new(UNIX_EPOCH + Duration::from_millis(1234));
            let mut store = FakeStore::default();
            let o =
                generate_fixed_obligation(&rule, (2026, 2), &mut ids, &clock, &mut store).unwrap();
            assert_eq!(ids.calls, 2);
            assert_eq!(clock.calls.get(), 1);
            assert_eq!(store.calls, 1);
            let expected = rule.generate_obligation([1; 16], (2026, 2)).unwrap();
            assert_eq!(
                store.stored,
                Some((
                    [1; 16],
                    [3; 16],
                    2026,
                    2,
                    expected.due_at_ms,
                    amount,
                    ctx,
                    [2; 16],
                    1234
                ))
            );
            assert_eq!(o.id, [1; 16]);
            assert_eq!(o.recurrence_id, [3; 16]);
            assert_eq!(o.due_at_ms, expected.due_at_ms);
            assert!(
                matches!(o.state,ObligationState::Payable{amount_cents}if amount_cents==amount)
            );
            assert!(matches!(
                (&o.context, ctx),
                (Context::Home, 0) | (Context::Factory, 1)
            ));
            assert_eq!(rule.amount_cents, amount);
            assert_eq!(rule.due_day, 31);
            assert_eq!(rule.name, "  Café\0  ");
        }
    }
}
#[test]
fn invalid_rule_or_period_is_rejected_before_ids_clock_or_store() {
    let mut blank = rule(7, 1, 0);
    blank.name = "\u{2003}\u{00a0}".into();
    for (rule, period) in [
        (blank, (2026, 1)),
        (rule(0, 1, 0), (2026, 1)),
        (rule(-1, 1, 0), (2026, 1)),
        (rule(i64::MIN, 1, 0), (2026, 1)),
        (rule(7, 0, 0), (2026, 1)),
        (rule(7, 32, 0), (2026, 1)),
        (rule(7, 1, 0), (0, 1)),
        (rule(7, 1, 0), (10000, 1)),
        (rule(7, 1, 0), (2026, 0)),
        (rule(7, 1, 0), (2026, 13)),
    ] {
        let mut ids = FixedIds::new(None);
        let clock = FixedClock::new(UNIX_EPOCH);
        let mut store = FakeStore::default();
        assert!(matches!(
            generate_fixed_obligation(&rule, period, &mut ids, &clock, &mut store),
            Err(GenerateFixedObligationError::InvalidInput)
        ));
        assert_eq!(ids.calls, 0);
        assert_eq!(clock.calls.get(), 0);
        assert_eq!(store.calls, 0);
    }
}
#[test]
fn identifier_failures_stop_before_clock_and_store_without_exposing_source() {
    for at in 0..2 {
        let mut ids = FixedIds::new(Some(at));
        let clock = FixedClock::new(UNIX_EPOCH);
        let mut store = FakeStore::default();
        let e = err(generate_fixed_obligation(
            &rule(7, 1, 0),
            (2026, 1),
            &mut ids,
            &clock,
            &mut store,
        ));
        assert!(matches!(
            e,
            GenerateFixedObligationError::IdGenerationFailed
        ));
        assert_eq!(ids.calls, at + 1);
        assert_eq!(clock.calls.get(), 0);
        assert_eq!(store.calls, 0);
        assert!(!format!("{e:?}/{e}").contains(SECRET_ID));
        assert!(e.source().is_none());
    }
}
#[test]
fn clock_before_epoch_and_millisecond_overflow_do_not_reach_store() {
    let mut ids = FixedIds::new(None);
    let clock = FixedClock::new(UNIX_EPOCH - Duration::from_millis(1));
    let mut store = FakeStore::default();
    assert!(matches!(
        generate_fixed_obligation(&rule(7, 1, 0), (2026, 1), &mut ids, &clock, &mut store),
        Err(GenerateFixedObligationError::ClockFailed)
    ));
    assert_eq!(ids.calls, 2);
    assert_eq!(clock.calls.get(), 1);
    assert_eq!(store.calls, 0);
    assert!(matches!(
        duration_to_epoch_millis(Duration::from_secs(u64::MAX)),
        Err(GenerateFixedObligationError::ClockFailed)
    ));
    assert_eq!(
        duration_to_epoch_millis(Duration::from_millis(i64::MAX as u64)).unwrap(),
        i64::MAX
    );
}
#[test]
fn store_failure_is_single_call_and_keeps_private_details_opaque() {
    let mut ids = FixedIds::new(None);
    let clock = FixedClock::new(UNIX_EPOCH);
    let mut store = FakeStore {
        fail: true,
        ..FakeStore::default()
    };
    let e = err(generate_fixed_obligation(
        &rule(7, 1, 1),
        (2026, 1),
        &mut ids,
        &clock,
        &mut store,
    ));
    assert!(matches!(e, GenerateFixedObligationError::StoreFailed));
    assert_eq!(ids.calls, 2);
    assert_eq!(clock.calls.get(), 1);
    assert_eq!(store.calls, 1);
    assert!(!format!("{e:?}/{e}").contains(SECRET_STORE));
    assert!(e.source().is_none());
}
#[test]
fn all_public_errors_have_static_debug_display_and_no_source() {
    for (e, name) in [
        (GenerateFixedObligationError::InvalidInput, "InvalidInput"),
        (
            GenerateFixedObligationError::IdGenerationFailed,
            "IdGenerationFailed",
        ),
        (GenerateFixedObligationError::ClockFailed, "ClockFailed"),
        (GenerateFixedObligationError::StoreFailed, "StoreFailed"),
    ] {
        assert_eq!(format!("{e:?}"), name);
        assert!(!e.to_string().is_empty());
        assert!(e.source().is_none());
        assert!(!format!("{e:?}/{e}").contains(SECRET_ID));
        assert!(!format!("{e:?}/{e}").contains(SECRET_STORE));
    }
}
