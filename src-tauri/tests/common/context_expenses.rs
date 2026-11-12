use super::exit_persistence::{assert_health, setup, snapshot, FixedClock, FixedIds};
use millani_artes::application::{
    calculate_home_expenses::calculate_home_expenses, create_entry::create_entry,
    create_exit::create_exit,
};
use millani_artes::database::open_database;
use millani_artes::domain::Context;
use millani_artes::infrastructure::movements::AdapterSqliteMovementStore;

#[test]
fn persisted_home_outflows_reconcile_after_reopen_from_query_only_store() {
    let (_directory, path, account) = setup();
    let mut store = AdapterSqliteMovementStore::new(open_database(&path).unwrap());

    create_exit(
        account,
        40,
        Context::Home,
        &mut FixedIds::new([0x11; 16], [0x21; 16]),
        &FixedClock,
        &mut store,
    )
    .unwrap_or_else(|_| panic!("home_exit_1"));

    create_exit(
        account,
        60,
        Context::Home,
        &mut FixedIds::new([0x12; 16], [0x22; 16]),
        &FixedClock,
        &mut store,
    )
    .unwrap_or_else(|_| panic!("home_exit_2"));

    create_entry(
        account,
        1_000,
        Context::Home,
        &mut FixedIds::new([0x13; 16], [0x23; 16]),
        &FixedClock,
        &mut store,
    )
    .unwrap_or_else(|_| panic!("home_entry"));

    create_exit(
        account,
        700,
        Context::Factory,
        &mut FixedIds::new([0x14; 16], [0x24; 16]),
        &FixedClock,
        &mut store,
    )
    .unwrap_or_else(|_| panic!("factory_exit"));

    drop(store);

    let before_connection = open_database(&path).unwrap();
    assert_health(&before_connection);
    let before = snapshot(&before_connection);
    drop(before_connection);

    let reader = open_database(&path).unwrap();
    reader.pragma_update(None, "query_only", true).unwrap();
    let read_only_store = AdapterSqliteMovementStore::new(reader);
    assert_eq!(calculate_home_expenses(&read_only_store).unwrap(), 100);
    drop(read_only_store);

    let after_connection = open_database(&path).unwrap();
    assert_health(&after_connection);
    assert_eq!(snapshot(&after_connection), before);
}
