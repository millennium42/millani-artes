use super::*;
use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::Context;
use std::cell::Cell;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

struct Ids {
    calls: usize,
    fail_at: Option<usize>,
    zero_at: Option<usize>,
}

impl UuidGenerator for Ids {
    type Error = ();

    fn generate(&mut self) -> Result<[u8; 16], ()> {
        self.calls += 1;
        if self.fail_at == Some(self.calls) {
            return Err(());
        }
        Ok([if self.zero_at == Some(self.calls) {
            0
        } else {
            self.calls as u8
        }; 16])
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
    write: Option<FinalizeSaleWrite>,
    result: Result<FinalizedSale, SaleError>,
}

impl SaleFinalizationStore for Store {
    fn finalize_sale_with_audit(
        &mut self,
        write: FinalizeSaleWrite,
    ) -> Result<FinalizedSale, SaleError> {
        self.write = Some(write);
        std::mem::replace(&mut self.result, Err(SaleError::StoreFailed))
    }
}

fn payment(account_id: [u8; 16], amount_cents: i64, context: Context) -> SalePayment {
    SalePayment {
        account_id,
        amount_cents,
        context,
    }
}

fn input() -> FinalizeSaleInput {
    FinalizeSaleInput {
        sale_id: [0x41; 16],
        payments: vec![
            payment([0x51; 16], 500, Context::Home),
            payment([0x52; 16], 1000, Context::Factory),
        ],
        pending: Some(PendingSaleTerms {
            customer_id: [0x61; 16],
            due_at_ms: 1_800_000_000_000,
        }),
    }
}

fn fixtures() -> (Ids, Time, Store) {
    (
        Ids {
            calls: 0,
            fail_at: None,
            zero_at: None,
        },
        Time {
            value: UNIX_EPOCH + Duration::from_millis(1234),
            calls: Cell::new(0),
        },
        Store {
            write: None,
            result: Ok(FinalizedSale {
                sale_id: [0x41; 16],
                total_cents: 3000,
                paid_cents: 1500,
                pending_cents: 1500,
            }),
        },
    )
}

#[test]
fn sal020_use_case_preserves_each_payment_and_generates_operation_ids_once() {
    let (mut ids, clock, mut store) = fixtures();
    let result = finalize_sale(input(), &mut ids, &clock, &mut store).unwrap();
    let write = store.write.unwrap();

    assert_eq!(ids.calls, 7);
    assert_eq!(clock.calls.get(), 1);
    assert_eq!(write.sale_id, [0x41; 16]);
    assert_eq!(write.sale_event_id, [1; 16]);
    assert_eq!(write.occurred_at_ms, 1234);
    assert_eq!(write.payments.len(), 2);
    for (payment, id, event_id, account_id, amount_cents, is_home) in [
        (&write.payments[0], [2; 16], [3; 16], [0x51; 16], 500, true),
        (
            &write.payments[1],
            [4; 16],
            [5; 16],
            [0x52; 16],
            1000,
            false,
        ),
    ] {
        assert_eq!(payment.movement.id, id);
        assert_eq!(payment.event_id, event_id);
        assert_eq!(payment.movement.amount_cents, amount_cents);
        match &payment.movement.kind {
            MovementKind::Entry { account_id: actual } => assert_eq!(*actual, account_id),
            _ => panic!("FINALIZATION_PAYMENT_NOT_ENTRY"),
        }
        assert_eq!(matches!(&payment.movement.context, Context::Home), is_home);
    }
    let pending = write.pending.unwrap();
    assert_eq!(pending.receivable_id, [6; 16]);
    assert_eq!(pending.event_id, [7; 16]);
    assert_eq!(pending.customer_id, [0x61; 16]);
    assert_eq!(pending.due_at_ms, 1_800_000_000_000);
    assert_eq!(result.sale_id, [0x41; 16]);
    assert_eq!(
        [result.total_cents, result.paid_cents, result.pending_cents],
        [3000, 1500, 1500]
    );
}

#[test]
fn sal020_non_positive_payments_and_sum_overflow_stop_before_ids_clock_and_store() {
    for amount in [0, -1, i64::MIN] {
        let (mut ids, clock, mut store) = fixtures();
        let mut invalid = input();
        invalid.payments = vec![payment([0x51; 16], amount, Context::Home)];
        assert_eq!(
            finalize_sale(invalid, &mut ids, &clock, &mut store).err(),
            Some(SaleError::InvalidPayment)
        );
        assert_eq!(ids.calls, 0);
        assert_eq!(clock.calls.get(), 0);
        assert!(store.write.is_none());
    }

    let (mut ids, clock, mut store) = fixtures();
    let mut overflow = input();
    overflow.payments = vec![
        payment([0x51; 16], i64::MAX, Context::Home),
        payment([0x52; 16], 1, Context::Factory),
    ];
    assert_eq!(
        finalize_sale(overflow, &mut ids, &clock, &mut store).err(),
        Some(SaleError::InvalidPayment)
    );
    assert_eq!(ids.calls, 0);
    assert_eq!(clock.calls.get(), 0);
    assert!(store.write.is_none());
}

#[test]
fn sal020_id_failures_stop_before_clock_and_store_and_zero_ids_are_rejected() {
    for fail_at in 1..=7 {
        let (mut ids, clock, mut store) = fixtures();
        ids.fail_at = Some(fail_at);
        assert_eq!(
            finalize_sale(input(), &mut ids, &clock, &mut store).err(),
            Some(SaleError::IdGenerationFailed)
        );
        assert_eq!(ids.calls, fail_at);
        assert_eq!(clock.calls.get(), 0);
        assert!(store.write.is_none());
    }

    let (mut ids, clock, mut store) = fixtures();
    ids.zero_at = Some(4);
    assert_eq!(
        finalize_sale(input(), &mut ids, &clock, &mut store).err(),
        Some(SaleError::IdGenerationFailed)
    );
    assert_eq!(ids.calls, 4);
    assert_eq!(clock.calls.get(), 0);
    assert!(store.write.is_none());
}

#[test]
fn sal020_invalid_clock_and_store_error_are_sanitized() {
    let (mut ids, mut clock, mut store) = fixtures();
    clock.value = UNIX_EPOCH - Duration::from_millis(1);
    assert_eq!(
        finalize_sale(input(), &mut ids, &clock, &mut store).err(),
        Some(SaleError::ClockFailed)
    );
    assert_eq!(clock.calls.get(), 1);
    assert!(store.write.is_none());

    let (mut ids, clock, mut store) = fixtures();
    store.result = Err(SaleError::StoreFailed);
    assert_eq!(
        finalize_sale(input(), &mut ids, &clock, &mut store).err(),
        Some(SaleError::StoreFailed)
    );
    assert_eq!(
        format!("{}", SaleError::StoreFailed),
        "sale operation failed"
    );
}
