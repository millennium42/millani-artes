use super::*;
use crate::domain::{
    ports::{Clock, UuidGenerator},
    DiscountInput, SaleItem,
};
use std::{
    cell::Cell,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

struct Ids {
    calls: usize,
    fail: usize,
}
impl UuidGenerator for Ids {
    type Error = ();
    fn generate(&mut self) -> Result<[u8; 16], ()> {
        self.calls += 1;
        if self.calls == self.fail {
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
    write: Option<FreeItemWrite>,
    fail: bool,
}
fn persisted_item() -> SaleItem {
    SaleItem {
        id: [91; 16],
        product_id: None,
        description: "Livre privado\0 🧵".into(),
        quantity: 2,
        unit_price_cents: 50,
        discount_cents: 1,
        total_cents: 99,
    }
}
impl FreeItemStore for Store {
    fn persist_free_item_with_audit(
        &mut self,
        write: FreeItemWrite,
    ) -> Result<SaleItem, SaleError> {
        self.write = Some(write);
        if self.fail {
            Err(SaleError::StoreFailed)
        } else {
            Ok(persisted_item())
        }
    }
}
fn input(sale_id: Option<[u8; 16]>) -> FreeItemInput {
    FreeItemInput {
        sale_id,
        description: "Serviço livre\0 🧵".into(),
        quantity: 2,
        unit_price_cents: 123,
        discount: DiscountInput::PercentageBasisPoints(2500),
    }
}
fn fixtures() -> (Ids, Time, Store) {
    (
        Ids { calls: 0, fail: 0 },
        Time {
            value: UNIX_EPOCH + Duration::from_millis(1234),
            calls: Cell::new(0),
        },
        Store {
            write: None,
            fail: false,
        },
    )
}

#[test]
fn sal013_new_and_existing_sale_forward_exact_values_and_persisted_result() {
    for sale_id in [None, Some([8; 16])] {
        let (mut ids, time, mut store) = fixtures();
        let result = add_free_item(input(sale_id), &mut ids, &time, &mut store).unwrap();
        let write = store.write.unwrap();
        assert_eq!(write.sale_id, sale_id.unwrap_or([1; 16]));
        assert_eq!(result.sale_id, write.sale_id);
        assert_eq!(write.create_sale, sale_id.is_none());
        assert_eq!(ids.calls, if sale_id.is_none() { 3 } else { 2 });
        assert_eq!(time.calls.get(), 1);
        assert_eq!(
            write.item_id,
            if sale_id.is_none() { [2; 16] } else { [1; 16] }
        );
        assert_eq!(write.event_id, [ids.calls as u8; 16]);
        assert_eq!(write.occurred_at_ms, 1234);
        assert_eq!(write.description, "Serviço livre\0 🧵");
        assert_eq!(write.quantity, 2);
        assert_eq!(write.unit_price_cents, 123);
        assert!(matches!(
            write.discount,
            DiscountInput::PercentageBasisPoints(2500)
        ));
        assert_eq!(result.item.id, [91; 16]);
        assert_eq!(result.item.product_id, None);
        assert_eq!(result.item.description, "Livre privado\0 🧵");
        assert_eq!(result.item.total_cents, 99);
    }
}

#[test]
fn sal013_id_failures_stop_before_clock_and_store() {
    for sale_id in [None, Some([8; 16])] {
        for fail in 1..=if sale_id.is_none() { 3 } else { 2 } {
            let (mut ids, time, mut store) = fixtures();
            ids.fail = fail;
            assert_eq!(
                add_free_item(input(sale_id), &mut ids, &time, &mut store).err(),
                Some(SaleError::IdGenerationFailed)
            );
            assert_eq!(ids.calls, fail);
            assert_eq!(time.calls.get(), 0);
            assert!(store.write.is_none());
        }
    }
}

#[test]
fn sal013_clock_and_epoch_millis_boundaries_are_checked() {
    let (mut ids, mut time, mut store) = fixtures();
    time.value = UNIX_EPOCH - Duration::from_millis(1);
    assert_eq!(
        add_free_item(input(None), &mut ids, &time, &mut store).err(),
        Some(SaleError::ClockFailed)
    );
    assert_eq!(time.calls.get(), 1);
    assert!(store.write.is_none());
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
fn sal013_store_failure_and_errors_remain_opaque() {
    let (mut ids, time, mut store) = fixtures();
    store.fail = true;
    assert_eq!(
        add_free_item(input(None), &mut ids, &time, &mut store).err(),
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
        assert!(!format!("{error:?} {error}").contains("privado"));
        assert!(std::error::Error::source(&error).is_none());
    }
}
