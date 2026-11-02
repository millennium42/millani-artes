use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::SaleReturnItem;
use std::fmt;
use std::time::UNIX_EPOCH;

pub struct RecordSaleReturnWrite {
    pub return_id: [u8; 16],
    pub sale_item_id: [u8; 16],
    pub quantity: i64,
    pub audit_event_id: [u8; 16],
    pub occurred_at_ms: i64,
}

pub trait SaleReturnRecordingStore {
    fn persist_return_with_audit(
        &mut self,
        write: RecordSaleReturnWrite,
    ) -> Result<(), RecordSaleReturnError>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum RecordSaleReturnError {
    InvalidInput,
    IdGenerationFailed,
    ClockFailed,
    SaleItemNotFound,
    QuantityExceedsAvailable,
    StoreFailed,
}

impl fmt::Display for RecordSaleReturnError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidInput => "return quantity is invalid",
            Self::IdGenerationFailed => "return identifiers could not be generated",
            Self::ClockFailed => "return timestamp is invalid",
            Self::SaleItemNotFound => "sale item could not be found",
            Self::QuantityExceedsAvailable => "return quantity exceeds the available quantity",
            Self::StoreFailed => "return could not be stored",
        })
    }
}

impl std::error::Error for RecordSaleReturnError {}

pub fn record_sale_return<S: SaleReturnRecordingStore, I: UuidGenerator, C: Clock>(
    sale_item_id: [u8; 16],
    quantity: i64,
    ids: &mut I,
    clock: &C,
    store: &mut S,
) -> Result<SaleReturnItem, RecordSaleReturnError> {
    if quantity <= 0 {
        return Err(RecordSaleReturnError::InvalidInput);
    }

    let return_id = generate_id(ids)?;
    let audit_event_id = generate_id(ids)?;
    let occurred_at = clock
        .now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| RecordSaleReturnError::ClockFailed)?;
    let occurred_at_ms =
        i64::try_from(occurred_at.as_millis()).map_err(|_| RecordSaleReturnError::ClockFailed)?;

    store.persist_return_with_audit(RecordSaleReturnWrite {
        return_id,
        sale_item_id,
        quantity,
        audit_event_id,
        occurred_at_ms,
    })?;

    Ok(SaleReturnItem {
        sale_item_id,
        quantity,
    })
}

fn generate_id<I: UuidGenerator>(ids: &mut I) -> Result<[u8; 16], RecordSaleReturnError> {
    ids.generate()
        .map_err(|_| RecordSaleReturnError::IdGenerationFailed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::time::{Duration, SystemTime};

    struct SequenceIds {
        next: u8,
        fail_at: Option<u8>,
        calls: u8,
    }

    impl UuidGenerator for SequenceIds {
        type Error = ();

        fn generate(&mut self) -> Result<[u8; 16], Self::Error> {
            if self.fail_at == Some(self.calls) {
                return Err(());
            }
            let value = self.next;
            self.next += 1;
            self.calls += 1;
            Ok([value; 16])
        }
    }

    struct FixedClock {
        value: SystemTime,
        calls: Cell<u8>,
    }

    impl Clock for FixedClock {
        fn now(&self) -> SystemTime {
            self.calls.set(self.calls.get() + 1);
            self.value
        }
    }

    #[derive(Default)]
    struct FakeStore {
        write: Option<RecordSaleReturnWrite>,
    }

    impl SaleReturnRecordingStore for FakeStore {
        fn persist_return_with_audit(
            &mut self,
            write: RecordSaleReturnWrite,
        ) -> Result<(), RecordSaleReturnError> {
            self.write = Some(write);
            Ok(())
        }
    }

    fn sequence_ids() -> SequenceIds {
        SequenceIds {
            next: 0x41,
            fail_at: None,
            calls: 0,
        }
    }

    fn fixed_clock(value: SystemTime) -> FixedClock {
        FixedClock {
            value,
            calls: Cell::new(0),
        }
    }

    #[test]
    fn ret005_invalid_quantity_stops_before_ids_clock_or_store() {
        let mut ids = sequence_ids();
        let clock = fixed_clock(UNIX_EPOCH);
        let mut store = FakeStore::default();

        assert!(matches!(
            record_sale_return([0x31; 16], 0, &mut ids, &clock, &mut store),
            Err(RecordSaleReturnError::InvalidInput)
        ));
        assert_eq!(ids.calls, 0);
        assert_eq!(clock.calls.get(), 0);
        assert!(store.write.is_none());
    }

    #[test]
    fn ret005_generated_record_uses_two_ids_and_clock_and_returns_only_after_store_success() {
        let mut ids = sequence_ids();
        let clock = fixed_clock(UNIX_EPOCH + Duration::from_millis(1234));
        let mut store = FakeStore::default();

        let result = record_sale_return([0x31; 16], 2, &mut ids, &clock, &mut store).unwrap();

        assert_eq!(result.sale_item_id, [0x31; 16]);
        assert_eq!(result.quantity, 2);
        assert_eq!(ids.calls, 2);
        assert_eq!(clock.calls.get(), 1);
        let write = store.write.unwrap();
        assert_eq!(write.return_id, [0x41; 16]);
        assert_eq!(write.audit_event_id, [0x42; 16]);
        assert_eq!(write.sale_item_id, [0x31; 16]);
        assert_eq!(write.quantity, 2);
        assert_eq!(write.occurred_at_ms, 1234);
    }

    #[test]
    fn ret005_id_or_clock_failure_does_not_call_store() {
        let mut ids = SequenceIds {
            fail_at: Some(1),
            ..sequence_ids()
        };
        let clock = fixed_clock(UNIX_EPOCH);
        let mut store = FakeStore::default();
        assert!(matches!(
            record_sale_return([0x31; 16], 1, &mut ids, &clock, &mut store),
            Err(RecordSaleReturnError::IdGenerationFailed)
        ));
        assert_eq!(clock.calls.get(), 0);
        assert!(store.write.is_none());

        let mut ids = sequence_ids();
        let clock = fixed_clock(UNIX_EPOCH - Duration::from_millis(1));
        assert!(matches!(
            record_sale_return([0x31; 16], 1, &mut ids, &clock, &mut store),
            Err(RecordSaleReturnError::ClockFailed)
        ));
        assert_eq!(ids.calls, 2);
        assert_eq!(clock.calls.get(), 1);
        assert!(store.write.is_none());
    }

    #[test]
    fn ret005_error_messages_are_static_and_sanitized() {
        let errors = [
            (
                RecordSaleReturnError::InvalidInput,
                "return quantity is invalid",
            ),
            (
                RecordSaleReturnError::IdGenerationFailed,
                "return identifiers could not be generated",
            ),
            (
                RecordSaleReturnError::ClockFailed,
                "return timestamp is invalid",
            ),
            (
                RecordSaleReturnError::SaleItemNotFound,
                "sale item could not be found",
            ),
            (
                RecordSaleReturnError::QuantityExceedsAvailable,
                "return quantity exceeds the available quantity",
            ),
            (
                RecordSaleReturnError::StoreFailed,
                "return could not be stored",
            ),
        ];

        for (error, expected) in errors {
            assert_eq!(error.to_string(), expected);
        }
    }
}
