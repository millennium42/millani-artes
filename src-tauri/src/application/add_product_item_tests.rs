use super::*;
use crate::domain::{
    ports::{Clock, UuidGenerator},
    SaleStatus,
};
use std::{
    cell::Cell,
    time::{Duration, SystemTime},
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
    write: Option<ProductItemWrite>,
    read_id: Option<[u8; 16]>,
    fail: bool,
}
fn item() -> SaleItem {
    SaleItem {
        id: [91; 16],
        product_id: Some([92; 16]),
        description: "PRIVATE\0 🧵".into(),
        quantity: 2,
        unit_price_cents: 50,
        discount_cents: 1,
        total_cents: 99,
    }
}
impl SaleStore for Store {
    fn persist_product_item_with_audit(
        &mut self,
        w: ProductItemWrite,
    ) -> Result<SaleItem, SaleError> {
        self.write = Some(w);
        if self.fail {
            Err(SaleError::StoreFailed)
        } else {
            Ok(item())
        }
    }
    fn load_sale(&mut self, id: [u8; 16]) -> Result<Option<Sale>, SaleError> {
        self.read_id = Some(id);
        if self.fail {
            Err(SaleError::StoreFailed)
        } else if id == [0; 16] {
            Ok(None)
        } else {
            Ok(Some(Sale {
                id,
                status: SaleStatus::Open,
                items: vec![item()],
            }))
        }
    }
}
fn input(sale: Option<[u8; 16]>) -> ProductItemInput {
    ProductItemInput {
        sale_id: sale,
        product_id: [9; 16],
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
            read_id: None,
            fail: false,
        },
    )
}

#[test]
fn sal012_new_and_existing_sale_forward_exact_write_metadata_and_persisted_result() {
    for sale in [None, Some([8; 16])] {
        let (mut ids, time, mut store) = fixtures();
        let r = add_product_item(input(sale), &mut ids, &time, &mut store).unwrap();
        let w = store.write.unwrap();
        assert_eq!(w.sale_id, sale.unwrap_or([1; 16]));
        assert_eq!(r.sale_id, w.sale_id);
        assert_eq!(w.create_sale, sale.is_none());
        assert_eq!(ids.calls, if sale.is_none() { 3 } else { 2 });
        assert_eq!(time.calls.get(), 1);
        assert_eq!(w.item_id, if sale.is_none() { [2; 16] } else { [1; 16] });
        assert_eq!(w.event_id, [ids.calls as u8; 16]);
        assert_eq!(w.occurred_at_ms, 1234);
        assert_eq!(w.product_id, [9; 16]);
        assert_eq!(w.quantity, 2);
        assert_eq!(w.unit_price_cents, 123);
        assert!(matches!(
            w.discount,
            DiscountInput::PercentageBasisPoints(2500)
        ));
        assert_eq!(r.item.id, [91; 16]);
        assert_eq!(r.item.description, "PRIVATE\0 🧵");
        assert_eq!(r.item.total_cents, 99);
    }
}
#[test]
fn sal012_id_failures_stop_before_clock_and_store() {
    for sale in [None, Some([8; 16])] {
        for fail in 1..=if sale.is_none() { 3 } else { 2 } {
            let (mut ids, time, mut store) = fixtures();
            ids.fail = fail;
            assert_eq!(
                add_product_item(input(sale), &mut ids, &time, &mut store).err(),
                Some(SaleError::IdGenerationFailed)
            );
            assert_eq!(ids.calls, fail);
            assert_eq!(time.calls.get(), 0);
            assert!(store.write.is_none());
        }
    }
}
#[test]
fn sal012_clock_and_epoch_millis_boundaries_are_checked() {
    for (value, expected) in [
        (UNIX_EPOCH - Duration::from_millis(1), None),
        (UNIX_EPOCH, Some(0)),
    ] {
        let (mut ids, mut time, mut store) = fixtures();
        time.value = value;
        let r = add_product_item(input(None), &mut ids, &time, &mut store);
        if let Some(ms) = expected {
            assert!(r.is_ok());
            assert_eq!(store.write.unwrap().occurred_at_ms, ms)
        } else {
            assert_eq!(r.err(), Some(SaleError::ClockFailed));
            assert!(store.write.is_none())
        }
        assert_eq!(time.calls.get(), 1);
    }
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
fn sal012_store_errors_and_all_error_formatting_remain_opaque() {
    let (mut ids, time, mut store) = fixtures();
    store.fail = true;
    assert_eq!(
        add_product_item(input(None), &mut ids, &time, &mut store).err(),
        Some(SaleError::StoreFailed)
    );
    for e in [
        SaleError::IdGenerationFailed,
        SaleError::ClockFailed,
        SaleError::SaleNotFound,
        SaleError::SaleNotOpen,
        SaleError::ProductNotFound,
        SaleError::InvalidItem,
        SaleError::PositionExhausted,
        SaleError::StoreFailed,
    ] {
        assert!(!format!("{e:?} {e}").contains("PRIVATE"));
        assert!(std::error::Error::source(&e).is_none());
    }
}
#[test]
fn sal012_read_preserves_some_none_and_failure() {
    let (_, _, mut store) = fixtures();
    let r = read_sale(&mut store, [7; 16]).unwrap().unwrap();
    assert_eq!(r.id, [7; 16]);
    assert_eq!(r.items[0].total_cents, 99);
    assert_eq!(store.read_id, Some([7; 16]));
    assert!(read_sale(&mut store, [0; 16]).unwrap().is_none());
    store.fail = true;
    assert_eq!(
        read_sale(&mut store, [7; 16]).err(),
        Some(SaleError::StoreFailed)
    );
}
