use super::exit_persistence::{assert_health, setup, snapshot, FixedClock, FixedIds};
use millani_artes::application::create_exit::{create_exit, CreateExitError};
use millani_artes::database::open_database;
use millani_artes::domain::Context;
use millani_artes::infrastructure::movements::AdapterSqliteMovementStore;
use std::error::Error;

#[test]
fn exit_store_failures_roll_back_and_poison_only_the_failed_adapter() {
    for case in 0..4 {
        let (_directory, path, account_id) = setup();
        if case == 1 {
            let mut seed_store = AdapterSqliteMovementStore::new(open_database(&path).unwrap());
            let mut ids = FixedIds::new([0x11; 16], [0x21; 16]);
            create_exit(
                account_id,
                1,
                Context::Home,
                &mut ids,
                &FixedClock,
                &mut seed_store,
            )
            .expect("seed movement for duplicate id");
            drop(seed_store);
        } else if case == 2 {
            let connection = open_database(&path).unwrap();
            connection
                .execute_batch(
                    "CREATE TRIGGER main.fail_exit_audit BEFORE INSERT ON audit_event
                 WHEN NEW.operation = 'movement_exit_create'
                 BEGIN SELECT RAISE(ABORT, 'PRIVATE_TRIGGER_AUDIT'); END;",
                )
                .unwrap();
        } else if case == 3 {
            let connection = open_database(&path).unwrap();
            connection
                .execute_batch(
                    "CREATE TRIGGER main.ignore_exit_movement BEFORE INSERT ON movements
                 BEGIN SELECT RAISE(IGNORE); END;",
                )
                .unwrap();
        }
        let baseline = {
            let observer = open_database(&path).unwrap();
            assert_health(&observer);
            snapshot(&observer)
        };
        let mut store = AdapterSqliteMovementStore::new(open_database(&path).unwrap());
        let movement_id = if case == 1 { [0x11; 16] } else { [0x31; 16] };
        let input_account = if case == 0 { [0xFF; 16] } else { account_id };
        let mut ids = FixedIds::new(movement_id, [0x41; 16]);
        let error = create_exit(
            input_account,
            1,
            Context::Factory,
            &mut ids,
            &FixedClock,
            &mut store,
        )
        .err()
        .expect("physical store failure must be mapped");
        assert!(matches!(error, CreateExitError::StoreFailed));
        assert_eq!(ids.calls, 2);
        for text in [format!("{error}"), format!("{error:?}")] {
            assert!(!text.contains("PRIVATE_TRIGGER"));
            assert!(!text.is_empty());
        }
        assert!(error.source().is_none());
        {
            let observer = open_database(&path).unwrap();
            assert_eq!(snapshot(&observer), baseline);
            assert_health(&observer);
            if case == 2 {
                observer
                    .execute_batch("DROP TRIGGER main.fail_exit_audit")
                    .unwrap();
            } else if case == 3 {
                observer
                    .execute_batch("DROP TRIGGER main.ignore_exit_movement")
                    .unwrap();
            }
        }
        let mut retry_ids = FixedIds::new([0x51; 16], [0x61; 16]);
        let retry_error = create_exit(
            account_id,
            1,
            Context::Home,
            &mut retry_ids,
            &FixedClock,
            &mut store,
        )
        .err()
        .expect("poisoned adapter must reject a valid retry");
        assert!(matches!(retry_error, CreateExitError::StoreFailed));
        drop(store);
        let reopened = open_database(&path).unwrap();
        assert_eq!(snapshot(&reopened), baseline);
        assert_health(&reopened);
    }
}
