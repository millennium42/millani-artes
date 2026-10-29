use super::*;
use crate::domain::ports::{Clock, UuidGenerator};
use std::cell::Cell;
use std::time::{Duration, SystemTime};

struct Ids {
    calls: usize,
    fail: bool,
}
impl UuidGenerator for Ids {
    type Error = ();
    fn generate(&mut self) -> Result<[u8; 16], ()> {
        self.calls += 1;
        if self.fail {
            Err(())
        } else {
            Ok([self.calls as u8; 16])
        }
    }
}

struct Time {
    value: SystemTime,
    calls: Cell<usize>,
}
impl Clock for Time {
    fn now(&self) -> SystemTime {
        self.calls.set(self.calls.get() + 1);
        self.value
    }
}

struct Store {
    write: Option<RemoveSaleItemWrite>,
    calls: usize,
    fail: bool,
}
impl SaleItemRemovalStore for Store {
    fn remove_sale_item_with_audit(&mut self, write: RemoveSaleItemWrite) -> Result<(), SaleError> {
        self.calls += 1;
        self.write = Some(write);
        if self.fail {
            Err(SaleError::StoreFailed)
        } else {
            Ok(())
        }
    }
}

fn fixtures() -> (Ids, Time, Store) {
    (
        Ids {
            calls: 0,
            fail: false,
        },
        Time {
            value: SystemTime::UNIX_EPOCH + Duration::from_millis(1234),
            calls: Cell::new(0),
        },
        Store {
            write: None,
            calls: 0,
            fail: false,
        },
    )
}

#[test]
fn sal014_forwards_ids_and_checked_event_time_once() {
    let (mut ids, clock, mut store) = fixtures();
    remove_sale_item(
        RemoveSaleItemInput {
            sale_id: [8; 16],
            item_id: [9; 16],
        },
        &mut ids,
        &clock,
        &mut store,
    )
    .unwrap();
    let write = store.write.unwrap();
    assert_eq!(write.sale_id, [8; 16]);
    assert_eq!(write.item_id, [9; 16]);
    assert_eq!(write.event_id, [1; 16]);
    assert_eq!(write.occurred_at_ms, 1234);
    assert_eq!(ids.calls, 1);
    assert_eq!(clock.calls.get(), 1);
    assert_eq!(store.calls, 1);
}

#[test]
fn sal014_id_and_clock_failures_stop_before_the_store() {
    let (mut ids, mut clock, mut store) = fixtures();
    ids.fail = true;
    assert_eq!(
        remove_sale_item(
            RemoveSaleItemInput {
                sale_id: [8; 16],
                item_id: [9; 16],
            },
            &mut ids,
            &clock,
            &mut store,
        )
        .err(),
        Some(SaleError::IdGenerationFailed)
    );
    assert_eq!(ids.calls, 1);
    assert_eq!(clock.calls.get(), 0);
    assert_eq!(store.calls, 0);

    ids.fail = false;
    clock.value = SystemTime::UNIX_EPOCH - Duration::from_millis(1);
    assert_eq!(
        remove_sale_item(
            RemoveSaleItemInput {
                sale_id: [8; 16],
                item_id: [9; 16],
            },
            &mut ids,
            &clock,
            &mut store,
        )
        .err(),
        Some(SaleError::ClockFailed)
    );
    assert_eq!(clock.calls.get(), 1);
    assert_eq!(store.calls, 0);
    assert_eq!(
        duration_to_epoch_millis(Duration::from_millis(i64::MAX as u64)),
        Ok(i64::MAX)
    );
    assert_eq!(
        duration_to_epoch_millis(Duration::from_millis(i64::MAX as u64 + 1)),
        Err(SaleError::ClockFailed)
    );
}

#[test]
fn sal014_store_failure_and_sale_errors_remain_opaque() {
    let (mut ids, clock, mut store) = fixtures();
    store.fail = true;
    assert_eq!(
        remove_sale_item(
            RemoveSaleItemInput {
                sale_id: [8; 16],
                item_id: [9; 16],
            },
            &mut ids,
            &clock,
            &mut store,
        )
        .err(),
        Some(SaleError::StoreFailed)
    );
    for error in [
        SaleError::IdGenerationFailed,
        SaleError::ClockFailed,
        SaleError::SaleNotFound,
        SaleError::SaleNotOpen,
        SaleError::SaleItemNotFound,
        SaleError::ProductNotFound,
        SaleError::InvalidItem,
        SaleError::PositionExhausted,
        SaleError::StoreFailed,
    ] {
        assert!(!format!("{error:?} {error}").contains("PRIVATE"));
        assert!(std::error::Error::source(&error).is_none());
    }
}
