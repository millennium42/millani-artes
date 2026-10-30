use super::{
    record_receivable_receipt, ReceivableReceiptRecordingStore, ReceivableReceiptWrite,
    RecordReceivableReceiptError,
};
use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::{Context, MovementKind, ReceivableReceipt};
use std::cell::Cell;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const PRIVATE_ID_MARKER: &str = "PRIVATE_RCV007_ID_FAILURE_9b2a";

struct FixedIds {
    values: [[u8; 16]; 4],
    fail_at: Option<usize>,
    calls: usize,
}

impl FixedIds {
    fn new(fail_at: Option<usize>) -> Self {
        Self {
            values: [[0x11; 16], [0x12; 16], [0x13; 16], [0x14; 16]],
            fail_at,
            calls: 0,
        }
    }
}

impl UuidGenerator for FixedIds {
    type Error = &'static str;

    fn generate(&mut self) -> Result<[u8; 16], Self::Error> {
        let index = self.calls;
        self.calls += 1;
        if self.fail_at == Some(index) {
            return Err(PRIVATE_ID_MARKER);
        }
        self.values
            .get(index)
            .copied()
            .ok_or("UUID_SOURCE_EXHAUSTED")
    }
}

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

#[derive(Debug, PartialEq, Eq)]
struct Persisted {
    receivable_id: [u8; 16],
    receipt_id: [u8; 16],
    receipt_event_id: [u8; 16],
    movement_id: [u8; 16],
    movement_event_id: [u8; 16],
    account_id: [u8; 16],
    amount_cents: i64,
    receipt_context: u8,
    movement_context: u8,
    occurred_at_ms: i64,
}

#[derive(Default)]
struct FakeStore {
    calls: usize,
    failure: Option<RecordReceivableReceiptError>,
    persisted: Option<Persisted>,
}

impl ReceivableReceiptRecordingStore for FakeStore {
    fn persist_receipt_and_entry(
        &mut self,
        write: ReceivableReceiptWrite<'_>,
    ) -> Result<(), RecordReceivableReceiptError> {
        self.calls += 1;
        let receipt_context = match &write.receipt.context {
            Context::Home => 0,
            Context::Factory => 1,
        };
        let movement_context = match &write.movement.context {
            Context::Home => 0,
            Context::Factory => 1,
        };
        let account_id = match &write.movement.kind {
            MovementKind::Entry { account_id } => *account_id,
            _ => panic!("expected entry movement"),
        };
        self.persisted = Some(Persisted {
            receivable_id: write.receivable_id,
            receipt_id: write.receipt_id,
            receipt_event_id: write.receipt_event_id,
            movement_id: write.movement.id,
            movement_event_id: write.movement_event_id,
            account_id,
            amount_cents: write.receipt.amount_cents,
            receipt_context,
            movement_context,
            occurred_at_ms: write.occurred_at_ms,
        });
        self.failure.take().map_or(Ok(()), Err)
    }
}

fn receipt(account_id: [u8; 16], amount_cents: i64, context: Context) -> ReceivableReceipt {
    ReceivableReceipt {
        account_id,
        amount_cents,
        context,
    }
}

#[test]
fn success_preserves_receipt_fields_in_entry_and_store_write() {
    for amount_cents in [1, i64::MAX] {
        for context_key in 0..2 {
            let context = if context_key == 0 {
                Context::Home
            } else {
                Context::Factory
            };
            let mut ids = FixedIds::new(None);
            let clock = FixedClock::new(UNIX_EPOCH + Duration::from_millis(1234));
            let mut store = FakeStore::default();
            let movement = record_receivable_receipt(
                [0x21; 16],
                receipt([0x22; 16], amount_cents, context),
                &mut ids,
                &clock,
                &mut store,
            )
            .expect("valid receipt must be recorded");

            assert_eq!(movement.id, [0x13; 16]);
            assert_eq!(movement.amount_cents, amount_cents);
            assert!(matches!(
                &movement.kind,
                MovementKind::Entry { account_id } if *account_id == [0x22; 16]
            ));
            assert!(matches!(
                (&movement.context, context_key),
                (Context::Home, 0) | (Context::Factory, 1)
            ));
            assert_eq!(
                store.persisted,
                Some(Persisted {
                    receivable_id: [0x21; 16],
                    receipt_id: [0x11; 16],
                    receipt_event_id: [0x12; 16],
                    movement_id: [0x13; 16],
                    movement_event_id: [0x14; 16],
                    account_id: [0x22; 16],
                    amount_cents,
                    receipt_context: context_key,
                    movement_context: context_key,
                    occurred_at_ms: 1234,
                })
            );
            assert_eq!(ids.calls, 4);
            assert_eq!(clock.calls.get(), 1);
            assert_eq!(store.calls, 1);
        }
    }
}

#[test]
fn invalid_amounts_fail_before_ids_clock_or_store() {
    for amount_cents in [0, -1, i64::MIN] {
        for context in [Context::Home, Context::Factory] {
            let mut ids = FixedIds::new(None);
            let clock = FixedClock::new(UNIX_EPOCH);
            let mut store = FakeStore::default();
            let error = record_receivable_receipt(
                [0x21; 16],
                receipt([0x22; 16], amount_cents, context),
                &mut ids,
                &clock,
                &mut store,
            )
            .err()
            .expect("invalid amount must be rejected");
            assert_eq!(error, RecordReceivableReceiptError::InvalidInput);
            assert_eq!(ids.calls, 0);
            assert_eq!(clock.calls.get(), 0);
            assert_eq!(store.calls, 0);
            assert!(store.persisted.is_none());
        }
    }
}

#[test]
fn any_identifier_failure_stops_before_clock_and_store() {
    for fail_at in 0..4 {
        let mut ids = FixedIds::new(Some(fail_at));
        let clock = FixedClock::new(UNIX_EPOCH);
        let mut store = FakeStore::default();
        let error = record_receivable_receipt(
            [0x21; 16],
            receipt([0x22; 16], 1, Context::Home),
            &mut ids,
            &clock,
            &mut store,
        )
        .err()
        .expect("identifier failure must stop the operation");
        assert_eq!(error, RecordReceivableReceiptError::IdGenerationFailed);
        assert_eq!(ids.calls, fail_at + 1);
        assert_eq!(clock.calls.get(), 0);
        assert_eq!(store.calls, 0);
        assert!(!error.to_string().contains(PRIVATE_ID_MARKER));
    }
}

#[test]
fn pre_epoch_clock_and_store_errors_are_sanitized() {
    let mut ids = FixedIds::new(None);
    let clock = FixedClock::new(UNIX_EPOCH - Duration::from_millis(1));
    let mut store = FakeStore::default();
    let clock_error = record_receivable_receipt(
        [0x21; 16],
        receipt([0x22; 16], 1, Context::Factory),
        &mut ids,
        &clock,
        &mut store,
    )
    .err()
    .expect("pre-epoch clock must fail");
    assert_eq!(clock_error, RecordReceivableReceiptError::ClockFailed);
    assert_eq!(ids.calls, 4);
    assert_eq!(clock.calls.get(), 1);
    assert_eq!(store.calls, 0);

    let mut ids = FixedIds::new(None);
    let clock = FixedClock::new(UNIX_EPOCH);
    let mut store = FakeStore {
        failure: Some(RecordReceivableReceiptError::AmountExceedsDue),
        ..Default::default()
    };
    let error = record_receivable_receipt(
        [0x21; 16],
        receipt([0x22; 16], 1, Context::Home),
        &mut ids,
        &clock,
        &mut store,
    )
    .err()
    .expect("store rejection must propagate");
    assert_eq!(error, RecordReceivableReceiptError::AmountExceedsDue);
    assert!(!error.to_string().contains("PRIVATE_"));
}

#[test]
fn errors_have_static_messages_and_no_source_payload() {
    let errors = [
        (
            RecordReceivableReceiptError::InvalidInput,
            "receipt amount is invalid",
        ),
        (
            RecordReceivableReceiptError::IdGenerationFailed,
            "receipt identifiers could not be generated",
        ),
        (
            RecordReceivableReceiptError::ClockFailed,
            "receipt timestamp is invalid",
        ),
        (
            RecordReceivableReceiptError::ReceivableNotFound,
            "receivable could not be found",
        ),
        (
            RecordReceivableReceiptError::AmountExceedsDue,
            "receipt amount exceeds the due balance",
        ),
        (
            RecordReceivableReceiptError::StoreFailed,
            "receipt and entry could not be stored",
        ),
    ];

    for (error, expected_message) in errors {
        assert_eq!(error.to_string(), expected_message);
        assert!(std::error::Error::source(&error).is_none());
    }
}
