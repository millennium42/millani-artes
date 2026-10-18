use super::exit_persistence::{assert_health, setup, snapshot, FixedClock, FixedIds};
use millani_artes::application::create_transfer::{create_transfer, CreateTransferError};
use millani_artes::database::open_database;
use millani_artes::domain::Context;
use millani_artes::infrastructure::movements::AdapterSqliteMovementStore;
use rusqlite::types::Value;
use std::error::Error;

#[test]
fn transfer_rollback_extended_faults_preserve_database_and_allow_fresh_adapter() {
    for case in 0..3 {
        let (_directory, path, source) = setup();
        let destination = [0xA2u8; 16];
        let connection = open_database(&path).unwrap();
        connection
            .execute(
                "INSERT INTO main.accounts (id,name,account_type,initial_balance_cents) VALUES (?1,?2,?3,?4)",
                rusqlite::params![destination.as_slice(), "Destino sintético", "bank_pix", 7654i64],
            )
            .unwrap();
        match case {
            1 => connection
                .execute_batch(
                    "CREATE TRIGGER main.ignore_transfer_audit BEFORE INSERT ON audit_event
                     WHEN NEW.operation='movement_transfer_create'
                     BEGIN SELECT RAISE(IGNORE); END;",
                )
                .unwrap(),
            2 => {
                connection
                    .execute_batch(
                        "CREATE TRIGGER main.break_transfer_commit AFTER INSERT ON movements
                         WHEN NEW.kind='transfer'
                         BEGIN DELETE FROM accounts WHERE id=NEW.destination_account_id; END;",
                    )
                    .unwrap();
                connection
                    .pragma_update(None, "defer_foreign_keys", true)
                    .unwrap();
                assert_eq!(
                    connection
                        .pragma_query_value(None, "defer_foreign_keys", |row| row.get::<_, i64>(0))
                        .unwrap(),
                    1
                );
            }
            _ => {}
        }
        let baseline = {
            let observer = open_database(&path).unwrap();
            assert_health(&observer);
            snapshot(&observer)
        };
        assert!(baseline
            .2
            .iter()
            .any(|row| row[0] == Value::Blob(vec![0xB2; 16])));
        let event_id = if case == 0 { [0xB2; 16] } else { [0x41; 16] };
        let mut ids = FixedIds::new([0x31; 16], event_id);
        let mut store = AdapterSqliteMovementStore::new(connection);
        let error = create_transfer(
            source,
            destination,
            7,
            Context::Factory,
            &mut ids,
            &FixedClock,
            &mut store,
        )
        .err()
        .unwrap_or_else(|| panic!("fault {case} returned success"));
        assert_eq!(error, CreateTransferError::StoreFailed);
        assert_eq!(format!("{error}"), "transfer could not be stored");
        assert_eq!(format!("{error:?}"), "StoreFailed");
        assert!(error.source().is_none());
        assert_eq!(ids.calls, 2);
        {
            let observer = open_database(&path).unwrap();
            assert_eq!(snapshot(&observer), baseline, "fault {case} leaked writes");
            assert_health(&observer);
            match case {
                1 => observer
                    .execute_batch("DROP TRIGGER main.ignore_transfer_audit")
                    .unwrap(),
                2 => observer
                    .execute_batch("DROP TRIGGER main.break_transfer_commit")
                    .unwrap(),
                _ => {}
            }
        }
        let mut retry_ids = FixedIds::new([0x51; 16], [0x61; 16]);
        let error = create_transfer(
            source,
            destination,
            1,
            Context::Home,
            &mut retry_ids,
            &FixedClock,
            &mut store,
        )
        .err()
        .expect("failed adapter must stay poisoned");
        assert_eq!(error, CreateTransferError::StoreFailed);
        assert_eq!(retry_ids.calls, 2);
        drop(store);
        {
            let reopened = open_database(&path).unwrap();
            assert_eq!(
                snapshot(&reopened),
                baseline,
                "fault {case} retry changed database"
            );
            assert_health(&reopened);
        }
        let mut fresh = AdapterSqliteMovementStore::new(open_database(&path).unwrap());
        let mut fresh_ids = FixedIds::new([0x71; 16], [0x81; 16]);
        create_transfer(
            source,
            destination,
            1,
            Context::Home,
            &mut fresh_ids,
            &FixedClock,
            &mut fresh,
        )
        .expect("fresh adapter must persist after rollback");
        assert_eq!(fresh_ids.calls, 2);
        drop(fresh);
        let reopened = open_database(&path).unwrap();
        assert_health(&reopened);
        let (accounts, movements, audit, ledger) = snapshot(&reopened);
        assert_eq!(accounts, baseline.0);
        assert_eq!(ledger, baseline.3);
        assert_eq!(
            movements,
            vec![vec![
                Value::Blob(vec![0x71; 16]),
                Value::Text("transfer".into()),
                Value::Integer(1),
                Value::Text("home".into()),
                Value::Null,
                Value::Blob(source.to_vec()),
                Value::Blob(destination.to_vec()),
            ]]
        );
        assert_eq!(audit.len(), baseline.2.len() + 1);
        assert_eq!(
            audit[0],
            vec![
                Value::Blob(vec![0x81; 16]),
                Value::Integer(1234),
                Value::Text("movement_transfer_create".into()),
                Value::Text("movement".into()),
                Value::Blob(vec![0x71; 16]),
            ]
        );
        assert_eq!(audit[1..], baseline.2);
    }
}
