use super::exit_persistence::{assert_health, setup, snapshot, FixedClock, FixedIds};
use millani_artes::application::create_transfer::{
    create_transfer, CreateTransferError, TransferCreationStore,
};
use millani_artes::database::open_database;
use millani_artes::domain::{Context, Movement, MovementKind};
use millani_artes::infrastructure::movements::AdapterSqliteMovementStore;
use rusqlite::types::Value;
use std::error::Error;

#[test]
fn transfer_use_case_persists_both_contexts_and_audit_in_one_database() {
    let (_directory, path, source_account_id) = setup();
    // Seed destination account [0xA2;16] via SQL (no extra audit)
    let connection = open_database(&path).unwrap();
    connection
        .execute(
            "INSERT INTO main.accounts (id, name, account_type, initial_balance_cents) VALUES (?, ?, ?, ?)",
            (
                Value::Blob([0xA2; 16].to_vec()),
                Value::Text("Destino".into()),
                Value::Text("bank_pix".into()),
                Value::Integer(-1234),
            ),
        )
        .unwrap();
    drop(connection);

    let baseline = snapshot(&open_database(&path).unwrap());
    let mut store = AdapterSqliteMovementStore::new(open_database(&path).unwrap());
    for (movement_id, audit_id, amount, context, from, to) in [
        (
            [0x11; 16],
            [0x21; 16],
            i64::MAX,
            Context::Home,
            source_account_id,
            [0xA2; 16],
        ),
        (
            [0x12; 16],
            [0x22; 16],
            1,
            Context::Factory,
            [0xA2; 16],
            source_account_id,
        ),
    ] {
        let mut ids = FixedIds::new(movement_id, audit_id);
        let movement =
            create_transfer(from, to, amount, context, &mut ids, &FixedClock, &mut store)
                .expect("persist transfer");
        assert_eq!(movement.id, movement_id);
        assert_eq!(movement.amount_cents, amount);
        assert!(matches!(
            &movement.kind,
            MovementKind::Transfer {
                source_account_id: src,
                destination_account_id: dst
            } if *src == from && *dst == to
        ));
        assert_eq!(ids.calls, 2);
    }
    drop(store);
    let connection = open_database(&path).unwrap();
    assert_health(&connection);
    let (accounts, movements, audit, ledger) = snapshot(&connection);
    assert_eq!(accounts, baseline.0);
    assert_eq!(ledger, baseline.3);
    assert_eq!(
        movements,
        vec![
            vec![
                Value::Blob(vec![0x11; 16]),
                Value::Text("transfer".into()),
                Value::Integer(i64::MAX),
                Value::Text("home".into()),
                Value::Null,
                Value::Blob(source_account_id.to_vec()),
                Value::Blob([0xA2; 16].to_vec())
            ],
            vec![
                Value::Blob(vec![0x12; 16]),
                Value::Text("transfer".into()),
                Value::Integer(1),
                Value::Text("factory".into()),
                Value::Null,
                Value::Blob([0xA2; 16].to_vec()),
                Value::Blob(source_account_id.to_vec())
            ],
        ]
    );
    assert_eq!(
        audit,
        vec![
            vec![
                Value::Blob(vec![0x21; 16]),
                Value::Integer(1234),
                Value::Text("movement_transfer_create".into()),
                Value::Text("movement".into()),
                Value::Blob(vec![0x11; 16])
            ],
            vec![
                Value::Blob(vec![0x22; 16]),
                Value::Integer(1234),
                Value::Text("movement_transfer_create".into()),
                Value::Text("movement".into()),
                Value::Blob(vec![0x12; 16])
            ],
            vec![
                Value::Blob(vec![0xB2; 16]),
                Value::Integer(1234),
                Value::Text("account_create".into()),
                Value::Text("account".into()),
                Value::Blob(source_account_id.to_vec())
            ],
        ]
    );
}

#[test]
fn invalid_direct_transfers_leave_store_usable_and_database_unchanged() {
    let (_directory, path, source_account_id) = setup();
    // Seed destination account [0xA2;16] via SQL (no extra audit)
    let connection = open_database(&path).unwrap();
    connection
        .execute(
            "INSERT INTO main.accounts (id, name, account_type, initial_balance_cents) VALUES (?, ?, ?, ?)",
            (
                Value::Blob([0xA2; 16].to_vec()),
                Value::Text("Destino".into()),
                Value::Text("bank_pix".into()),
                Value::Integer(-1234),
            ),
        )
        .unwrap();
    drop(connection);

    let baseline = {
        let connection = open_database(&path).unwrap();
        snapshot(&connection)
    };
    let mut store = AdapterSqliteMovementStore::new(open_database(&path).unwrap());
    for case in 0..6 {
        let kind = match case {
            3 => MovementKind::Entry {
                account_id: source_account_id,
            },
            4 => MovementKind::Exit {
                account_id: source_account_id,
            },
            5 => MovementKind::Transfer {
                source_account_id,
                destination_account_id: source_account_id,
            },
            _ => MovementKind::Transfer {
                source_account_id,
                destination_account_id: [0xA2; 16],
            },
        };
        let context = if case % 2 == 0 {
            Context::Home
        } else {
            Context::Factory
        };
        let movement = Movement {
            id: [0x30 + case; 16],
            kind,
            amount_cents: match case {
                0 => 0,
                1 => -1,
                2 => i64::MIN,
                _ => 1,
            },
            context,
        };
        let error = store
            .persist_transfer_with_audit(&movement, [0x40 + case; 16], 1234)
            .expect_err("invalid movement rejected");
        assert_eq!(format!("{error}"), "transfer could not be stored");
        assert_eq!(format!("{error:?}"), "TransferStoreError");
        assert!(error.source().is_none());
        let observer = open_database(&path).unwrap();
        assert_eq!(snapshot(&observer), baseline);
        assert_health(&observer);
    }
    let mut ids = FixedIds::new([0x51; 16], [0x61; 16]);
    let result = create_transfer(
        source_account_id,
        [0xA2; 16],
        1,
        Context::Factory,
        &mut ids,
        &FixedClock,
        &mut store,
    );
    assert!(result.is_ok());
    drop(store);
    let connection = open_database(&path).unwrap();
    assert_health(&connection);
    let (accounts, movements, audit, ledger) = snapshot(&connection);
    assert_eq!(accounts, baseline.0);
    assert_eq!(ledger, baseline.3);
    assert_eq!(
        movements,
        vec![vec![
            Value::Blob(vec![0x51; 16]),
            Value::Text("transfer".into()),
            Value::Integer(1),
            Value::Text("factory".into()),
            Value::Null,
            Value::Blob(source_account_id.to_vec()),
            Value::Blob([0xA2; 16].to_vec()),
        ]]
    );
    assert_eq!(audit.len(), baseline.2.len() + 1);
    assert_eq!(
        audit[0],
        vec![
            Value::Blob(vec![0x61; 16]),
            Value::Integer(1234),
            Value::Text("movement_transfer_create".into()),
            Value::Text("movement".into()),
            Value::Blob(vec![0x51; 16]),
        ]
    );
    assert_eq!(audit[1..], baseline.2);
}

#[test]
fn transfer_store_failures_roll_back_and_poison_only_the_failed_adapter() {
    for case in 0..5 {
        let (_directory, path, source_account_id) = setup();
        // Seed destination account [0xA2;16] via SQL (no extra audit)
        let connection = open_database(&path).unwrap();
        connection
            .execute(
                "INSERT INTO main.accounts (id, name, account_type, initial_balance_cents) VALUES (?, ?, ?, ?)",
                (
                    Value::Blob([0xA2; 16].to_vec()),
                    Value::Text("Destino".into()),
                    Value::Text("bank_pix".into()),
                    Value::Integer(-1234),
                ),
            )
            .unwrap();
        drop(connection);

        if case == 1 {
            let mut seed_store = AdapterSqliteMovementStore::new(open_database(&path).unwrap());
            let mut ids = FixedIds::new([0x11; 16], [0x21; 16]);
            create_transfer(
                source_account_id,
                [0xA2; 16],
                1,
                Context::Home,
                &mut ids,
                &FixedClock,
                &mut seed_store,
            )
            .expect("seed transfer for duplicate id");
            drop(seed_store);
        } else if case == 2 {
            let connection = open_database(&path).unwrap();
            connection
                .execute_batch(
                    "CREATE TRIGGER main.fail_transfer_audit BEFORE INSERT ON audit_event
                 WHEN NEW.operation = 'movement_transfer_create'
                 BEGIN SELECT RAISE(ABORT, 'PRIVATE_TRIGGER_AUDIT'); END;",
                )
                .unwrap();
        } else if case == 3 {
            let connection = open_database(&path).unwrap();
            connection
                .execute_batch(
                    "CREATE TRIGGER main.ignore_transfer_movement BEFORE INSERT ON movements
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
        let input_account = if case == 0 {
            [0xFF; 16]
        } else {
            source_account_id
        };
        let input_destination = if case == 4 { [0xFF; 16] } else { [0xA2; 16] };
        let mut ids = FixedIds::new(movement_id, [0x41; 16]);
        let error = create_transfer(
            input_account,
            input_destination,
            1,
            Context::Factory,
            &mut ids,
            &FixedClock,
            &mut store,
        )
        .err()
        .expect("store failure must be mapped");
        assert!(matches!(error, CreateTransferError::StoreFailed));
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
                    .execute_batch("DROP TRIGGER main.fail_transfer_audit")
                    .unwrap();
            } else if case == 3 {
                observer
                    .execute_batch("DROP TRIGGER main.ignore_transfer_movement")
                    .unwrap();
            }
        }
        let mut retry_ids = FixedIds::new([0x51; 16], [0x61; 16]);
        let retry_error = create_transfer(
            source_account_id,
            [0xA2; 16],
            1,
            Context::Home,
            &mut retry_ids,
            &FixedClock,
            &mut store,
        )
        .err()
        .expect("poisoned adapter must reject a valid retry");
        assert!(matches!(retry_error, CreateTransferError::StoreFailed));
        drop(store);
        let reopened = open_database(&path).unwrap();
        assert_eq!(snapshot(&reopened), baseline);
        assert_health(&reopened);
    }
}
