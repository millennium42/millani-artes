use super::exit_persistence::{assert_health, setup, snapshot, FixedClock, FixedIds};
use millani_artes::application::{
    calculate_context_result::{calculate_context_result, CalculateContextResultError},
    create_entry::create_entry,
    create_exit::create_exit,
};
use millani_artes::database::open_database;
use millani_artes::domain::Context;
use millani_artes::infrastructure::movements::AdapterSqliteMovementStore;
use rusqlite::Connection;

fn assert_error(result: Result<i64, CalculateContextResultError>) {
    let e = result.unwrap_err();
    assert_eq!(format!("{e:?}"), "CalculateContextResultError");
    assert_eq!(format!("{e}"), "context result could not be calculated");
    assert!(std::error::Error::source(&e).is_none());
}
#[test]
fn read_failure_and_poisoned_adapter_do_not_yield_numeric_results() {
    let missing = AdapterSqliteMovementStore::new(Connection::open_in_memory().unwrap());
    assert_error(calculate_context_result(&Context::Home, &missing));
    let (_directory, path, _source) = setup();
    let before = snapshot(&open_database(&path).unwrap());
    let mut store = AdapterSqliteMovementStore::new(open_database(&path).unwrap());
    assert!(create_exit(
        [0xFE; 16],
        1,
        Context::Home,
        &mut FixedIds::new([0x31; 16], [0x41; 16]),
        &FixedClock,
        &mut store
    )
    .is_err());
    assert_error(calculate_context_result(&Context::Home, &store));
    assert_error(calculate_context_result(&Context::Factory, &store));
    let connection = open_database(&path).unwrap();
    assert_health(&connection);
    assert_eq!(snapshot(&connection), before);
    connection.pragma_update(None, "query_only", true).unwrap();
    let fresh = AdapterSqliteMovementStore::new(connection);
    assert_eq!(calculate_context_result(&Context::Home, &fresh).unwrap(), 0);
    assert_eq!(
        calculate_context_result(&Context::Factory, &fresh).unwrap(),
        0
    );
    assert_eq!(snapshot(&open_database(&path).unwrap()), before);
}
#[test]
fn real_aggregate_checks_overflow_and_cancellation_without_changing_history() {
    let (_directory, path, source) = setup();
    let mut store = AdapterSqliteMovementStore::new(open_database(&path).unwrap());
    create_entry(
        source,
        i64::MAX,
        Context::Home,
        &mut FixedIds::new([0x11; 16], [0x21; 16]),
        &FixedClock,
        &mut store,
    )
    .unwrap_or_else(|_| panic!("entry1"));
    create_entry(
        source,
        i64::MAX,
        Context::Home,
        &mut FixedIds::new([0x12; 16], [0x22; 16]),
        &FixedClock,
        &mut store,
    )
    .unwrap_or_else(|_| panic!("entry2"));
    create_exit(
        source,
        i64::MAX,
        Context::Home,
        &mut FixedIds::new([0x13; 16], [0x23; 16]),
        &FixedClock,
        &mut store,
    )
    .unwrap_or_else(|_| panic!("exit"));
    let before = snapshot(&open_database(&path).unwrap());
    assert_eq!(
        calculate_context_result(&Context::Home, &store).unwrap(),
        i64::MAX
    );
    assert_eq!(
        calculate_context_result(&Context::Factory, &store).unwrap(),
        0
    );
    assert_eq!(snapshot(&open_database(&path).unwrap()), before);
    create_entry(
        source,
        1,
        Context::Home,
        &mut FixedIds::new([0x14; 16], [0x24; 16]),
        &FixedClock,
        &mut store,
    )
    .unwrap_or_else(|_| panic!("entry3"));
    let after = snapshot(&open_database(&path).unwrap());
    assert_error(calculate_context_result(&Context::Home, &store));
    assert_eq!(
        calculate_context_result(&Context::Factory, &store).unwrap(),
        0
    );
    let reopened = open_database(&path).unwrap();
    assert_health(&reopened);
    assert_eq!(snapshot(&reopened), after);
}
