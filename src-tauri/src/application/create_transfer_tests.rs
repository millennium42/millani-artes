use super::{
    create_transfer, duration_to_epoch_millis, CreateTransferError, TransferCreationStore,
};
use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::{Context, Movement, MovementKind};
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

type PersistedTransfer = ([u8; 16], [u8; 16], [u8; 16], i64, u8, [u8; 16], i64);

#[derive(Default)]
struct FakeStore {
    calls: usize,
    fail: bool,
    persisted: Option<PersistedTransfer>,
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

impl TransferCreationStore for FakeStore {
    type Error = StoreFailure;

    fn persist_transfer_with_audit(
        &mut self,
        movement: &Movement,
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<(), Self::Error> {
        self.calls += 1;
        let (source_account_id, destination_account_id) = match &movement.kind {
            MovementKind::Transfer {
                source_account_id,
                destination_account_id,
            } => (*source_account_id, *destination_account_id),
            _ => panic!("expected an transfer"),
        };
        let context = match &movement.context {
            Context::Home => 0,
            Context::Factory => 1,
        };
        self.persisted = Some((
            movement.id,
            source_account_id,
            destination_account_id,
            movement.amount_cents,
            context,
            event_id,
            occurred_at_ms,
        ));
        if self.fail {
            Err(StoreFailure(PRIVATE_STORE_MARKER))
        } else {
            Ok(())
        }
    }
}

#[test]
fn success_persists_all_fields_for_both_contexts_and_amount_bounds() {
    for amount in [1, i64::MAX] {
        for context_key in 0..2 {
            let context = if context_key == 0 {
                Context::Home
            } else {
                Context::Factory
            };
            let mut ids = FixedIds::new(None);
            let clock = FixedClock::new(UNIX_EPOCH + Duration::from_millis(1234));
            let mut store = FakeStore::default();
            let movement = create_transfer(
                [3; 16], [4; 16], amount, context, &mut ids, &clock, &mut store,
            )
            .expect("valid transfer must be persisted");
            assert_eq!(movement.id, [1; 16]);
            assert_eq!(movement.amount_cents, amount);
            assert!(
                matches!(&movement.kind, MovementKind::Transfer { source_account_id, destination_account_id } if *source_account_id == [3; 16] && *destination_account_id == [4; 16])
            );
            assert!(matches!(
                (&movement.context, context_key),
                (Context::Home, 0) | (Context::Factory, 1)
            ));
            assert_eq!(
                store.persisted,
                Some((
                    [1; 16],
                    [3; 16],
                    [4; 16],
                    amount,
                    context_key,
                    [2; 16],
                    1234
                ))
            );
            assert_eq!(ids.calls, 2);
            assert_eq!(clock.calls.get(), 1);
            assert_eq!(store.calls, 1);
        }
    }
}

#[test]
fn invalid_amounts_fail_before_any_side_effect_in_both_contexts() {
    for amount in [0, -1, i64::MIN] {
        for context_key in 0..2 {
            let context = if context_key == 0 {
                Context::Home
            } else {
                Context::Factory
            };
            let mut ids = FixedIds::new(None);
            let clock = FixedClock::new(UNIX_EPOCH);
            let mut store = FakeStore::default();
            let error = create_transfer(
                [3; 16], [4; 16], amount, context, &mut ids, &clock, &mut store,
            )
            .err()
            .expect("invalid amount must fail");
            assert!(matches!(error, CreateTransferError::InvalidInput));
            assert_eq!(ids.calls, 0);
            assert_eq!(clock.calls.get(), 0);
            assert_eq!(store.calls, 0);
            assert!(store.persisted.is_none());
        }
    }
}

#[test]
fn either_id_failure_stops_before_clock_and_store() {
    for fail_at in [0, 1] {
        let mut ids = FixedIds::new(Some(fail_at));
        let clock = FixedClock::new(UNIX_EPOCH);
        let mut store = FakeStore::default();
        let error = create_transfer(
            [3; 16],
            [4; 16],
            1,
            Context::Home,
            &mut ids,
            &clock,
            &mut store,
        )
        .err()
        .expect("identifier failure must be mapped");
        assert!(matches!(error, CreateTransferError::IdGenerationFailed));
        assert_eq!(ids.calls, fail_at + 1);
        assert_eq!(clock.calls.get(), 0);
        assert_eq!(store.calls, 0);
        assert!(store.persisted.is_none());
        assert!(!format!("{error}").contains(PRIVATE_ID_MARKER));
    }
}

#[test]
fn pre_epoch_clock_failure_stops_before_store() {
    let mut ids = FixedIds::new(None);
    let clock = FixedClock::new(UNIX_EPOCH - Duration::from_millis(1));
    let mut store = FakeStore::default();
    let error = create_transfer(
        [3; 16],
        [4; 16],
        1,
        Context::Factory,
        &mut ids,
        &clock,
        &mut store,
    )
    .err()
    .expect("pre-epoch timestamp must fail");
    assert!(matches!(error, CreateTransferError::ClockFailed));
    assert_eq!(ids.calls, 2);
    assert_eq!(clock.calls.get(), 1);
    assert_eq!(store.calls, 0);
    assert!(store.persisted.is_none());
}

#[test]
fn store_failure_and_public_errors_do_not_expose_private_details() {
    let mut ids = FixedIds::new(None);
    let clock = FixedClock::new(UNIX_EPOCH);
    let mut store = FakeStore {
        fail: true,
        ..Default::default()
    };
    let error = create_transfer(
        [3; 16],
        [4; 16],
        1,
        Context::Home,
        &mut ids,
        &clock,
        &mut store,
    )
    .err()
    .expect("store failure must be mapped");
    assert!(matches!(error, CreateTransferError::StoreFailed));
    assert_eq!(ids.calls, 2);
    assert_eq!(clock.calls.get(), 1);
    assert_eq!(store.calls, 1);
    assert_eq!(
        store.persisted,
        Some(([1; 16], [3; 16], [4; 16], 1, 0, [2; 16], 0))
    );
    for error in [
        CreateTransferError::InvalidInput,
        CreateTransferError::IdGenerationFailed,
        CreateTransferError::ClockFailed,
        error,
    ] {
        let display = format!("{error}");
        let debug = format!("{error:?}");
        assert!(!display.is_empty());
        assert!(!debug.is_empty());
        for marker in [PRIVATE_STORE_MARKER, PRIVATE_ID_MARKER, "PRIVATE_"] {
            assert!(!display.contains(marker));
            assert!(!debug.contains(marker));
        }
        assert!(error.source().is_none());
    }
}

#[test]
fn duration_conversion_accepts_max_millis_and_rejects_next_millisecond() {
    assert_eq!(
        duration_to_epoch_millis(Duration::from_millis(i64::MAX as u64)),
        Ok(i64::MAX)
    );
    assert!(matches!(
        duration_to_epoch_millis(Duration::from_millis(i64::MAX as u64 + 1)),
        Err(CreateTransferError::ClockFailed)
    ));
}

#[test]
fn equal_accounts_fail_before_any_port_in_both_contexts() {
    for account in [[0; 16], [u8::MAX; 16]] {
        for context in [Context::Home, Context::Factory] {
            let mut ids = FixedIds::new(None);
            let clock = FixedClock::new(UNIX_EPOCH);
            let mut store = FakeStore::default();
            let error = create_transfer(
                account,
                account,
                i64::MAX,
                context,
                &mut ids,
                &clock,
                &mut store,
            )
            .err()
            .expect("equal accounts rejected");
            assert_eq!(error, CreateTransferError::InvalidInput);
            assert_eq!(ids.calls, 0);
            assert_eq!(clock.calls.get(), 0);
            assert_eq!(store.calls, 0);
            assert!(store.persisted.is_none());
        }
    }
}
