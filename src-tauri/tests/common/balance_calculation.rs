use super::exit_persistence::{assert_health, setup, FixedClock, FixedIds};
use millani_artes::application::{
    create_entry::create_entry, create_exit::create_exit, create_transfer::create_transfer,
    list_accounts::list_accounts, list_movements::list_movements,
};
use millani_artes::database::open_database;
use millani_artes::domain::{Context, Movement, MovementKind};
use millani_artes::infrastructure::{
    accounts::AdapterSqliteAccountStore, movements::AdapterSqliteMovementStore,
};

#[test]
fn persisted_movements_and_real_account_reader_feed_account_balance() {
    let (_directory, path, source) = setup();
    let destination = [0xA2u8; 16];
    let connection = open_database(&path).unwrap();
    connection.execute("INSERT INTO main.accounts(id,name,account_type,initial_balance_cents) VALUES(?1,?2,?3,?4)",
        rusqlite::params![destination.as_slice(),"Destino sintético","bank_pix",7654i64]).unwrap();
    let initial_audit: i64 = connection
        .query_row("SELECT COUNT(*) FROM main.audit_event", [], |r| r.get(0))
        .unwrap();
    drop(connection);
    let mut store = AdapterSqliteMovementStore::new(open_database(&path).unwrap());
    create_entry(
        source,
        100,
        Context::Home,
        &mut FixedIds::new([0x11; 16], [0x21; 16]),
        &FixedClock,
        &mut store,
    )
    .unwrap_or_else(|_| panic!("entry"));
    create_exit(
        source,
        30,
        Context::Factory,
        &mut FixedIds::new([0x12; 16], [0x22; 16]),
        &FixedClock,
        &mut store,
    )
    .unwrap_or_else(|_| panic!("exit"));
    create_transfer(
        source,
        destination,
        20,
        Context::Home,
        &mut FixedIds::new([0x13; 16], [0x23; 16]),
        &FixedClock,
        &mut store,
    )
    .unwrap_or_else(|_| panic!("transfer"));
    create_transfer(
        destination,
        source,
        5,
        Context::Factory,
        &mut FixedIds::new([0x14; 16], [0x24; 16]),
        &FixedClock,
        &mut store,
    )
    .unwrap_or_else(|_| panic!("reverse transfer"));
    drop(store);
    let accounts = list_accounts(&AdapterSqliteAccountStore::new(
        open_database(&path).unwrap(),
    ))
    .unwrap();
    let reopened = open_database(&path).unwrap();
    assert_health(&reopened);
    let reader_connection = open_database(&path).unwrap();
    reader_connection
        .pragma_update(None, "query_only", true)
        .unwrap();
    let movements: Vec<Movement> =
        list_movements(&AdapterSqliteMovementStore::new(reader_connection)).unwrap();
    assert_eq!(
        movements.iter().map(|m| m.id).collect::<Vec<_>>(),
        vec![[0x11; 16], [0x12; 16], [0x13; 16], [0x14; 16]]
    );
    assert_eq!(
        movements.iter().map(|m| m.amount_cents).collect::<Vec<_>>(),
        vec![100, 30, 20, 5]
    );
    assert!(matches!(movements[0].context, Context::Home));
    assert!(matches!(movements[1].context, Context::Factory));
    assert!(matches!(movements[2].context, Context::Home));
    assert!(matches!(movements[3].context, Context::Factory));
    assert!(matches!(movements[0].kind,MovementKind::Entry{account_id}if account_id==source));
    assert!(matches!(movements[1].kind,MovementKind::Exit{account_id}if account_id==source));
    assert!(
        matches!(movements[2].kind,MovementKind::Transfer{source_account_id,destination_account_id}if source_account_id==source && destination_account_id==destination)
    );
    assert!(
        matches!(movements[3].kind,MovementKind::Transfer{source_account_id,destination_account_id}if source_account_id==destination && destination_account_id==source)
    );
    assert_eq!(
        reopened
            .query_row("SELECT COUNT(*) FROM main.movements", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        4
    );
    assert_eq!(
        reopened
            .query_row("SELECT COUNT(*) FROM main.audit_event", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        initial_audit + 4
    );
    assert_eq!(movements.len(), 4);
    assert_eq!(accounts.len(), 2);
    for account in accounts {
        if account.id == source {
            assert_eq!(account.initial_balance_cents, -1234);
            assert_eq!(account.balance_cents(&movements), Ok(-1179));
        } else {
            assert_eq!(account.id, destination);
            assert_eq!(account.initial_balance_cents, 7654);
            assert_eq!(account.balance_cents(&movements), Ok(7669));
        }
    }
}
