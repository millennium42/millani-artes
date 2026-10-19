use super::{duration_to_epoch_millis, pay_recurrence, PayRecurrenceError, RecurrencePaymentStore};
use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::{Context, Movement, MovementKind, ObligationState, RecurrenceObligation};
use std::cell::Cell;
use std::error::Error;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const PRIVATE_STORE_MARKER: &str = "PRIVATE_ENTRY_STORE_FAILURE_9f2a";
const PRIVATE_ID_MARKER: &str = "PRIVATE_ID_GENERATION_FAILURE_3c7b";

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
    values: [[u8; 16]; 2],
    fail_at: Option<usize>,
    calls: usize,
}

impl FixedIds {
    fn new(fail_at: Option<usize>) -> Self {
        Self {
            values: [[1; 16], [2; 16]],
            fail_at,
            calls: 0,
        }
    }
}

impl UuidGenerator for FixedIds {
    type Error = &'static str;

    fn generate(&mut self) -> Result<[u8; 16], Self::Error> {
        let call = self.calls;
        self.calls += 1;
        if self.fail_at == Some(call) {
            return Err(PRIVATE_ID_MARKER);
        }
        self.values
            .get(call)
            .copied()
            .ok_or("UUID_SOURCE_EXHAUSTED")
    }
}

type ObligationFields = ([u8; 16], [u8; 16], u16, u8, i64, i64, [u8; 16], u8);
type PaymentFields = ([u8; 16], [u8; 16], i64, u8);
#[derive(Default)]
struct FakeStore {
    calls: usize,
    fail: bool,
    persisted: Option<(ObligationFields, PaymentFields, [u8; 16], i64)>,
}
struct StoreFailure(&'static str);

impl std::fmt::Debug for StoreFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("StoreFailure").field(&self.0).finish()
    }
}

impl std::fmt::Display for StoreFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl Error for StoreFailure {}

impl RecurrencePaymentStore for FakeStore {
    type Error = StoreFailure;
    fn persist_recurrence_payment_with_audit(
        &mut self,
        o: &RecurrenceObligation,
        p: &Movement,
        event: [u8; 16],
        ms: i64,
    ) -> Result<(), Self::Error> {
        self.calls += 1;
        let (amount, link) = match &o.state {
            ObligationState::Paid {
                amount_cents,
                movement_id,
            } => (*amount_cents, *movement_id),
            _ => panic!("nonpaid target"),
        };
        let account = match &p.kind {
            MovementKind::Exit { account_id } => *account_id,
            _ => panic!("nonexit payment"),
        };
        self.persisted = Some((
            (
                o.id,
                o.recurrence_id,
                o.period_year,
                o.period_month,
                o.due_at_ms,
                amount,
                link,
                if matches!(&o.context, Context::Home) {
                    0
                } else {
                    1
                },
            ),
            (
                p.id,
                account,
                p.amount_cents,
                if matches!(&p.context, Context::Home) {
                    0
                } else {
                    1
                },
            ),
            event,
            ms,
        ));
        if self.fail {
            Err(StoreFailure(PRIVATE_STORE_MARKER))
        } else {
            Ok(())
        }
    }
}
fn obligation(overdue: u8, amount: i64, ctx: u8) -> RecurrenceObligation {
    RecurrenceObligation {
        id: [7; 16],
        recurrence_id: [4; 16],
        period_year: 2026,
        period_month: 2,
        due_at_ms: 1000,
        state: if overdue == 0 {
            ObligationState::Payable {
                amount_cents: amount,
            }
        } else {
            ObligationState::Overdue {
                amount_cents: amount,
            }
        },
        context: if ctx == 0 {
            Context::Home
        } else {
            Context::Factory
        },
    }
}
#[test]
fn payment_preserves_metadata_and_links_exact_exit_in_both_states_contexts_and_amount_bounds() {
    for state in 0..2 {
        for ctx in 0..2 {
            for amount in [1, i64::MAX] {
                let mut ids = FixedIds::new(None);
                let clock = FixedClock::new(UNIX_EPOCH + Duration::from_millis(1234));
                let mut store = FakeStore::default();
                let o = pay_recurrence(
                    obligation(state, amount, ctx),
                    [3; 16],
                    &mut ids,
                    &clock,
                    &mut store,
                )
                .unwrap();
                assert_eq!(
                    store.persisted,
                    Some((
                        ([7; 16], [4; 16], 2026, 2, 1000, amount, [1; 16], ctx),
                        ([1; 16], [3; 16], amount, ctx),
                        [2; 16],
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
                    ([7; 16], [4; 16], 2026, 2, 1000)
                );
                assert!(
                    matches!(o.state,ObligationState::Paid{amount_cents,movement_id}if amount_cents==amount&&movement_id==[1;16])
                );
                assert!(matches!(
                    (o.context, ctx),
                    (Context::Home, 0) | (Context::Factory, 1)
                ));
                assert_eq!((ids.calls, clock.calls.get(), store.calls), (2, 1, 1));
            }
        }
    }
}
#[test]
fn noneligible_or_invalid_states_stop_before_identifiers_clock_and_store() {
    for state in [
        ObligationState::AwaitingValue,
        ObligationState::Paid {
            amount_cents: 1,
            movement_id: [9; 16],
        },
        ObligationState::Paid {
            amount_cents: 0,
            movement_id: [9; 16],
        },
        ObligationState::Payable { amount_cents: 0 },
        ObligationState::Payable { amount_cents: -1 },
        ObligationState::Payable {
            amount_cents: i64::MIN,
        },
        ObligationState::Overdue { amount_cents: 0 },
        ObligationState::Overdue { amount_cents: -1 },
        ObligationState::Overdue {
            amount_cents: i64::MIN,
        },
    ] {
        let mut o = obligation(0, 1, 0);
        o.state = state;
        let mut ids = FixedIds::new(None);
        let clock = FixedClock::new(UNIX_EPOCH);
        let mut store = FakeStore::default();
        assert!(matches!(
            pay_recurrence(o, [3; 16], &mut ids, &clock, &mut store),
            Err(PayRecurrenceError::InvalidInput)
        ));
        assert_eq!((ids.calls, clock.calls.get(), store.calls), (0, 0, 0));
        assert!(store.persisted.is_none());
    }
}
#[test]
fn either_identifier_failure_or_zero_movement_identifier_prevents_clock_and_store() {
    for fail in 0..2 {
        let mut ids = FixedIds::new(Some(fail));
        let clock = FixedClock::new(UNIX_EPOCH);
        let mut store = FakeStore::default();
        let e = pay_recurrence(obligation(0, 1, 0), [3; 16], &mut ids, &clock, &mut store)
            .err()
            .unwrap();
        assert!(matches!(e, PayRecurrenceError::IdGenerationFailed));
        assert_eq!(
            (ids.calls, clock.calls.get(), store.calls),
            (fail + 1, 0, 0)
        );
        assert!(!e.to_string().contains(PRIVATE_ID_MARKER));
    }
    let mut ids = FixedIds::new(None);
    ids.values[0] = [0; 16];
    let clock = FixedClock::new(UNIX_EPOCH);
    let mut store = FakeStore::default();
    assert!(matches!(
        pay_recurrence(obligation(1, 1, 1), [3; 16], &mut ids, &clock, &mut store),
        Err(PayRecurrenceError::InvalidInput)
    ));
    assert_eq!((ids.calls, clock.calls.get(), store.calls), (1, 0, 0));
}
#[test]
fn pre_epoch_audit_clock_stops_after_two_identifiers_without_persisting() {
    let mut ids = FixedIds::new(None);
    let clock = FixedClock::new(UNIX_EPOCH - Duration::from_millis(1));
    let mut store = FakeStore::default();
    assert!(matches!(
        pay_recurrence(obligation(0, 1, 0), [3; 16], &mut ids, &clock, &mut store),
        Err(PayRecurrenceError::ClockFailed)
    ));
    assert_eq!((ids.calls, clock.calls.get(), store.calls), (2, 1, 0));
    assert!(store.persisted.is_none());
}
#[test]
fn store_failure_and_all_public_errors_hide_private_payloads_and_never_retry() {
    let mut ids = FixedIds::new(None);
    let clock = FixedClock::new(UNIX_EPOCH);
    let mut store = FakeStore {
        fail: true,
        ..FakeStore::default()
    };
    let error = pay_recurrence(obligation(0, 1, 0), [3; 16], &mut ids, &clock, &mut store)
        .err()
        .unwrap();
    assert_eq!((ids.calls, clock.calls.get(), store.calls), (2, 1, 1));
    for e in [
        PayRecurrenceError::InvalidInput,
        PayRecurrenceError::IdGenerationFailed,
        PayRecurrenceError::ClockFailed,
        error,
    ] {
        assert!(e.source().is_none());
        assert!(!format!("{e:?}/{e}").contains("PRIVATE"));
        assert!(!e.to_string().is_empty());
    }
}
#[test]
fn duration_conversion_preserves_i64_bound_and_rejects_overflow() {
    assert_eq!(
        duration_to_epoch_millis(Duration::from_millis(i64::MAX as u64)),
        Ok(i64::MAX)
    );
    assert!(matches!(
        duration_to_epoch_millis(Duration::from_millis(i64::MAX as u64 + 1)),
        Err(PayRecurrenceError::ClockFailed)
    ));
}
