use super::entry_persistence::{FixedClock, FixedIds, Snapshot};
use super::TempDirectory;
use millani_artes::application::generate_variable_obligation::generate_variable_obligation;
use millani_artes::database::{
    execute_versioned_migration, open_database, RECURRENCES_MIGRATION_SQL,
    RECURRENCES_MIGRATION_VERSION,
};
use millani_artes::domain::{Context, ObligationState, VariableRecurrence};
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
use millani_artes::application::generate_fixed_obligation::generate_fixed_obligation;
use millani_artes::application::mark_obligation_overdue::{
    mark_obligation_overdue, MarkObligationOverdueError, OverdueObligationStore,
};
use millani_artes::application::supply_obligation_amount::supply_obligation_amount;
use millani_artes::domain::{ports::Clock, FixedRecurrence, RecurrenceObligation};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
struct At(SystemTime);
impl Clock for At {
    fn now(&self) -> SystemTime {
        self.0
    }
}
fn seed(
    p: &std::path::Path,
    kind: u8,
    ctx: u8,
    year: u16,
    month: u8,
    amount: i64,
) -> RecurrenceObligation {
    let mut store = AdapterSqliteRecurrenceStore::new(open_database(p).unwrap());
    let r = rule(ctx);
    let mut ids = FixedIds::new([0x41 + month; 16], [0x61 + month; 16]);
    if kind == 0 {
        generate_fixed_obligation(
            &FixedRecurrence {
                id: r.id,
                name: r.name,
                context: r.context,
                due_day: r.due_day,
                amount_cents: amount,
            },
            (year, month),
            &mut ids,
            &FixedClock,
            &mut store,
        )
        .unwrap()
    } else {
        let o = generate_variable_obligation(&r, (year, month), &mut ids, &FixedClock, &mut store)
            .unwrap();
        let mut audit = FixedIds::new([0x81 + month; 16], [0x91 + month; 16]);
        supply_obligation_amount(o, amount, &mut audit, &FixedClock, &mut store).unwrap()
    }
}
fn target(ctx: u8) -> RecurrenceObligation {
    let mut o = rule(ctx)
        .generate_obligation([0x43; 16], (2026, 2))
        .unwrap();
    o.state = ObligationState::Overdue { amount_cents: 7 };
    o
}
fn due_instant() -> SystemTime {
    UNIX_EPOCH + Duration::from_millis(target(0).due_at_ms as u64)
}
fn observed() -> SystemTime {
    due_instant() + Duration::from_secs(10)
}
fn snapshot(p: &std::path::Path) -> State {
    state(&open_database(p).unwrap())
}
fn opaque(e: &MarkObligationOverdueError) {
    assert!(e.source().is_none());
    assert!(!format!("{e:?}/{e}").contains("PRIVATE_SQL_MARKER"));
}
#[test]
fn overdue_commits_only_state_and_exact_audit_for_both_kinds_contexts_and_epoch_ranges() {
    for kind in 0..2 {
        for ctx in 0..2 {
            for amount in [1, i64::MAX] {
                for year in [1, 2026] {
                    let (_d, p) = setup();
                    let o = seed(&p, kind, ctx, year, 2, amount);
                    let _other = seed(&p, kind, ctx, year, 3, amount);
                    let due = o.due_at_ms;
                    let instant = if due < 0 {
                        UNIX_EPOCH
                    } else {
                        UNIX_EPOCH + Duration::from_millis(due as u64) + Duration::from_nanos(100)
                    };
                    let before = snapshot(&p);
                    let mut store = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
                    let mut ids = FixedIds::new([0xe0; 16], [0xe1; 16]);
                    let after_o =
                        mark_obligation_overdue(o, &mut ids, &At(instant), &mut store).unwrap();
                    assert_eq!(ids.calls, 1);
                    assert!(
                        matches!(after_o.state,ObligationState::Overdue{amount_cents}if amount_cents==amount)
                    );
                    assert_eq!(
                        (
                            after_o.id,
                            after_o.recurrence_id,
                            after_o.period_year,
                            after_o.period_month,
                            after_o.due_at_ms
                        ),
                        ([0x43; 16], rule(ctx).id, year, 2, due)
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
                    expected[0][5] = Value::Text("overdue".into());
                    assert_eq!(after.2, expected);
                    assert_eq!(after.1, before.1);
                    assert_eq!(after.0 .0, before.0 .0);
                    assert_eq!(after.0 .1, before.0 .1);
                    assert_eq!(after.0 .3, before.0 .3);
                    assert_eq!(after.3, before.3);
                    assert_eq!(after.0 .2.len(), before.0 .2.len() + 1);
                    for event in &before.0 .2 {
                        assert_eq!(after.0 .2.iter().filter(|r| *r == event).count(), 1);
                    }
                    let ms = i64::try_from(instant.duration_since(UNIX_EPOCH).unwrap().as_millis())
                        .unwrap();
                    if due >= 0 {
                        assert_eq!(ms, due);
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
                            Value::Integer(ms),
                            Value::Text("recurrence_overdue".into()),
                            Value::Text("recurrence_obligation".into()),
                            Value::Blob(vec![0x43; 16])
                        ]]
                    );
                }
            }
        }
    }
}
#[test]
fn invalid_state_amount_or_observation_has_no_io_and_preserves_adapter_usability() {
    let (_d, p) = setup();
    let _pending = seed(&p, 1, 0, 2026, 2, 7);
    let before = snapshot(&p);
    let mut store = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
    for state in [
        ObligationState::AwaitingValue,
        ObligationState::Payable { amount_cents: 7 },
        ObligationState::Overdue { amount_cents: 0 },
        ObligationState::Overdue { amount_cents: -1 },
        ObligationState::Overdue {
            amount_cents: i64::MIN,
        },
        ObligationState::Paid {
            amount_cents: 7,
            movement_id: [1; 16],
        },
    ] {
        let mut o = target(0);
        o.state = state;
        let e = store
            .persist_overdue_obligation_with_audit(&o, [0xe0; 16], observed())
            .unwrap_err();
        assert!(e.source().is_none());
        assert!(!format!("{e:?}/{e}").contains(&rule(0).name));
        assert_eq!(snapshot(&p), before);
    }
    for at in [due_instant() - Duration::from_nanos(100), due_instant()] {
        assert!(store
            .persist_overdue_obligation_with_audit(&target(0), [0xe0; 16], at)
            .is_err());
        assert_eq!(snapshot(&p), before);
    }
    let mut ancient = target(0);
    ancient.due_at_ms = i64::MIN;
    assert!(store
        .persist_overdue_obligation_with_audit(
            &ancient,
            [0xe0; 16],
            UNIX_EPOCH - Duration::from_millis(1)
        )
        .is_err());
    assert_eq!(snapshot(&p), before);
    assert!(store
        .persist_overdue_obligation_with_audit(
            &target(0),
            [0xe0; 16],
            due_instant() + Duration::from_nanos(100)
        )
        .is_ok());
    health(&open_database(&p).unwrap());
}
#[test]
fn seven_snapshot_mismatches_roll_back_and_poison_without_overwriting_amount() {
    for case in 0..7 {
        let (_d, p) = setup();
        let _pending = seed(&p, 1, 0, 2026, 2, 7);
        let before = snapshot(&p);
        let mut o = target(0);
        match case {
            0 => o.id = [9; 16],
            1 => o.recurrence_id = [9; 16],
            2 => o.period_year = 2025,
            3 => o.period_month = 3,
            4 => o.due_at_ms += 1,
            5 => o.context = Context::Factory,
            _ => o.state = ObligationState::Overdue { amount_cents: 8 },
        };
        let mut store = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
        let e = store
            .persist_overdue_obligation_with_audit(&o, [0xe0; 16], observed())
            .unwrap_err();
        assert!(e.source().is_none());
        assert_eq!(snapshot(&p), before);
        assert!(store
            .persist_overdue_obligation_with_audit(&target(0), [0xe1; 16], observed())
            .is_err());
        assert_eq!(snapshot(&p), before);
        let mut fresh = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
        assert!(fresh
            .persist_overdue_obligation_with_audit(&target(0), [0xe2; 16], observed())
            .is_ok());
        health(&open_database(&p).unwrap());
    }
}
#[test]
fn pending_paid_overdue_missing_and_replay_cannot_be_marked_again_or_change_money() {
    for case in 0..4 {
        let (_d, p) = setup();
        let pending = seed(&p, 1, 0, 2026, 2, 7);
        let c = open_database(&p).unwrap();
        match case {
            0 => {
                c.execute("UPDATE main.recurrence_obligations SET state='awaiting_value',amount_cents=NULL",[]).unwrap();
            }
            1 => {
                c.execute("UPDATE main.recurrence_obligations SET state='overdue'", [])
                    .unwrap();
            }
            2 => {
                c.execute("INSERT INTO main.movements(id,kind,amount_cents,context,account_id) VALUES (?1,'exit',7,'home',(SELECT id FROM main.accounts LIMIT 1))",params![[0x99u8;16].as_slice()]).unwrap();
                c.execute(
                    "UPDATE main.recurrence_obligations SET state='paid',movement_id=?1",
                    params![[0x99u8; 16].as_slice()],
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
        let e = mark_obligation_overdue(pending, &mut ids, &At(observed()), &mut store)
            .err()
            .unwrap();
        opaque(&e);
        assert_eq!(snapshot(&p), before);
        assert!(store
            .persist_overdue_obligation_with_audit(&target(0), [0xe2; 16], observed())
            .is_err());
        assert_eq!(snapshot(&p), before);
    }
    let (_d, p) = setup();
    let pending = seed(&p, 0, 0, 2026, 2, 7);
    let mut store = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
    let mut ids = FixedIds::new([0xe0; 16], [0xe1; 16]);
    mark_obligation_overdue(pending, &mut ids, &At(observed()), &mut store).unwrap();
    let before = snapshot(&p);
    assert!(store
        .persist_overdue_obligation_with_audit(&target(0), [0xe2; 16], observed())
        .is_err());
    assert_eq!(snapshot(&p), before);
    health(&open_database(&p).unwrap());
}
#[test]
fn state_update_and_audit_faults_roll_back_together_and_poison_adapter() {
    for case in 0..4 {
        let (_d, p) = setup();
        let pending = seed(&p, 1, 0, 2026, 2, 7);
        let c = open_database(&p).unwrap();
        let trigger=match case{0=>Some("CREATE TRIGGER main.rec011_fail BEFORE UPDATE ON recurrence_obligations BEGIN SELECT RAISE(ABORT,'PRIVATE_SQL_MARKER'); END"),1=>Some("CREATE TRIGGER main.rec011_fail BEFORE UPDATE ON recurrence_obligations BEGIN SELECT RAISE(IGNORE); END"),2=>Some("CREATE TRIGGER main.rec011_fail BEFORE INSERT ON audit_event WHEN NEW.operation='recurrence_overdue' BEGIN SELECT RAISE(ABORT,'PRIVATE_SQL_MARKER'); END"),_=>None};
        if let Some(sql) = trigger {
            c.execute_batch(sql).unwrap();
        }
        let before = state(&c);
        drop(c);
        let mut store = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
        let event = if case == 3 { 0x63 } else { 0xe0 };
        let mut ids = FixedIds::new([event; 16], [0xe1; 16]);
        let e = mark_obligation_overdue(pending, &mut ids, &At(observed()), &mut store)
            .err()
            .unwrap();
        opaque(&e);
        assert_eq!(snapshot(&p), before);
        let c = open_database(&p).unwrap();
        health(&c);
        if trigger.is_some() {
            c.execute_batch("DROP TRIGGER main.rec011_fail").unwrap();
        }
        drop(c);
        assert!(store
            .persist_overdue_obligation_with_audit(&target(0), [0xe2; 16], observed())
            .is_err());
        assert_eq!(snapshot(&p), before);
        let mut fresh = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
        assert!(fresh
            .persist_overdue_obligation_with_audit(&target(0), [0xe3; 16], observed())
            .is_ok());
        health(&open_database(&p).unwrap());
    }
}
