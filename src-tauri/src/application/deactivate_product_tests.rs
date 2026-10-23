use super::{
    deactivate_product, duration_to_epoch_millis, DeactivateProductError, ProductDeactivationStore,
};
use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::Product;
use std::cell::Cell;
use std::error::Error;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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

impl UuidGenerator for FixedIds {
    type Error = &'static str;
    fn generate(&mut self) -> Result<[u8; 16], Self::Error> {
        let call = self.calls;
        self.calls += 1;
        if self.fail_at == Some(call) {
            return Err("PRIVATE_ID_FAILURE");
        }
        [[0xA1; 16], [0xB2; 16]]
            .get(call)
            .copied()
            .ok_or("EXHAUSTED")
    }
}

type PersistedProduct = ([u8; 16], [u8; 16], i64);

#[derive(Default)]
struct FakeStore {
    calls: usize,
    persisted: Option<PersistedProduct>,
    not_found: bool,
    fail: bool,
    name: String,
    description: String,
    suggested_price_cents: i64,
}

impl ProductDeactivationStore for FakeStore {
    type Error = &'static str;
    fn deactivate_product_with_audit(
        &mut self,
        id: [u8; 16],
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<Option<Product>, Self::Error> {
        self.calls += 1;
        if self.fail {
            return Err("PRIVATE_STORE_FAILURE");
        }
        if self.not_found {
            return Ok(None);
        }
        self.persisted = Some((id, event_id, occurred_at_ms));
        Ok(Some(Product {
            id,
            name: self.name.clone(),
            description: self.description.clone(),
            suggested_price_cents: self.suggested_price_cents,
            is_active: false,
        }))
    }
}

fn ids(fail_at: Option<usize>) -> FixedIds {
    FixedIds { calls: 0, fail_at }
}

#[test]
fn returns_stored_product_and_exact_audit_metadata() {
    for price in [0, 1, i64::MAX] {
        let name = "  Café '🧵'\0  ".to_string();
        let description = "  Descrição\0 livre\r\n  ".to_string();
        let clock = FixedClock::new(UNIX_EPOCH + Duration::from_millis(1234));
        let mut ids = ids(None);
        let mut store = FakeStore {
            name: name.clone(),
            description: description.clone(),
            suggested_price_cents: price,
            ..FakeStore::default()
        };
        let product = deactivate_product([0xD1; 16], &mut ids, &clock, &mut store).unwrap();
        assert_eq!(product.id, [0xD1; 16]);
        assert_eq!(product.name, name);
        assert_eq!(product.description, description);
        assert_eq!(product.suggested_price_cents, price);
        assert!(!product.is_active);
        assert_eq!(store.persisted, Some(([0xD1; 16], [0xA1; 16], 1234)));
        assert_eq!((ids.calls, clock.calls.get(), store.calls), (1, 1, 1));
    }
}

#[test]
fn either_id_failure_prevents_clock_and_store() {
    let clock = FixedClock::new(UNIX_EPOCH);
    let mut ids = ids(Some(0));
    let mut store = FakeStore::default();
    let error = deactivate_product([0xD1; 16], &mut ids, &clock, &mut store)
        .err()
        .unwrap();
    assert_eq!(error, DeactivateProductError::IdGenerationFailed);
    assert_eq!((ids.calls, clock.calls.get(), store.calls), (1, 0, 0));
}

#[test]
fn pre_epoch_clock_prevents_store() {
    let clock = FixedClock::new(UNIX_EPOCH - Duration::from_millis(1));
    let mut ids = ids(None);
    let mut store = FakeStore::default();
    let error = deactivate_product([0xD1; 16], &mut ids, &clock, &mut store)
        .err()
        .unwrap();
    assert_eq!(error, DeactivateProductError::ClockFailed);
    assert_eq!((ids.calls, clock.calls.get(), store.calls), (1, 1, 0));
}

#[test]
fn epoch_milliseconds_accept_max_and_reject_overflow() {
    assert_eq!(
        duration_to_epoch_millis(Duration::from_millis(i64::MAX as u64)),
        Ok(i64::MAX)
    );
    assert_eq!(
        duration_to_epoch_millis(Duration::from_millis(i64::MAX as u64 + 1)),
        Err(DeactivateProductError::ClockFailed)
    );
}

#[test]
fn store_returns_none_product_not_found() {
    let clock = FixedClock::new(UNIX_EPOCH);
    let mut ids = ids(None);
    let mut store = FakeStore {
        not_found: true,
        ..FakeStore::default()
    };
    let error = deactivate_product([0xD1; 16], &mut ids, &clock, &mut store)
        .err()
        .unwrap();
    assert_eq!(error, DeactivateProductError::NotFound);
    assert_eq!((ids.calls, clock.calls.get(), store.calls), (1, 1, 1));
}

#[test]
fn store_failure_returns_store_failed() {
    let clock = FixedClock::new(UNIX_EPOCH);
    let mut ids = ids(None);
    let mut store = FakeStore {
        fail: true,
        ..FakeStore::default()
    };
    let error = deactivate_product([0xD1; 16], &mut ids, &clock, &mut store)
        .err()
        .unwrap();
    assert_eq!(error, DeactivateProductError::StoreFailed);
    assert_eq!((ids.calls, clock.calls.get(), store.calls), (1, 1, 1));
}

#[test]
fn all_error_traits_are_opaque() {
    for (error, debug, display) in [
        (
            DeactivateProductError::IdGenerationFailed,
            "IdGenerationFailed",
            "product audit identifier could not be generated",
        ),
        (
            DeactivateProductError::ClockFailed,
            "ClockFailed",
            "product timestamp is invalid",
        ),
        (
            DeactivateProductError::NotFound,
            "NotFound",
            "product was not found",
        ),
        (
            DeactivateProductError::StoreFailed,
            "StoreFailed",
            "product could not be deactivated",
        ),
    ] {
        assert_eq!(format!("{error:?}"), debug);
        assert_eq!(error.to_string(), display);
        assert!(error.source().is_none());
    }
}
