use super::exit_persistence::{assert_health, setup, snapshot, FixedIds};
use millani_artes::application::create_entry::create_entry;
use millani_artes::application::create_exit::create_exit;
use millani_artes::application::edit_movement::edit_movement;
use millani_artes::application::list_today_exits::{
    list_today_exit_cents, ListTodayExitsError, TodayExitReadStore,
};
use millani_artes::database::open_database;
use millani_artes::domain::ports::Clock;
use millani_artes::domain::{brasilia_day_window_ms, Context, Movement, MovementKind};
use millani_artes::infrastructure::movements::{AdapterSqliteMovementStore, TodayExitReadError};
use rusqlite::Connection;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const NOW_MS: i64 = 1_800_000_000_000;

struct MillisClock(i64);

impl Clock for MillisClock {
    fn now(&self) -> SystemTime {
        UNIX_EPOCH + Duration::from_millis(self.0 as u64)
    }
}

fn add_entry(path: &std::path::Path, account_id: [u8; 16], id: u8, amount: i64, at_ms: i64) {
    create_entry(
        account_id,
        amount,
        Context::Home,
        &mut FixedIds::new([id; 16], [id + 0x20; 16]),
        &MillisClock(at_ms),
        &mut AdapterSqliteMovementStore::new(open_database(path).unwrap()),
    )
    .unwrap();
}

fn add_exit(path: &std::path::Path, account_id: [u8; 16], id: u8, amount: i64, at_ms: i64) {
    create_exit(
        account_id,
        amount,
        Context::Home,
        &mut FixedIds::new([id; 16], [id + 0x20; 16]),
        &MillisClock(at_ms),
        &mut AdapterSqliteMovementStore::new(open_database(path).unwrap()),
    )
    .unwrap();
}

fn add_exit_without_audit(path: &std::path::Path, account_id: [u8; 16], id: u8, amount: i64) {
    open_database(path)
        .unwrap()
        .execute(
            "INSERT INTO main.movements (id, kind, amount_cents, context, account_id, source_account_id, destination_account_id) VALUES (?1, 'exit', ?2, 'home', ?3, NULL, NULL)",
            rusqlite::params![[id; 16].as_slice(), amount, account_id.as_slice()],
        )
        .unwrap();
}

fn edit_as_exit(path: &std::path::Path, account_id: [u8; 16], id: u8, amount: i64) {
    edit_movement(
        Movement {
            id: [id; 16],
            kind: MovementKind::Exit { account_id },
            amount_cents: amount,
            context: Context::Factory,
        },
        &mut FixedIds::new([id + 0x40; 16], [id + 0x60; 16]),
        &MillisClock(NOW_MS),
        &mut AdapterSqliteMovementStore::new(open_database(path).unwrap()),
    )
    .unwrap();
}

fn assert_static_store_error(result: Result<Vec<i64>, TodayExitReadError>) {
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("expected a static read error"),
    };
    assert_eq!(format!("{error:?}"), "TodayExitReadError");
    assert_eq!(format!("{error}"), "today's exit history could not be read");
    assert!(std::error::Error::source(&error).is_none());
}

fn assert_app_error(result: Result<i64, ListTodayExitsError>) {
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("expected a static dashboard error"),
    };
    assert_eq!(format!("{error:?}"), "ListTodayExitsError");
    assert_eq!(format!("{error}"), "today's exits could not be read");
    assert!(std::error::Error::source(&error).is_none());
}

#[test]
fn dashboard_today_uses_original_exit_creation_and_is_read_only_after_reopen() {
    let (_directory, path, account_id) = setup();
    let (start_ms, end_ms) = brasilia_day_window_ms(NOW_MS).unwrap();

    add_exit(&path, account_id, 0x11, 100, start_ms);
    add_exit(&path, account_id, 0x12, 500, start_ms - 1);
    add_exit(&path, account_id, 0x13, 70, end_ms);
    add_exit(&path, account_id, 0x14, 250, end_ms - 1);
    add_entry(&path, account_id, 0x15, 80, start_ms + 1);
    add_entry(&path, account_id, 0x16, 90, start_ms + 2);
    edit_as_exit(&path, account_id, 0x16, 95);
    add_exit_without_audit(&path, account_id, 0x17, 88);
    edit_as_exit(&path, account_id, 0x11, 150);
    edit_as_exit(&path, account_id, 0x12, 600);

    let reader = open_database(&path).unwrap();
    assert_health(&reader);
    reader.pragma_update(None, "query_only", true).unwrap();
    let before = snapshot(&reader);
    let result = list_today_exit_cents(
        &MillisClock(NOW_MS),
        &AdapterSqliteMovementStore::new(reader),
    )
    .unwrap();

    assert_eq!(result, 400);
    assert_eq!(snapshot(&open_database(&path).unwrap()), before);
}

#[test]
fn missing_audit_schema_fails_with_a_sanitized_error() {
    let connection = Connection::open_in_memory().unwrap();
    let store = AdapterSqliteMovementStore::new(connection);

    assert_static_store_error(store.read_exit_amounts_created_between(0, 1));
    assert_app_error(list_today_exit_cents(&MillisClock(NOW_MS), &store));
}

#[test]
fn duplicate_creation_audits_fail_closed_without_guessing_the_original() {
    let (_directory, path, account_id) = setup();
    let (start_ms, end_ms) = brasilia_day_window_ms(NOW_MS).unwrap();
    add_exit(&path, account_id, 0x12, 500, start_ms - 1);

    open_database(&path)
        .unwrap()
        .execute(
            "INSERT INTO main.audit_event (event_id, occurred_at_ms, operation, entity, entity_id) VALUES (?1, ?2, 'movement_exit_create', 'movement', ?3)",
            rusqlite::params![[0x77_u8; 16].as_slice(), start_ms + 3, [0x12_u8; 16].as_slice()],
        )
        .unwrap();

    let store = AdapterSqliteMovementStore::new(open_database(&path).unwrap());
    assert_static_store_error(store.read_exit_amounts_created_between(start_ms, end_ms));
    assert_app_error(list_today_exit_cents(&MillisClock(NOW_MS), &store));
}
