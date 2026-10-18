use super::exit_persistence::{assert_health, setup, FixedClock, FixedIds};
use millani_artes::application::{
    create_entry::create_entry, create_exit::create_exit, create_transfer::create_transfer,
    list_accounts::list_accounts,
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
    let mut stmt=reopened.prepare("SELECT id,kind,amount_cents,context,account_id,source_account_id,destination_account_id FROM main.movements ORDER BY id").unwrap();
    let movements = stmt
        .query_map([], |row| {
            let id: Vec<u8> = row.get(0)?;
            let kind: String = row.get(1)?;
            let context: String = row.get(3)?;
            let account = |column: usize| {
                row.get::<_, Vec<u8>>(column)
                    .unwrap()
                    .try_into()
                    .expect("16-byte fixture ID")
            };
            Ok(Movement {
                id: id.try_into().expect("16-byte movement ID"),
                amount_cents: row.get(2)?,
                context: Context::try_from(context.as_str()).unwrap(),
                kind: match kind.as_str() {
                    "entry" => MovementKind::Entry {
                        account_id: account(4),
                    },
                    "exit" => MovementKind::Exit {
                        account_id: account(4),
                    },
                    "transfer" => MovementKind::Transfer {
                        source_account_id: account(5),
                        destination_account_id: account(6),
                    },
                    _ => panic!("invalid fixture kind"),
                },
            })
        })
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap();
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
