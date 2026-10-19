use super::entry_persistence::{FixedClock, FixedIds, Snapshot};
use super::TempDirectory;
use millani_artes::application::generate_variable_obligation::generate_variable_obligation;
use millani_artes::database::{
    execute_versioned_migration, open_database, RECURRENCES_MIGRATION_SQL,
    RECURRENCES_MIGRATION_VERSION,
};
use millani_artes::domain::{Context, ObligationState, RecurrenceObligation, VariableRecurrence};
use millani_artes::infrastructure::recurrences::AdapterSqliteRecurrenceStore;
use rusqlite::{params, types::Value, Connection};
use std::error::Error;
use std::path::PathBuf;
type State = (Snapshot, Vec<Vec<Value>>, Vec<Vec<Value>>, Vec<Vec<Value>>);
fn setup() -> (TempDirectory, PathBuf) {
    let (d, p, _) = super::exit_persistence::setup();
    drop(
        execute_versioned_migration(
            open_database(&p).unwrap(),
            RECURRENCES_MIGRATION_VERSION,
            RECURRENCES_MIGRATION_SQL,
        )
        .unwrap(),
    );
    (d, p)
}
fn rows(c: &Connection, sql: &str) -> Vec<Vec<Value>> {
    let mut s = c.prepare(sql).unwrap();
    let n = s.column_count();
    s.query_map([], |r| {
        (0..n)
            .map(|i| r.get(i))
            .collect::<rusqlite::Result<Vec<Value>>>()
    })
    .unwrap()
    .collect::<rusqlite::Result<_>>()
    .unwrap()
}
fn state(c: &Connection) -> State {
    (
        super::exit_persistence::snapshot(c),
        rows(c, "SELECT * FROM main.recurrences ORDER BY id"),
        rows(c, "SELECT * FROM main.recurrence_obligations ORDER BY id"),
        rows(c, "SELECT * FROM main.categories ORDER BY id"),
    )
}
fn health(c: &Connection) {
    assert!(c.is_autocommit());
    assert_eq!(
        rows(c, "PRAGMA main.integrity_check"),
        vec![vec![Value::Text("ok".into())]]
    );
    assert!(rows(c, "PRAGMA main.foreign_key_check").is_empty());
    assert_eq!(
        super::exit_persistence::snapshot(c).3,
        vec![1, 2, 3, 4, 5, 6, 7]
    );
}
fn rule(ctx: u8) -> VariableRecurrence {
    VariableRecurrence {
        id: [0x31 + ctx; 16],
        name: "  Café '🧵'\0; DROP TABLE accounts; --  ".into(),
        context: if ctx == 0 {
            Context::Home
        } else {
            Context::Factory
        },
        due_day: 31,
    }
}
use millani_artes::application::supply_obligation_amount::{
    supply_obligation_amount, ObligationAmountStore, SupplyObligationAmountError,
};
fn seed(p: &std::path::Path, ctx: u8, month: u8) -> RecurrenceObligation {
    let mut store = AdapterSqliteRecurrenceStore::new(open_database(p).unwrap());
    let mut ids = FixedIds::new([0x41 + month; 16], [0x61 + month; 16]);
    generate_variable_obligation(&rule(ctx), (2026, month), &mut ids, &FixedClock, &mut store)
        .unwrap()
}
fn target(ctx: u8) -> RecurrenceObligation {
    let mut o = rule(ctx)
        .generate_obligation([0x43; 16], (2026, 2))
        .unwrap();
    o.state = o.state.supply_amount(7).unwrap();
    o
}
fn opaque(e: &SupplyObligationAmountError) {
    assert!(e.source().is_none());
    assert!(!format!("{e:?}/{e}").contains("PRIVATE_SQL_MARKER"));
}
#[test]
fn amount_supply_updates_only_one_obligation_with_exact_audit_and_reopens() {
    for ctx in 0..2 {
        for amount in [1, i64::MAX] {
            let (_d, p) = setup();
            let o = seed(&p, ctx, 2);
            let _other = seed(&p, ctx, 3);
            let before = state(&open_database(&p).unwrap());
            let mut store = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
            let mut ids = FixedIds::new([0xe0; 16], [0xe1; 16]);
            let after_o =
                supply_obligation_amount(o, amount, &mut ids, &FixedClock, &mut store).unwrap();
            assert_eq!(ids.calls, 1);
            assert!(
                matches!(after_o.state,ObligationState::Payable{amount_cents}if amount_cents==amount)
            );
            let pure = rule(ctx)
                .generate_obligation([0x43; 16], (2026, 2))
                .unwrap();
            assert_eq!(
                (
                    after_o.id,
                    after_o.recurrence_id,
                    after_o.period_year,
                    after_o.period_month,
                    after_o.due_at_ms
                ),
                (
                    pure.id,
                    pure.recurrence_id,
                    pure.period_year,
                    pure.period_month,
                    pure.due_at_ms
                )
            );
            assert!(matches!(
                (after_o.context, ctx),
                (Context::Home, 0) | (Context::Factory, 1)
            ));
            drop(store);
            let c = open_database(&p).unwrap();
            health(&c);
            let after = state(&c);
            let mut expected = before.2.clone();
            expected[0][5] = Value::Text("payable".into());
            expected[0][6] = Value::Integer(amount);
            assert_eq!(after.2, expected);
            assert_eq!(after.1, before.1);
            assert_eq!(after.0 .0, before.0 .0);
            assert_eq!(after.0 .1, before.0 .1);
            assert_eq!(after.0 .3, before.0 .3);
            assert_eq!(after.3, before.3);
            assert_eq!(after.0 .2.len(), before.0 .2.len() + 1);
            for event in &before.0 .2 {
                assert_eq!(after.0 .2.iter().filter(|x| *x == event).count(), 1);
            }
            assert_eq!(
                after
                    .0
                     .2
                    .iter()
                    .filter(|r| r[0] == Value::Blob(vec![0xe0; 16]))
                    .collect::<Vec<_>>(),
                vec![&vec![
                    Value::Blob(vec![0xe0; 16]),
                    Value::Integer(1234),
                    Value::Text("recurrence_amount_supply".into()),
                    Value::Text("recurrence_obligation".into()),
                    Value::Blob(vec![0x43; 16])
                ]]
            );
        }
    }
}
#[test]
fn invalid_direct_amount_payloads_leave_store_usable_and_data_unchanged() {
    let (_d, p) = setup();
    let pending = seed(&p, 0, 2);
    let before = state(&open_database(&p).unwrap());
    let mut store = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
    for state in [
        ObligationState::AwaitingValue,
        ObligationState::Payable { amount_cents: 0 },
        ObligationState::Payable { amount_cents: -1 },
        ObligationState::Payable {
            amount_cents: i64::MIN,
        },
        ObligationState::Overdue { amount_cents: 7 },
        ObligationState::Paid {
            amount_cents: 7,
            movement_id: [1; 16],
        },
    ] {
        let mut o = target(0);
        o.state = state;
        let e = store
            .persist_obligation_amount_with_audit(&o, [0xe0; 16], 1234)
            .unwrap_err();
        assert!(e.source().is_none());
        assert!(!format!("{e:?}/{e}").contains(&rule(0).name));
        assert_eq!(state_snapshot(&p), before);
    }
    let mut ids = FixedIds::new([0xe0; 16], [0xe1; 16]);
    assert!(supply_obligation_amount(pending, 7, &mut ids, &FixedClock, &mut store).is_ok());
    health(&open_database(&p).unwrap());
}
fn state_snapshot(p: &std::path::Path) -> State {
    state(&open_database(p).unwrap())
}
#[test]
fn mismatched_metadata_cannot_update_a_row_and_poisoned_store_never_retries() {
    for case in 0..6 {
        let (_d, p) = setup();
        let pending = seed(&p, 0, 2);
        let before = state_snapshot(&p);
        let mut o = target(0);
        match case {
            0 => o.id = [9; 16],
            1 => o.recurrence_id = [9; 16],
            2 => o.period_year = 2025,
            3 => o.period_month = 3,
            4 => o.due_at_ms += 1,
            _ => o.context = Context::Factory,
        };
        let mut store = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
        let e = store
            .persist_obligation_amount_with_audit(&o, [0xe0; 16], 1234)
            .unwrap_err();
        assert!(e.source().is_none());
        assert_eq!(state_snapshot(&p), before);
        assert!(store
            .persist_obligation_amount_with_audit(&target(0), [0xe1; 16], 1234)
            .is_err());
        assert_eq!(state_snapshot(&p), before);
        let mut fresh = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
        let mut ids = FixedIds::new([0xe2; 16], [0xe3; 16]);
        assert!(supply_obligation_amount(pending, 7, &mut ids, &FixedClock, &mut fresh).is_ok());
        health(&open_database(&p).unwrap());
    }
}
#[test]
fn persisted_nonpending_fixed_missing_and_replayed_obligations_cannot_be_overwritten() {
    for case in 0..5 {
        let (_d, p) = setup();
        let pending = seed(&p, 0, 2);
        let c = open_database(&p).unwrap();
        match case {
            0 => {
                c.execute(
                    "UPDATE main.recurrence_obligations SET state='payable',amount_cents=17",
                    [],
                )
                .unwrap();
            }
            1 => {
                c.execute(
                    "UPDATE main.recurrence_obligations SET state='overdue',amount_cents=17",
                    [],
                )
                .unwrap();
            }
            2 => {
                c.execute("INSERT INTO main.movements(id,kind,amount_cents,context,account_id) VALUES (?1,'exit',17,'home',(SELECT id FROM main.accounts LIMIT 1))",params![[0x99u8;16].as_slice()]).unwrap();
                c.execute("UPDATE main.recurrence_obligations SET state='paid',amount_cents=17,movement_id=?1",params![[0x99u8;16].as_slice()]).unwrap();
            }
            3 => {
                c.execute(
                    "UPDATE main.recurrences SET kind='fixed',amount_cents=17",
                    [],
                )
                .unwrap();
            }
            _ => {
                c.execute("DELETE FROM main.recurrence_obligations", [])
                    .unwrap();
            }
        }
        health(&c);
        let before = state(&c);
        drop(c);
        let mut store = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
        let mut ids = FixedIds::new([0xe0; 16], [0xe1; 16]);
        let e = supply_obligation_amount(pending, 7, &mut ids, &FixedClock, &mut store)
            .err()
            .unwrap();
        opaque(&e);
        assert_eq!(state_snapshot(&p), before);
        assert!(store
            .persist_obligation_amount_with_audit(&target(0), [0xe2; 16], 1234)
            .is_err());
        assert_eq!(state_snapshot(&p), before);
    }
    let (_d, p) = setup();
    let pending = seed(&p, 0, 2);
    let mut store = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
    let mut ids = FixedIds::new([0xe0; 16], [0xe1; 16]);
    supply_obligation_amount(pending, 17, &mut ids, &FixedClock, &mut store).unwrap();
    let before = state_snapshot(&p);
    assert!(store
        .persist_obligation_amount_with_audit(&target(0), [0xe2; 16], 1234)
        .is_err());
    assert_eq!(state_snapshot(&p), before);
    health(&open_database(&p).unwrap());
}
#[test]
fn update_and_audit_faults_rollback_together_and_poison_store() {
    for case in 0..4 {
        let (_d, p) = setup();
        let pending = seed(&p, 0, 2);
        let c = open_database(&p).unwrap();
        let trigger=match case{0=>Some("CREATE TRIGGER main.rec010_fail BEFORE UPDATE ON recurrence_obligations BEGIN SELECT RAISE(ABORT,'PRIVATE_SQL_MARKER'); END"),1=>Some("CREATE TRIGGER main.rec010_fail BEFORE UPDATE ON recurrence_obligations BEGIN SELECT RAISE(IGNORE); END"),2=>Some("CREATE TRIGGER main.rec010_fail BEFORE INSERT ON audit_event WHEN NEW.operation='recurrence_amount_supply' BEGIN SELECT RAISE(ABORT,'PRIVATE_SQL_MARKER'); END"),_=>None};
        if let Some(sql) = trigger {
            c.execute_batch(sql).unwrap();
        }
        let before = state(&c);
        drop(c);
        let mut store = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
        let event = if case == 3 { 0x63 } else { 0xe0 };
        let mut ids = FixedIds::new([event; 16], [0xe1; 16]);
        let e = supply_obligation_amount(pending, 7, &mut ids, &FixedClock, &mut store)
            .err()
            .unwrap();
        opaque(&e);
        assert_eq!(state_snapshot(&p), before);
        let c = open_database(&p).unwrap();
        health(&c);
        if trigger.is_some() {
            c.execute_batch("DROP TRIGGER main.rec010_fail").unwrap();
        }
        drop(c);
        assert!(store
            .persist_obligation_amount_with_audit(&target(0), [0xe2; 16], 1234)
            .is_err());
        assert_eq!(state_snapshot(&p), before);
        let mut fresh = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
        assert!(fresh
            .persist_obligation_amount_with_audit(&target(0), [0xe3; 16], 1234)
            .is_ok());
        health(&open_database(&p).unwrap());
    }
}
