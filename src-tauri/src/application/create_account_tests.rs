use super::{create_account, duration_to_epoch_millis};
use crate::domain::ports::{Clock, UuidGenerator};
use crate::domain::{Account, AccountType};
use std::cell::Cell;
use std::error::Error;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const SENSITIVE_MARKER: &str = "PRIVATE_ACCOUNT_STORE_FAILURE_9f2a";

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
    values: [[u8; 16]; 2],
    fail_at: Option<usize>,
    calls: usize,
}

impl UuidGenerator for FixedIds {
    type Error = &'static str;

    fn generate(&mut self) -> Result<[u8; 16], Self::Error> {
        let call = self.calls;
        self.calls += 1;
        if self.fail_at == Some(call) {
            return Err("UUID_SOURCE_SECRET");
        }
        self.values
            .get(call)
            .copied()
            .ok_or("UUID_SOURCE_EXHAUSTED")
    }
}

type PersistedAccount = ([u8; 16], String, &'static str, i64, [u8; 16], i64);

#[derive(Default)]
struct FakeStore {
    calls: usize,
    fail: bool,
    persisted: Option<PersistedAccount>,
}

struct StoreFailure(&'static str);

impl std::fmt::Debug for StoreFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("StoreFailure").field(&self.0).finish()
    }
}

impl std::fmt::Display for StoreFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl Error for StoreFailure {}

impl super::AccountCreationStore for FakeStore {
    type Error = StoreFailure;

    fn persist_account_with_audit(
        &mut self,
        account: &Account,
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<(), Self::Error> {
        self.calls += 1;
        let account_type = match &account.account_type {
            AccountType::Cash => "cash",
            AccountType::BankPix => "bank_pix",
            AccountType::Card => "card",
            AccountType::Other => "other",
        };
        self.persisted = Some((
            account.id,
            account.name.clone(),
            account_type,
            account.initial_balance_cents,
            event_id,
            occurred_at_ms,
        ));
        if self.fail {
            Err(StoreFailure(SENSITIVE_MARKER))
        } else {
            Ok(())
        }
    }
}

fn ids(fail_at: Option<usize>) -> FixedIds {
    FixedIds {
        values: [[0xA1; 16], [0xB2; 16]],
        fail_at,
        calls: 0,
    }
}

#[test]
fn valid_account_preserves_fields_and_persists_once_with_audit_metadata() {
    let name = "  Ateliê Millani 🧵  ".to_owned();
    let now = UNIX_EPOCH + Duration::from_millis(1_765_000_000_123);
    let clock = FixedClock::new(now);
    let mut uuid_generator = ids(None);
    let mut store = FakeStore::default();

    let account = create_account(
        name.clone(),
        AccountType::BankPix,
        i64::MIN,
        &mut uuid_generator,
        &clock,
        &mut store,
    )
    .unwrap();

    assert_eq!(account.id, [0xA1; 16]);
    assert_eq!(account.name, name);
    assert!(matches!(&account.account_type, AccountType::BankPix));
    assert_eq!(account.initial_balance_cents, i64::MIN);
    assert_eq!(uuid_generator.calls, 2);
    assert_eq!(clock.calls.get(), 1);
    assert_eq!(store.calls, 1);
    assert_eq!(
        store.persisted,
        Some((
            [0xA1; 16],
            "  Ateliê Millani 🧵  ".into(),
            "bank_pix",
            i64::MIN,
            [0xB2; 16],
            1_765_000_000_123,
        ))
    );
}

#[test]
fn invalid_empty_and_unicode_whitespace_names_fail_before_ports() {
    for name in [String::new(), " \t\r\n".into(), "\u{2003}\u{00A0}".into()] {
        let clock = FixedClock::new(UNIX_EPOCH);
        let mut uuid_generator = ids(None);
        let mut store = FakeStore::default();

        let result = create_account(
            name,
            AccountType::Cash,
            0,
            &mut uuid_generator,
            &clock,
            &mut store,
        );

        assert!(result.is_err());
        assert_eq!(uuid_generator.calls, 0);
        assert_eq!(clock.calls.get(), 0);
        assert_eq!(store.calls, 0);
    }
}

#[test]
fn either_uuid_generation_failure_prevents_persistence() {
    for fail_at in [0, 1] {
        let clock = FixedClock::new(UNIX_EPOCH);
        let mut uuid_generator = ids(Some(fail_at));
        let mut store = FakeStore::default();

        let result = create_account(
            "Conta válida".into(),
            AccountType::Other,
            0,
            &mut uuid_generator,
            &clock,
            &mut store,
        );

        assert!(result.is_err());
        assert_eq!(uuid_generator.calls, fail_at + 1);
        assert_eq!(clock.calls.get(), 0);
        assert_eq!(store.calls, 0);
    }
}

#[test]
fn pre_epoch_clock_fails_before_persistence() {
    let clock = FixedClock::new(UNIX_EPOCH - Duration::from_millis(1));
    let mut uuid_generator = ids(None);
    let mut store = FakeStore::default();

    let result = create_account(
        "Conta válida".into(),
        AccountType::Card,
        0,
        &mut uuid_generator,
        &clock,
        &mut store,
    );

    assert!(result.is_err());
    assert_eq!(uuid_generator.calls, 2);
    assert_eq!(clock.calls.get(), 1);
    assert_eq!(store.calls, 0);
}

#[test]
fn out_of_i64_range_duration_fails_conversion() {
    assert!(duration_to_epoch_millis(Duration::from_millis(i64::MAX as u64 + 1)).is_err());
}

#[test]
fn store_failure_does_not_leak_payload_through_error_traits() {
    let clock = FixedClock::new(UNIX_EPOCH + Duration::from_millis(10));
    let mut uuid_generator = ids(None);
    let mut store = FakeStore {
        fail: true,
        ..FakeStore::default()
    };

    let result = create_account(
        "PRIVATE_ACCOUNT_NAME_13ac".into(),
        AccountType::Cash,
        0,
        &mut uuid_generator,
        &clock,
        &mut store,
    );
    let error = result.err().expect("STORE_FAILURE_WAS_HIDDEN_AS_SUCCESS");
    let display = error.to_string();
    let debug = format!("{error:?}");

    assert!(!display.contains(SENSITIVE_MARKER));
    assert!(!debug.contains(SENSITIVE_MARKER));
    assert!(!display.contains("PRIVATE_ACCOUNT_NAME_13ac"));
    assert!(!debug.contains("PRIVATE_ACCOUNT_NAME_13ac"));
    assert!(error.source().is_none());
    assert_eq!(store.calls, 1);
}

#[test]
fn application_and_adapter_errors_are_static_and_have_no_source() {
    let errors = [
        (
            crate::application::create_account::CreateAccountError::InvalidName,
            "InvalidName",
            "account name is invalid",
        ),
        (
            crate::application::create_account::CreateAccountError::IdGenerationFailed,
            "IdGenerationFailed",
            "account identifiers could not be generated",
        ),
        (
            crate::application::create_account::CreateAccountError::ClockFailed,
            "ClockFailed",
            "account timestamp is invalid",
        ),
        (
            crate::application::create_account::CreateAccountError::StoreFailed,
            "StoreFailed",
            "account could not be stored",
        ),
    ];
    for (error, debug, display) in errors {
        assert_eq!(format!("{error:?}"), debug);
        assert_eq!(error.to_string(), display);
        assert!(error.source().is_none());
    }

    let error = crate::infrastructure::accounts::AccountStoreError;
    assert_eq!(format!("{error:?}"), "AccountStoreError");
    assert_eq!(error.to_string(), "account could not be stored");
    assert!(error.source().is_none());
}
