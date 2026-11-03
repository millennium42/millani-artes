use super::*;
use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::{Context, MovementKind};
use std::cell::Cell;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

struct Ids {
    next: u8,
    calls: Cell<u8>,
    fail: bool,
}
impl UuidGenerator for Ids {
    type Error = ();
    fn generate(&mut self) -> Result<[u8; 16], ()> {
        let n = self.calls.get();
        self.calls.set(n + 1);
        if self.fail {
            return Err(());
        }
        let id = self.next;
        self.next += 1;
        Ok([id; 16])
    }
}
struct ClockStub(Option<SystemTime>, Cell<u8>);
impl Clock for ClockStub {
    fn now(&self) -> SystemTime {
        self.1.set(self.1.get() + 1);
        self.0.unwrap_or(UNIX_EPOCH - Duration::from_millis(1))
    }
}
type ObservedRefund = ([u8; 16], [u8; 16], i64, [u8; 16], bool, [u8; 16], i64);
#[derive(Default)]
struct Store {
    seen: Option<ObservedRefund>,
    fail: bool,
}
impl SaleRefundRecordingStore for Store {
    fn persist_refund_with_audit(
        &mut self,
        w: SaleRefundWrite<'_>,
    ) -> Result<(), RecordSaleRefundError> {
        if self.fail {
            return Err(RecordSaleRefundError::StoreFailed);
        }
        let account = match &w.movement.kind {
            MovementKind::Exit { account_id } => *account_id,
            _ => [0; 16],
        };
        self.seen = Some((
            w.return_item_id,
            w.movement.id,
            w.movement.amount_cents,
            account,
            matches!(&w.movement.context, Context::Factory),
            w.audit_event_id,
            w.occurred_at_ms,
        ));
        Ok(())
    }
}
#[test]
fn record_refund_forwards_account_amount_context_and_generated_audit() {
    let mut ids = Ids {
        next: 0x51,
        calls: Cell::new(0),
        fail: false,
    };
    let clock = ClockStub(Some(UNIX_EPOCH + Duration::from_millis(1234)), Cell::new(0));
    let mut store = Store::default();
    let m = record_sale_refund(
        [0x41; 16],
        [0x42; 16],
        700,
        Context::Factory,
        &mut ids,
        &clock,
        &mut store,
    )
    .unwrap();
    assert_eq!(m.id, [0x51; 16]);
    assert_eq!(m.amount_cents, 700);
    assert!(matches!(&m.kind,MovementKind::Exit{account_id} if *account_id==[0x42;16]));
    assert!(matches!(&m.context, Context::Factory));
    assert_eq!(
        store.seen,
        Some(([0x41; 16], [0x51; 16], 700, [0x42; 16], true, [0x52; 16], 1234))
    );
}
#[test]
fn rejects_bad_amount_before_side_effects_and_sanitizes_dependency_failures() {
    let mut ids = Ids {
        next: 1,
        calls: Cell::new(0),
        fail: false,
    };
    let clock = ClockStub(Some(UNIX_EPOCH), Cell::new(0));
    let mut store = Store::default();
    assert!(matches!(
        record_sale_refund(
            [1; 16],
            [2; 16],
            0,
            Context::Home,
            &mut ids,
            &clock,
            &mut store
        ),
        Err(RecordSaleRefundError::InvalidInput)
    ));
    assert_eq!(ids.calls.get(), 0);
    assert_eq!(clock.1.get(), 0);
    assert!(store.seen.is_none());
    ids.fail = true;
    assert!(matches!(
        record_sale_refund(
            [1; 16],
            [2; 16],
            1,
            Context::Home,
            &mut ids,
            &clock,
            &mut store
        ),
        Err(RecordSaleRefundError::IdGenerationFailed)
    ));
    ids.fail = false;
    let bad_clock = ClockStub(None, Cell::new(0));
    assert!(matches!(
        record_sale_refund(
            [1; 16],
            [2; 16],
            1,
            Context::Home,
            &mut ids,
            &bad_clock,
            &mut store
        ),
        Err(RecordSaleRefundError::ClockFailed)
    ));
    let mut failed = Store {
        seen: None,
        fail: true,
    };
    assert!(matches!(
        record_sale_refund(
            [1; 16],
            [2; 16],
            1,
            Context::Home,
            &mut ids,
            &clock,
            &mut failed
        ),
        Err(RecordSaleRefundError::StoreFailed)
    ));
    assert!(failed.seen.is_none());
}
