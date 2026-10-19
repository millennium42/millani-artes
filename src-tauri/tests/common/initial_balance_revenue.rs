use super::exit_persistence::{assert_health, setup, snapshot, FixedClock, FixedIds};
use millani_artes::application::{
    calculate_context_result::calculate_context_result, create_account::create_account,
    create_entry::create_entry, list_accounts::list_accounts, list_movements::list_movements,
};
use millani_artes::database::open_database;
use millani_artes::domain::{AccountType, Context, MovementKind};
use millani_artes::infrastructure::{
    accounts::AdapterSqliteAccountStore, movements::AdapterSqliteMovementStore,
};

#[test]
fn opening_balances_are_not_entries_or_reported_income() {
    let (_directory, path, _seed) = setup();

    for (index, opening) in [i64::MIN, -1, 0, 1, i64::MAX].into_iter().enumerate() {
        let before = snapshot(&open_database(&path).unwrap());

        let id = [0x61 + index as u8; 16];
        let audit = [0x81 + index as u8; 16];

        let mut account_store = AdapterSqliteAccountStore::new(open_database(&path).unwrap());
        create_account(
            format!("Conta sintética {}", index),
            AccountType::Other,
            opening,
            &mut FixedIds::new(id, audit),
            &FixedClock,
            &mut account_store,
        )
        .unwrap();
        drop(account_store);

        let fresh_account_store = AdapterSqliteAccountStore::new(open_database(&path).unwrap());
        let accounts = list_accounts(&fresh_account_store).unwrap();
        let found = accounts
            .iter()
            .any(|a| a.id == id && a.initial_balance_cents == opening);
        assert!(found, "account with opening balance not found");
        drop(fresh_account_store);

        let connection = open_database(&path).unwrap();
        assert_health(&connection);
        let after = snapshot(&connection);

        connection.pragma_update(None, "query_only", true).unwrap();
        let movement_store = AdapterSqliteMovementStore::new(connection);
        let movements = list_movements(&movement_store).unwrap();
        assert!(movements.is_empty(), "opening produced movement");

        let result_home = calculate_context_result(&Context::Home, &movement_store);
        let result_factory = calculate_context_result(&Context::Factory, &movement_store);
        assert_eq!(result_home.unwrap(), 0);
        assert_eq!(result_factory.unwrap(), 0);

        assert_eq!(after.0.len(), before.0.len() + 1);
        assert_eq!(after.2.len(), before.2.len() + 1);
        assert_eq!(after.1, before.1);
        assert_eq!(after.3, before.3);

        assert!(after
            .2
            .iter()
            .all(|row| row[2] == rusqlite::types::Value::Text("account_create".into())));
        assert_eq!(snapshot(&open_database(&path).unwrap()), after);

        drop(movement_store);
    }

    let connection = open_database(&path).unwrap();
    assert_health(&connection);
    let before = snapshot(&connection);

    let mut movement_store = AdapterSqliteMovementStore::new(connection);

    let id1 = [0x63; 16];
    let audit1 = [0x71; 16];
    let id2 = [0x64; 16];
    let audit2 = [0x72; 16];

    let mut ids = FixedIds::new(id1, audit1);
    create_entry(
        [0x63; 16],
        1,
        Context::Home,
        &mut ids,
        &FixedClock,
        &mut movement_store,
    )
    .unwrap();

    let mut ids = FixedIds::new(id2, audit2);
    create_entry(
        [0x63; 16],
        2,
        Context::Factory,
        &mut ids,
        &FixedClock,
        &mut movement_store,
    )
    .unwrap();

    drop(movement_store);

    let connection = open_database(&path).unwrap();
    assert_health(&connection);
    let after = snapshot(&connection);

    connection.pragma_update(None, "query_only", true).unwrap();
    let fresh_store = AdapterSqliteMovementStore::new(connection);
    let movements = list_movements(&fresh_store).unwrap();
    assert_eq!(movements.len(), 2);
    assert_eq!(movements[0].id, id1);
    assert_eq!(movements[1].id, id2);
    assert_eq!(movements[0].amount_cents, 1);
    assert_eq!(movements[1].amount_cents, 2);
    assert!(
        matches!(&movements[0].kind, MovementKind::Entry { account_id: id } if *id == [0x63;16])
    );
    assert!(
        matches!(&movements[1].kind, MovementKind::Entry { account_id: id } if *id == [0x63;16])
    );

    let result_home = calculate_context_result(&Context::Home, &fresh_store);
    let result_factory = calculate_context_result(&Context::Factory, &fresh_store);
    assert_eq!(result_home.unwrap(), 1);
    assert_eq!(result_factory.unwrap(), 2);

    assert_eq!(after.0, before.0);
    assert_eq!(after.3, before.3);
    assert_eq!(after.2.len(), before.2.len() + 2);

    assert_eq!(snapshot(&open_database(&path).unwrap()), after);
    drop(fresh_store);
}
