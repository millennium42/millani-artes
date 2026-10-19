use millani_artes::domain::ports::Clock;

use super::exit_persistence::{setup, snapshot, FixedClock, FixedIds};
use millani_artes::application::create_account::create_account;
use millani_artes::application::create_entry::create_entry;
use millani_artes::application::edit_movement::{edit_movement, EditMovementError};
use millani_artes::application::list_movements::list_movements;
use millani_artes::database::open_database;
use millani_artes::domain::{AccountType, Context, Movement, MovementKind};
use millani_artes::infrastructure::accounts::AdapterSqliteAccountStore;
use millani_artes::infrastructure::movements::AdapterSqliteMovementStore;
use rusqlite::types::Value;
use std::time::{Duration, UNIX_EPOCH};

struct AuditClock(u64);

impl Clock for AuditClock {
    fn now(&self) -> std::time::SystemTime {
        UNIX_EPOCH + Duration::from_millis(self.0)
    }
}

#[test]
fn movement_edits_append_exact_immutable_audit_and_rollback_if_audit_fails() {
    let (_dir, path, a) = setup();

    create_account(
        "Outra conta".into(),
        AccountType::Other,
        0,
        &mut FixedIds::new([0xA3; 16], [0xB3; 16]),
        &FixedClock,
        &mut AdapterSqliteAccountStore::new(open_database(&path).unwrap()),
    )
    .unwrap();
    let b = [0xA3; 16];

    create_entry(
        a,
        10,
        Context::Home,
        &mut FixedIds::new([0x11; 16], [0x21; 16]),
        &FixedClock,
        &mut AdapterSqliteMovementStore::new(open_database(&path).unwrap()),
    )
    .unwrap();

    let mut before = snapshot(&open_database(&path).unwrap());

    let mut store = AdapterSqliteMovementStore::new(open_database(&path).unwrap());

    for (index, (kind_key, ctx_key, amount)) in [
        (0u8, 0u8, 1i64),
        (1, 1, 2),
        (2, 0, 3),
        (0, 1, 4),
        (1, 0, 5),
        (2, 1, 6),
        (2, 1, 6),
    ]
    .into_iter()
    .enumerate()
    {
        let event_id = [0x40 + index as u8; 16];
        let time = 1234 + index as u64;
        let mut ids = FixedIds::new(event_id, [0; 16]);

        let kind = match kind_key {
            0 => MovementKind::Entry { account_id: a },
            1 => MovementKind::Exit { account_id: b },
            _ => MovementKind::Transfer {
                source_account_id: a,
                destination_account_id: b,
            },
        };
        let ctx = match ctx_key {
            0 => Context::Home,
            _ => Context::Factory,
        };

        let m = edit_movement(
            Movement {
                id: [0x11; 16],
                kind,
                amount_cents: amount,
                context: ctx,
            },
            &mut ids,
            &AuditClock(time),
            &mut store,
        )
        .unwrap();

        assert_eq!(m.id, [0x11; 16]);
        assert_eq!(m.amount_cents, amount);

        let c = open_database(&path).unwrap();
        let after = snapshot(&c);

        assert_eq!(after.2.len(), before.2.len() + 1, "edit audit count");

        let expected_audit = vec![
            Value::Blob(event_id.to_vec()),
            Value::Integer(time as i64),
            Value::Text("movement_update".into()),
            Value::Text("movement".into()),
            Value::Blob(vec![0x11; 16]),
        ];
        assert!(after.2.iter().any(|row| row == &expected_audit));

        for prev_row in &before.2 {
            assert!(after.2.iter().any(|row| row == prev_row));
        }

        assert_eq!(after.0, before.0);

        assert_eq!(after.3, before.3);

        let reader = AdapterSqliteMovementStore::new(open_database(&path).unwrap());
        let movements = list_movements(&reader).unwrap();
        assert_eq!(movements.len(), 1);
        assert_eq!(movements[0].id, [0x11; 16]);
        assert_eq!(movements[0].amount_cents, amount);
        assert!(matches!(
            (&movements[0].context, ctx_key),
            (Context::Home, 0) | (Context::Factory, 1)
        ));
        assert!(match &movements[0].kind {
            MovementKind::Entry { account_id } => kind_key == 0 && *account_id == a,
            MovementKind::Exit { account_id } => kind_key == 1 && *account_id == b,
            MovementKind::Transfer {
                source_account_id,
                destination_account_id,
            } => kind_key == 2 && *source_account_id == a && *destination_account_id == b,
        });

        drop(c);

        let reopened = snapshot(&open_database(&path).unwrap());
        assert_eq!(reopened, after);
        before = reopened;

        assert_eq!(ids.calls, 1);
    }

    drop(store);

    let c = open_database(&path).unwrap();
    let event_id = [0x40_u8; 16];

    let result = c.execute(
        "UPDATE main.audit_event SET operation='tampered' WHERE event_id=?1",
        rusqlite::params![event_id.to_vec()],
    );
    assert!(result.is_err());

    let after_tamper = snapshot(&c);
    assert_eq!(after_tamper, before);

    let result = c.execute(
        "DELETE FROM main.audit_event WHERE event_id=?1",
        rusqlite::params![event_id.to_vec()],
    );
    assert!(result.is_err());

    let after_delete = snapshot(&c);
    assert_eq!(after_delete, before);

    drop(c);

    let c = open_database(&path).unwrap();
    c.execute_batch(
        "CREATE TRIGGER main.reject_edit_audit BEFORE INSERT ON audit_event WHEN NEW.operation='movement_update' BEGIN SELECT RAISE(ABORT,'PRIVATE_AUDIT_FAILURE'); END;",
    )
    .unwrap();

    let baseline_with_trigger = snapshot(&c);
    drop(c);

    let mut store = AdapterSqliteMovementStore::new(open_database(&path).unwrap());
    let mut ids = FixedIds::new([0x50; 16], [0; 16]);

    let result = edit_movement(
        Movement {
            id: [0x11; 16],
            kind: MovementKind::Entry { account_id: a },
            amount_cents: 999,
            context: Context::Home,
        },
        &mut ids,
        &AuditClock(9000),
        &mut store,
    );

    let err = result.err().expect("audit failure must reject edit");
    assert_eq!(err, EditMovementError::StoreFailed);

    assert_eq!(ids.calls, 1);

    assert!(list_movements(&store).is_err());

    drop(store);

    let observer = open_database(&path).unwrap();
    assert_eq!(snapshot(&observer), baseline_with_trigger);
    super::exit_persistence::assert_health(&observer);

    let fresh_reader = AdapterSqliteMovementStore::new(open_database(&path).unwrap());
    let movements = list_movements(&fresh_reader).unwrap();
    assert_eq!(movements.len(), 1);
    assert_eq!(movements[0].id, [0x11; 16]);

    let c = open_database(&path).unwrap();
    let reader = AdapterSqliteMovementStore::new(c);
    let movements = list_movements(&reader).unwrap();
    assert_eq!(movements.len(), 1);
    assert_eq!(movements[0].id, [0x11; 16]);
    assert_eq!(movements[0].amount_cents, 6);
    assert!(matches!(&movements[0].context, Context::Factory));
    assert!(
        matches!(&movements[0].kind,MovementKind::Transfer{source_account_id,destination_account_id} if *source_account_id==a && *destination_account_id==b)
    );
}
