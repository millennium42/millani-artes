use super::{
    create_category, duration_to_epoch_millis, CategoryCreationStore, CreateCategoryError,
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

type PersistedCategory = ([u8; 16], String, bool, [u8; 16], i64);

#[derive(Default)]
struct FakeStore {
    calls: usize,
    fail: bool,
    persisted: Option<PersistedCategory>,
}

impl CategoryCreationStore for FakeStore {
    type Error = &'static str;
    fn persist_category_with_audit(
        &mut self,
        category: &Category,
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<(), Self::Error> {
        self.calls += 1;
        self.persisted = Some((
            category.id,
            category.name.clone(),
            category.is_active,
            event_id,
            occurred_at_ms,
        ));
        if self.fail {
            Err("PRIVATE_STORE_FAILURE")
        } else {
            Ok(())
        }
    }
}

fn ids(fail_at: Option<usize>) -> FixedIds {
    FixedIds { calls: 0, fail_at }
}

#[test]
fn creates_active_category_preserving_name_and_audit_metadata() {
    let name = "  Ateliê '🧵'\0  ".to_string();
    let clock = FixedClock::new(UNIX_EPOCH + Duration::from_millis(1234));
    let mut ids = ids(None);
    let mut store = FakeStore::default();
    let category = create_category(name.clone(), &mut ids, &clock, &mut store).unwrap();
    assert_eq!(category.id, [0xA1; 16]);
    assert_eq!(category.name, name);
    assert!(category.is_active);
    assert_eq!(
        store.persisted,
        Some(([0xA1; 16], name, true, [0xB2; 16], 1234))
    );
    assert_eq!((ids.calls, clock.calls.get(), store.calls), (2, 1, 1));
}

#[test]
fn invalid_names_fail_before_any_port() {
    for name in ["", " \t\r\n", "\u{2003}\u{00A0}"] {
        let clock = FixedClock::new(UNIX_EPOCH);
        let mut ids = ids(None);
        let mut store = FakeStore::default();
        let error = create_category(name.into(), &mut ids, &clock, &mut store)
            .err()
            .unwrap();
        assert_eq!(error, CreateCategoryError::InvalidName);
        assert_eq!((ids.calls, clock.calls.get(), store.calls), (0, 0, 0));
    }
}

#[test]
fn either_id_failure_prevents_clock_and_store() {
    for fail_at in [0, 1] {
        let clock = FixedClock::new(UNIX_EPOCH);
        let mut ids = ids(Some(fail_at));
        let mut store = FakeStore::default();
        let error = create_category("Categoria".into(), &mut ids, &clock, &mut store)
            .err()
            .unwrap();
        assert_eq!(error, CreateCategoryError::IdGenerationFailed);
        assert_eq!(
            (ids.calls, clock.calls.get(), store.calls),
            (fail_at + 1, 0, 0)
        );
    }
}

#[test]
fn pre_epoch_clock_prevents_store() {
    let clock = FixedClock::new(UNIX_EPOCH - Duration::from_millis(1));
    let mut ids = ids(None);
    let mut store = FakeStore::default();
    let error = create_category("Categoria".into(), &mut ids, &clock, &mut store)
        .err()
        .unwrap();
    assert_eq!(error, CreateCategoryError::ClockFailed);
    assert_eq!((ids.calls, clock.calls.get(), store.calls), (2, 1, 0));
}

#[test]
fn epoch_milliseconds_accept_max_and_reject_overflow() {
    assert_eq!(
        duration_to_epoch_millis(Duration::from_millis(i64::MAX as u64)),
        Ok(i64::MAX)
    );
    assert_eq!(
        duration_to_epoch_millis(Duration::from_millis(i64::MAX as u64 + 1)),
        Err(CreateCategoryError::ClockFailed)
    );
}

#[test]
fn store_failure_and_all_error_traits_are_opaque() {
    let clock = FixedClock::new(UNIX_EPOCH);
    let mut ids = ids(None);
    let mut store = FakeStore {
        fail: true,
        ..FakeStore::default()
    };
    let error = create_category("PRIVATE_CATEGORY_NAME".into(), &mut ids, &clock, &mut store)
        .err()
        .unwrap();
    assert_eq!(error, CreateCategoryError::StoreFailed);
    assert_eq!((ids.calls, clock.calls.get(), store.calls), (2, 1, 1));
    for (error, debug, display) in [
        (
            CreateCategoryError::InvalidName,
            "InvalidName",
            "category name is invalid",
        ),
        (
            CreateCategoryError::IdGenerationFailed,
            "IdGenerationFailed",
            "category identifiers could not be generated",
        ),
        (
            CreateCategoryError::ClockFailed,
            "ClockFailed",
            "category timestamp is invalid",
        ),
        (
            CreateCategoryError::StoreFailed,
            "StoreFailed",
            "category could not be stored",
        ),
    ] {
        assert_eq!(format!("{error:?}"), debug);
        assert_eq!(error.to_string(), display);
        assert!(error.source().is_none());
    }
}
