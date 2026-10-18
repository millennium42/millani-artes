use super::{
    deactivate_category, duration_to_epoch_millis, CategoryDeactivationStore,
    DeactivateCategoryError,
};
use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::Category;
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

type PersistedCategory = ([u8; 16], [u8; 16], i64);

#[derive(Default)]
struct FakeStore {
    calls: usize,
    persisted: Option<PersistedCategory>,
    not_found: bool,
    fail: bool,
    name: String,
}

impl CategoryDeactivationStore for FakeStore {
    type Error = &'static str;
    fn deactivate_category_with_audit(
        &mut self,
        id: [u8; 16],
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<Option<Category>, Self::Error> {
        self.calls += 1;
        if self.fail {
            return Err("PRIVATE_STORE_FAILURE");
        }
        if self.not_found {
            return Ok(None);
        }
        self.persisted = Some((id, event_id, occurred_at_ms));
        Ok(Some(Category {
            id,
            name: self.name.clone(),
            is_active: false,
        }))
    }
}

fn ids(fail_at: Option<usize>) -> FixedIds {
    FixedIds { calls: 0, fail_at }
}

#[test]
fn returns_stored_category_and_exact_audit_metadata() {
    let name = "  Café '🧵'\0  ".to_string();
    let clock = FixedClock::new(UNIX_EPOCH + Duration::from_millis(1234));
    let mut ids = ids(None);
    let mut store = FakeStore {
        name: name.clone(),
        ..FakeStore::default()
    };
    let category = deactivate_category([0xD1; 16], &mut ids, &clock, &mut store).unwrap();
    assert_eq!(category.id, [0xD1; 16]);
    assert_eq!(category.name, name);
    assert!(!category.is_active);
    assert_eq!(store.persisted, Some(([0xD1; 16], [0xA1; 16], 1234)));
    assert_eq!((ids.calls, clock.calls.get(), store.calls), (1, 1, 1));
}

#[test]
fn either_id_failure_prevents_clock_and_store() {
    let clock = FixedClock::new(UNIX_EPOCH);
    let mut ids = ids(Some(0));
    let mut store = FakeStore::default();
    let error = deactivate_category([0xD1; 16], &mut ids, &clock, &mut store)
        .err()
        .unwrap();
    assert_eq!(error, DeactivateCategoryError::IdGenerationFailed);
    assert_eq!((ids.calls, clock.calls.get(), store.calls), (1, 0, 0));
}

#[test]
fn pre_epoch_clock_prevents_store() {
    let clock = FixedClock::new(UNIX_EPOCH - Duration::from_millis(1));
    let mut ids = ids(None);
    let mut store = FakeStore::default();
    let error = deactivate_category([0xD1; 16], &mut ids, &clock, &mut store)
        .err()
        .unwrap();
    assert_eq!(error, DeactivateCategoryError::ClockFailed);
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
        Err(DeactivateCategoryError::ClockFailed)
    );
}

#[test]
fn store_returns_none_category_not_found() {
    let clock = FixedClock::new(UNIX_EPOCH);
    let mut ids = ids(None);
    let mut store = FakeStore {
        not_found: true,
        ..FakeStore::default()
    };
    let error = deactivate_category([0xD1; 16], &mut ids, &clock, &mut store)
        .err()
        .unwrap();
    assert_eq!(error, DeactivateCategoryError::NotFound);
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
    let error = deactivate_category([0xD1; 16], &mut ids, &clock, &mut store)
        .err()
        .unwrap();
    assert_eq!(error, DeactivateCategoryError::StoreFailed);
    assert_eq!((ids.calls, clock.calls.get(), store.calls), (1, 1, 1));
}

#[test]
fn all_error_traits_are_opaque() {
    for (error, debug, display) in [
        (
            DeactivateCategoryError::IdGenerationFailed,
            "IdGenerationFailed",
            "category audit identifier could not be generated",
        ),
        (
            DeactivateCategoryError::ClockFailed,
            "ClockFailed",
            "category timestamp is invalid",
        ),
        (
            DeactivateCategoryError::NotFound,
            "NotFound",
            "category was not found",
        ),
        (
            DeactivateCategoryError::StoreFailed,
            "StoreFailed",
            "category could not be deactivated",
        ),
    ] {
        assert_eq!(format!("{error:?}"), debug);
        assert_eq!(error.to_string(), display);
        assert!(error.source().is_none());
    }
}
