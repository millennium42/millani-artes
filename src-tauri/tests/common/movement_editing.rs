use super::exit_persistence::{setup, snapshot, FixedClock, FixedIds};
use millani_artes::application::calculate_context_result::calculate_context_result;
use millani_artes::application::create_account::create_account;
use millani_artes::application::create_entry::create_entry;
use millani_artes::application::edit_movement::{edit_movement, MovementEditingStore};
use millani_artes::application::list_accounts::list_accounts;
use millani_artes::application::list_movements::list_movements;
use millani_artes::database::open_database;
use millani_artes::domain::{AccountType, Context, Movement, MovementKind};
use millani_artes::infrastructure::accounts::AdapterSqliteAccountStore;
use millani_artes::infrastructure::movements::AdapterSqliteMovementStore;
use rusqlite::types::Value;
use std::error::Error;

fn proposed(id: [u8; 16], kind: MovementKind, amount: i64, ctx: Context) -> Movement {
    Movement {
        id,
        kind,
        amount_cents: amount,
        context: ctx,
    }
}
fn seed(path: &std::path::Path, account: [u8; 16]) {
    create_entry(
        account,
        10,
        Context::Home,
        &mut FixedIds::new([0x11; 16], [0x21; 16]),
        &FixedClock,
        &mut AdapterSqliteMovementStore::new(open_database(path).unwrap()),
    )
    .unwrap();
}

#[test]
fn canonical_edit_replaces_all_fields_without_creating_another_movement() {
    let (_dir, path, a) = setup();
    create_account(
        "Outra conta".into(),
        AccountType::Other,
        500,
        &mut FixedIds::new([0xA3; 16], [0xB3; 16]),
        &FixedClock,
        &mut AdapterSqliteAccountStore::new(open_database(&path).unwrap()),
    )
    .unwrap();
    let b = [0xA3; 16];
    seed(&path, a);
    let baseline = snapshot(&open_database(&path).unwrap());
    let mut store = AdapterSqliteMovementStore::new(open_database(&path).unwrap());
    for (index, kind, amount, ctx, a_delta, b_delta, home, factory) in [
        (
            0,
            MovementKind::Exit { account_id: b },
            7,
            Context::Factory,
            0,
            -7,
            0,
            -7,
        ),
        (
            1,
            MovementKind::Transfer {
                source_account_id: b,
                destination_account_id: a,
            },
            5,
            Context::Home,
            5,
            -5,
            0,
            0,
        ),
        (
            2,
            MovementKind::Entry { account_id: b },
            1,
            Context::Factory,
            0,
            1,
            0,
            1,
        ),
        (
            3,
            MovementKind::Transfer {
                source_account_id: a,
                destination_account_id: b,
            },
            1,
            Context::Factory,
            -1,
            1,
            0,
            0,
        ),
    ] {
        let m = edit_movement(
            proposed([0x11; 16], kind, amount, ctx),
            &mut FixedIds::new([0x31 + index; 16], [0; 16]),
            &FixedClock,
            &mut store,
        )
        .unwrap();
        assert_eq!(m.id, [0x11; 16]);
        let reader = AdapterSqliteMovementStore::new(open_database(&path).unwrap());
        let history = list_movements(&reader).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].id, [0x11; 16]);
        assert_eq!(history[0].amount_cents, amount);
        let accounts = list_accounts(&AdapterSqliteAccountStore::new(
            open_database(&path).unwrap(),
        ))
        .unwrap();
        assert_eq!(
            accounts
                .iter()
                .find(|x| x.id == a)
                .unwrap()
                .balance_cents(&history)
                .unwrap(),
            -1234 + a_delta
        );
        assert_eq!(
            accounts
                .iter()
                .find(|x| x.id == b)
                .unwrap()
                .balance_cents(&history)
                .unwrap(),
            500 + b_delta
        );
        assert_eq!(
            calculate_context_result(&Context::Home, &reader).unwrap(),
            home
        );
        assert_eq!(
            calculate_context_result(&Context::Factory, &reader).unwrap(),
            factory
        );
        let current = snapshot(&open_database(&path).unwrap());
        assert_eq!(current.0, baseline.0);
        assert_eq!(current.3, baseline.3);
        assert_eq!(current.2.len(), baseline.2.len() + index as usize + 1);
        assert!(current.2.iter().any(|row| row
            == &vec![
                Value::Blob(vec![0x31 + index; 16]),
                Value::Integer(1234),
                Value::Text("movement_update".into()),
                Value::Text("movement".into()),
                Value::Blob(vec![0x11; 16])
            ]));
    }
    drop(store);
    let c = open_database(&path).unwrap();
    super::exit_persistence::assert_health(&c);
}

#[test]
fn invalid_direct_edits_preserve_database_and_adapter_remains_usable() {
    let (_dir, path, a) = setup();
    seed(&path, a);
    let baseline = snapshot(&open_database(&path).unwrap());
    let mut store = AdapterSqliteMovementStore::new(open_database(&path).unwrap());
    for k in 0..3 {
        for amount in [0, -1, i64::MIN] {
            let kind = match k {
                0 => MovementKind::Entry { account_id: a },
                1 => MovementKind::Exit { account_id: a },
                _ => MovementKind::Transfer {
                    source_account_id: a,
                    destination_account_id: [0xA3; 16],
                },
            };
            let e = store
                .persist_movement_edit_with_audit(
                    &proposed([0x11; 16], kind, amount, Context::Home),
                    [0x41; 16],
                    1234,
                )
                .unwrap_err();
            assert_eq!(e.to_string(), "movement could not be edited");
            assert_eq!(format!("{e:?}"), "MovementEditStoreError");
            assert!(e.source().is_none());
            assert_eq!(snapshot(&open_database(&path).unwrap()), baseline);
        }
    }
    assert!(store
        .persist_movement_edit_with_audit(
            &proposed(
                [0x11; 16],
                MovementKind::Transfer {
                    source_account_id: a,
                    destination_account_id: a
                },
                1,
                Context::Factory
            ),
            [0x41; 16],
            1234
        )
        .is_err());
    edit_movement(
        proposed(
            [0x11; 16],
            MovementKind::Exit { account_id: a },
            2,
            Context::Home,
        ),
        &mut FixedIds::new([0x42; 16], [0; 16]),
        &FixedClock,
        &mut store,
    )
    .unwrap();
    drop(store);
    super::exit_persistence::assert_health(&open_database(&path).unwrap());
}

#[test]
fn missing_id_fk_and_audit_faults_roll_back_and_poison_only_failed_adapter() {
    for fault in 0..7 {
        let (_dir, path, a) = setup();
        seed(&path, a);
        let c = open_database(&path).unwrap();
        match fault {
            3=>c.execute_batch("CREATE TRIGGER main.ignore_movement_update BEFORE UPDATE ON movements BEGIN SELECT RAISE(IGNORE); END;").unwrap(),
            4=>c.execute_batch("CREATE TRIGGER main.ignore_edit_audit BEFORE INSERT ON audit_event WHEN NEW.operation='movement_update' BEGIN SELECT RAISE(IGNORE); END;").unwrap(),
            5=>c.execute_batch("CREATE TABLE main.edit_commit_fault(id INTEGER REFERENCES accounts(id) DEFERRABLE INITIALLY DEFERRED); CREATE TRIGGER main.edit_deferred_fault AFTER UPDATE ON movements BEGIN INSERT INTO edit_commit_fault VALUES(999); END;").unwrap(),
            6=>c.execute_batch("CREATE TRIGGER main.abort_edit_audit BEFORE INSERT ON audit_event WHEN NEW.operation='movement_update' BEGIN SELECT RAISE(ABORT,'PRIVATE_AUDIT_FAILURE'); END;").unwrap(),
            _=>{},
        }
        let baseline = snapshot(&c);
        drop(c);
        let mut store = AdapterSqliteMovementStore::new(open_database(&path).unwrap());
        let id = if fault == 0 { [0xEE; 16] } else { [0x11; 16] };
        let target = if fault == 1 { [0xEE; 16] } else { a };
        let audit = if fault == 2 { [0x21; 16] } else { [0x51; 16] };
        let e = store
            .persist_movement_edit_with_audit(
                &proposed(
                    id,
                    MovementKind::Exit { account_id: target },
                    1,
                    Context::Factory,
                ),
                audit,
                1234,
            )
            .unwrap_err();
        assert_eq!(e.to_string(), "movement could not be edited");
        assert!(e.source().is_none());
        assert_eq!(format!("{e:?}"), "MovementEditStoreError");
        assert!(list_movements(&store).is_err());
        assert!(store
            .persist_movement_edit_with_audit(
                &proposed(
                    [0x11; 16],
                    MovementKind::Entry { account_id: a },
                    1,
                    Context::Home
                ),
                [0x52; 16],
                1234
            )
            .is_err());
        drop(store);
        let c = open_database(&path).unwrap();
        assert_eq!(snapshot(&c), baseline);
        super::exit_persistence::assert_health(&c);
        if fault == 5 {
            assert_eq!(
                c.query_row("SELECT COUNT(*) FROM main.edit_commit_fault", [], |r| r
                    .get::<_, i64>(0))
                    .unwrap(),
                0
            );
        }
        let reader = AdapterSqliteMovementStore::new(c);
        let rows = list_movements(&reader).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].amount_cents, 10);
    }
}

#[test]
fn existing_category_reference_and_unrelated_movement_are_preserved() {
    let (_dir, path, a) = setup();
    seed(&path, a);
    let mut categories = millani_artes::infrastructure::categories::AdapterSqliteCategoryStore::new(
        open_database(&path).unwrap(),
    );
    let cat = millani_artes::application::create_category::create_category(
        "Categoria histórica".into(),
        &mut FixedIds::new([0x61; 16], [0x62; 16]),
        &FixedClock,
        &mut categories,
    )
    .unwrap();
    drop(categories);
    let c = open_database(&path).unwrap();
    c.execute(
        "UPDATE main.movements SET category_id=?1 WHERE id=?2",
        rusqlite::params![cat.id.as_slice(), [0x11_u8; 16].as_slice()],
    )
    .unwrap();
    drop(c);
    let mut store = AdapterSqliteMovementStore::new(open_database(&path).unwrap());
    create_entry(
        a,
        20,
        Context::Factory,
        &mut FixedIds::new([0x12; 16], [0x22; 16]),
        &FixedClock,
        &mut store,
    )
    .unwrap();
    let before = snapshot(&open_database(&path).unwrap());
    edit_movement(
        proposed(
            [0x11; 16],
            MovementKind::Exit { account_id: a },
            i64::MAX,
            Context::Factory,
        ),
        &mut FixedIds::new([0x63; 16], [0; 16]),
        &FixedClock,
        &mut store,
    )
    .unwrap();
    drop(store);
    let c = open_database(&path).unwrap();
    let after = snapshot(&c);
    assert_eq!(after.0, before.0);
    assert_eq!(after.1.len(), 2);
    assert_eq!(after.1[1], before.1[1]);
    assert_eq!(after.3, before.3);
    assert_eq!(
        c.query_row(
            "SELECT category_id FROM main.movements WHERE id=?1",
            [vec![0x11_u8; 16]],
            |r| r.get::<_, Vec<u8>>(0)
        )
        .unwrap(),
        cat.id
    );
    assert_eq!(
        c.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
}
