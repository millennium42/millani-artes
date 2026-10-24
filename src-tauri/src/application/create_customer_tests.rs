use super::{
    create_customer, duration_to_epoch_millis, CreateCustomerError, CustomerCreationStore,
};
use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::Customer;
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

type PersistedCustomer = ([u8; 16], String, Option<String>, [u8; 16], i64);

#[derive(Default)]
struct FakeStore {
    calls: usize,
    fail: bool,
    persisted: Option<PersistedCustomer>,
}

impl CustomerCreationStore for FakeStore {
    type Error = &'static str;
    fn persist_customer_with_audit(
        &mut self,
        customer: &Customer,
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<(), Self::Error> {
        self.calls += 1;
        self.persisted = Some((
            customer.id,
            customer.name.clone(),
            customer.phone.clone(),
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
fn creates_customer_preserving_optional_phone_and_all_audit_metadata() {
    for phone in [None, Some("(11) 91234-5678"), Some("(99) 99999-9999")] {
        for at in [0, 1234] {
            let name = "  Ateliê '🧵'\0  ".to_string();
            let clock = FixedClock::new(UNIX_EPOCH + Duration::from_millis(at));
            let mut ids = ids(None);
            let mut store = FakeStore::default();
            let customer = create_customer(
                name.clone(),
                phone.map(String::from),
                &mut ids,
                &clock,
                &mut store,
            )
            .unwrap();
            assert_eq!(customer.id, [0xA1; 16]);
            assert_eq!(customer.name, name);
            assert_eq!(customer.phone.as_deref(), phone);
            assert_eq!(
                store.persisted,
                Some((
                    [0xA1; 16],
                    name,
                    phone.map(String::from),
                    [0xB2; 16],
                    at as i64
                ))
            );
            assert_eq!((ids.calls, clock.calls.get(), store.calls), (2, 1, 1));
        }
    }
}
#[test]
fn invalid_names_and_priority_fail_before_any_port() {
    for name in ["", " \t\r\n", "\u{2003}\u{00a0}"] {
        for phone in [None, Some("PRIVATE_BAD_PHONE")] {
            let clock = FixedClock::new(UNIX_EPOCH);
            let mut ids = ids(None);
            let mut store = FakeStore::default();
            let e = create_customer(
                name.into(),
                phone.map(String::from),
                &mut ids,
                &clock,
                &mut store,
            )
            .err()
            .unwrap();
            assert_eq!(e, CreateCustomerError::InvalidName);
            assert_eq!((ids.calls, clock.calls.get(), store.calls), (0, 0, 0));
        }
    }
}

#[test]
fn either_id_failure_prevents_clock_and_store() {
    for fail_at in [0, 1] {
        let clock = FixedClock::new(UNIX_EPOCH);
        let mut ids = ids(Some(fail_at));
        let mut store = FakeStore::default();
        let error = create("Categoria".into(), &mut ids, &clock, &mut store)
            .err()
            .unwrap();
        assert_eq!(error, CreateCustomerError::IdGenerationFailed);
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
    let error = create("Categoria".into(), &mut ids, &clock, &mut store)
        .err()
        .unwrap();
    assert_eq!(error, CreateCustomerError::ClockFailed);
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
        Err(CreateCustomerError::ClockFailed)
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
    let error = create("PRIVATE_CATEGORY_NAME".into(), &mut ids, &clock, &mut store)
        .err()
        .unwrap();
    assert_eq!(error, CreateCustomerError::StoreFailed);
    assert_eq!((ids.calls, clock.calls.get(), store.calls), (2, 1, 1));
    for (error, debug, display) in [
        (
            CreateCustomerError::InvalidName,
            "InvalidName",
            "customer name is invalid",
        ),
        (
            CreateCustomerError::InvalidPhone,
            "InvalidPhone",
            "customer phone is invalid",
        ),
        (
            CreateCustomerError::IdGenerationFailed,
            "IdGenerationFailed",
            "customer identifiers could not be generated",
        ),
        (
            CreateCustomerError::ClockFailed,
            "ClockFailed",
            "customer timestamp is invalid",
        ),
        (
            CreateCustomerError::StoreFailed,
            "StoreFailed",
            "customer could not be stored",
        ),
    ] {
        assert_eq!(format!("{error:?}"), debug);
        assert_eq!(error.to_string(), display);
        assert!(error.source().is_none());
    }
}

fn create(
    name: String,
    ids: &mut FixedIds,
    clock: &FixedClock,
    store: &mut FakeStore,
) -> Result<Customer, CreateCustomerError> {
    create_customer(name, None, ids, clock, store)
}

#[test]
fn invalid_phones_fail_before_any_port() {
    for phone in [
        "",
        " ",
        "11912345678",
        "(20) 91234-5678",
        "(11) 81234-5678",
        "(11) 9é34-5678",
        "(11) 91234-5678\0",
    ] {
        let clock = FixedClock::new(UNIX_EPOCH);
        let mut ids = ids(None);
        let mut store = FakeStore::default();
        let e = create_customer(
            "  Válido ‘Á’\0  ".into(),
            Some(phone.into()),
            &mut ids,
            &clock,
            &mut store,
        )
        .err()
        .unwrap();
        assert_eq!(e, CreateCustomerError::InvalidPhone);
        assert_eq!((ids.calls, clock.calls.get(), store.calls), (0, 0, 0));
    }
}
