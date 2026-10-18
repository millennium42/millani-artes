use super::{
    duration_to_epoch_millis, rename_category, CategoryRenamingStore, RenameCategoryError,
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

type PersistedCategory = ([u8; 16], String, [u8; 16], i64, bool);

#[derive(Default)]
struct FakeStore {
    calls: usize,
    persisted: Option<PersistedCategory>,
    not_found: bool,
    fail: bool,
    is_active: bool,
}

impl CategoryRenamingStore for FakeStore {
    type Error = &'static str;
    fn rename_category_with_audit(
        &mut self,
        id: [u8; 16],
        name: &str,
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
        self.persisted = Some((
            id,
            name.to_string(),
            event_id,
            occurred_at_ms,
            self.is_active,
        ));
        Ok(Some(Category {
            id,
            name: name.to_string(),
            is_active: self.is_active,
        }))
    }
}

fn ids(fail_at: Option<usize>) -> FixedIds {
    FixedIds { calls: 0, fail_at }
}

#[test]
fn preserves_category_is_active_state() {
    for active in [true, false] {
        let name = "  Novo Nome '🧵'\0  ".to_string();
        let clock = FixedClock::new(UNIX_EPOCH + Duration::from_millis(1234));
        let mut ids = ids(None);
        let mut store = FakeStore {
            is_active: active,
            ..FakeStore::default()
        };
        let category =
            rename_category([0xD1; 16], name.clone(), &mut ids, &clock, &mut store).unwrap();
        assert_eq!(category.id, [0xD1; 16]);
        assert_eq!(category.name, name);
        assert_eq!(category.is_active, active);
        assert_eq!(
            store.persisted,
            Some(([0xD1; 16], name, [0xA1; 16], 1234, active))
        );
        assert_eq!((ids.calls, clock.calls.get(), store.calls), (1, 1, 1));
    }
}

#[test]
fn invalid_names_fail_before_any_port() {
    for name in ["", " \t\r\n", "\u{2003}\u{00A0}"] {
        let clock = FixedClock::new(UNIX_EPOCH);
        let mut ids = ids(None);
        let mut store = FakeStore::default();
        let error = rename_category([0xA1; 16], name.into(), &mut ids, &clock, &mut store)
            .err()
            .unwrap();
        assert_eq!(error, RenameCategoryError::InvalidName);
        assert_eq!((ids.calls, clock.calls.get(), store.calls), (0, 0, 0));
    }
}

#[test]
fn either_id_failure_prevents_clock_and_store() {
    let clock = FixedClock::new(UNIX_EPOCH);
    let mut ids = ids(Some(0));
    let mut store = FakeStore::default();
    let error = rename_category([0xA1; 16], "Nome".into(), &mut ids, &clock, &mut store)
        .err()
        .unwrap();
    assert_eq!(error, RenameCategoryError::IdGenerationFailed);
    assert_eq!((ids.calls, clock.calls.get(), store.calls), (1, 0, 0));
}

#[test]
fn pre_epoch_clock_prevents_store() {
    let clock = FixedClock::new(UNIX_EPOCH - Duration::from_millis(1));
    let mut ids = ids(None);
    let mut store = FakeStore::default();
    let error = rename_category([0xA1; 16], "Nome".into(), &mut ids, &clock, &mut store)
        .err()
        .unwrap();
    assert_eq!(error, RenameCategoryError::ClockFailed);
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
        Err(RenameCategoryError::ClockFailed)
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
    let error = rename_category([0xA1; 16], "Nome".into(), &mut ids, &clock, &mut store)
        .err()
        .unwrap();
    assert_eq!(error, RenameCategoryError::NotFound);
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
    let error = rename_category([0xA1; 16], "Nome".into(), &mut ids, &clock, &mut store)
        .err()
        .unwrap();
    assert_eq!(error, RenameCategoryError::StoreFailed);
    assert_eq!((ids.calls, clock.calls.get(), store.calls), (1, 1, 1));
}

#[test]
fn all_error_traits_are_opaque() {
    for (error, debug, display) in [
        (
            RenameCategoryError::InvalidName,
            "InvalidName",
            "category name is invalid",
        ),
        (
            RenameCategoryError::IdGenerationFailed,
            "IdGenerationFailed",
            "category audit identifier could not be generated",
        ),
        (
            RenameCategoryError::ClockFailed,
            "ClockFailed",
            "category timestamp is invalid",
        ),
        (
            RenameCategoryError::NotFound,
            "NotFound",
            "category was not found",
        ),
        (
            RenameCategoryError::StoreFailed,
            "StoreFailed",
            "category could not be renamed",
        ),
    ] {
        assert_eq!(format!("{error:?}"), debug);
        assert_eq!(error.to_string(), display);
        assert!(error.source().is_none());
    }
}
