use super::entry_persistence::{FixedClock, FixedIds, Snapshot};
use super::TempDirectory;
use millani_artes::application::generate_variable_obligation::{
    generate_variable_obligation, GenerateVariableObligationError, VariableObligationStore,
};
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
fn opaque(e: &GenerateVariableObligationError) {
    assert!(e.source().is_none());
    assert!(!format!("{e:?}/{e}").contains("PRIVATE_SQL_MARKER"));
}

#[test]
fn variable_generation_commits_configuration_obligations_and_exact_audit_without_movements() {
    let (_d, p) = setup();
    let before = state(&open_database(&p).unwrap());
    let mut store = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
    let mut audits = Vec::new();
    let mut expected = Vec::new();
    for ctx in 0..2 {
        let rule = rule(ctx);
        for month in [2, 3] {
            let id = 0x40 + ctx * 2 + month;
            let event = id + 0x20;
            let mut ids = FixedIds::new([id; 16], [event; 16]);
            let o = generate_variable_obligation(
                &rule,
                (2026, month),
                &mut ids,
                &FixedClock,
                &mut store,
            )
            .unwrap();
            assert_eq!(ids.calls, 2);
            let pure = rule.generate_obligation([id; 16], (2026, month)).unwrap();
            assert_eq!(o.due_at_ms, pure.due_at_ms);
            assert!(matches!(o.state, ObligationState::AwaitingValue));
            expected.push(vec![
                Value::Blob(vec![id; 16]),
                Value::Blob(rule.id.to_vec()),
                Value::Integer(2026),
                Value::Integer(month.into()),
                Value::Integer(o.due_at_ms),
                Value::Text("awaiting_value".into()),
                Value::Null,
                Value::Null,
                Value::Text(if ctx == 0 { "home" } else { "factory" }.into()),
            ]);
            audits.push(vec![
                Value::Blob(vec![event; 16]),
                Value::Integer(1234),
                Value::Text("recurrence_variable_generate".into()),
                Value::Text("recurrence_obligation".into()),
                Value::Blob(vec![id; 16]),
            ]);
        }
    }
    drop(store);
    let c = open_database(&p).unwrap();
    health(&c);
    let after = state(&c);
    assert_eq!(after.2, expected);
    assert_eq!(after.1.len(), 2);
    for ctx in 0..2 {
        let rule = rule(ctx);
        assert_eq!(
            after.1[usize::from(ctx)],
            vec![
                Value::Blob(rule.id.to_vec()),
                Value::Text(rule.name),
                Value::Text("variable".into()),
                Value::Null,
                Value::Text(if ctx == 0 { "home" } else { "factory" }.into()),
                Value::Integer(31)
            ]
        );
    }
    assert_eq!(before.0 .0, after.0 .0);
    assert_eq!(before.0 .1, after.0 .1);
    assert_eq!(before.0 .3, after.0 .3);
    assert_eq!(before.3, after.3);
    assert_eq!(after.0 .2.len(), before.0 .2.len() + 4);
    for event in before.0 .2.iter().chain(audits.iter()) {
        assert_eq!(after.0 .2.iter().filter(|r| *r == event).count(), 1);
    }
}

#[test]
fn invalid_direct_payloads_have_no_effect_and_keep_recurrence_adapter_usable() {
    let (_d, p) = setup();
    let before = state(&open_database(&p).unwrap());
    let rule = rule(0);
    let mut store = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
    for case in 0..8 {
        let mut o = rule.generate_obligation([0x41; 16], (2026, 1)).unwrap();
        match case {
            0 => o.recurrence_id = [9; 16],
            1 => o.period_year = 0,
            2 => o.period_month = 13,
            3 => o.due_at_ms += 1,
            4 => o.context = Context::Factory,
            5 => o.state = ObligationState::Payable { amount_cents: 7 },
            6 => o.state = ObligationState::Overdue { amount_cents: 1 },
            _ => {
                o.state = ObligationState::Paid {
                    amount_cents: 1,
                    movement_id: [9; 16],
                }
            }
        };
        let e = store
            .persist_variable_obligation_with_audit(&rule, &o, [0x61; 16], 1234)
            .unwrap_err();
        assert!(e.source().is_none());
        assert!(!format!("{e:?}/{e}").contains(&rule.name));
        assert_eq!(state(&open_database(&p).unwrap()), before);
    }
    let mut ids = FixedIds::new([0x41; 16], [0x61; 16]);
    assert!(
        generate_variable_obligation(&rule, (2026, 1), &mut ids, &FixedClock, &mut store).is_ok()
    );
    health(&open_database(&p).unwrap());
}

#[test]
fn recurrence_generation_faults_rollback_configuration_obligation_audit_and_poison_adapter() {
    for case in 0..5 {
        let (_d, p) = setup();
        let c = open_database(&p).unwrap();
        let before = state(&c);
        let trigger=match case{0=>Some("CREATE TRIGGER main.rec008_fail BEFORE INSERT ON recurrences BEGIN SELECT RAISE(ABORT,'PRIVATE_SQL_MARKER'); END"),1=>Some("CREATE TRIGGER main.rec008_fail BEFORE INSERT ON recurrence_obligations BEGIN SELECT RAISE(ABORT,'PRIVATE_SQL_MARKER'); END"),2=>Some("CREATE TRIGGER main.rec008_fail BEFORE INSERT ON audit_event WHEN NEW.operation='recurrence_variable_generate' BEGIN SELECT RAISE(ABORT,'PRIVATE_SQL_MARKER'); END"),4=>Some("CREATE TRIGGER main.rec008_fail BEFORE INSERT ON recurrence_obligations BEGIN SELECT RAISE(IGNORE); END"),_=>None};
        if let Some(sql) = trigger {
            c.execute_batch(sql).unwrap();
        }
        let event = if case == 3 {
            match &before.0 .2[0][0] {
                Value::Blob(id) => id.clone().try_into().unwrap(),
                _ => panic!("audit id"),
            }
        } else {
            [0x61; 16]
        };
        drop(c);
        let mut store = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
        let mut ids = FixedIds::new([0x41; 16], event);
        let e =
            generate_variable_obligation(&rule(0), (2026, 1), &mut ids, &FixedClock, &mut store)
                .err()
                .unwrap();
        assert!(matches!(e, GenerateVariableObligationError::StoreFailed));
        opaque(&e);
        let c = open_database(&p).unwrap();
        health(&c);
        assert_eq!(state(&c), before);
        if trigger.is_some() {
            c.execute_batch("DROP TRIGGER main.rec008_fail").unwrap();
        }
        drop(c);
        let mut ids = FixedIds::new([0x42; 16], [0x62; 16]);
        assert!(matches!(
            generate_variable_obligation(&rule(0), (2026, 2), &mut ids, &FixedClock, &mut store),
            Err(GenerateVariableObligationError::StoreFailed)
        ));
        assert_eq!(state(&open_database(&p).unwrap()), before);
        let mut fresh = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
        let mut ids = FixedIds::new([0x43; 16], [0x63; 16]);
        assert!(generate_variable_obligation(
            &rule(0),
            (2026, 3),
            &mut ids,
            &FixedClock,
            &mut fresh
        )
        .is_ok());
        health(&open_database(&p).unwrap());
    }
}

#[test]
fn existing_conflicting_recurrence_configuration_is_never_overwritten() {
    for case in 0..4 {
        let (_d, p) = setup();
        let c = open_database(&p).unwrap();
        let rule = rule(0);
        let name = if case == 0 { "different" } else { &rule.name };
        let amount = if case == 1 { Some(7_i64) } else { None };
        let context = if case == 2 { "factory" } else { "home" };
        let day = if case == 3 { 30 } else { 31 };
        let kind = if case == 1 { "fixed" } else { "variable" };
        c.execute(
            "INSERT INTO main.recurrences VALUES(?1,?2,?3,?4,?5,?6)",
            params![rule.id.as_slice(), name, kind, amount, context, day],
        )
        .unwrap();
        let before = state(&c);
        drop(c);
        let mut store = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
        let mut ids = FixedIds::new([0x41; 16], [0x61; 16]);
        assert!(matches!(
            generate_variable_obligation(&rule, (2026, 1), &mut ids, &FixedClock, &mut store),
            Err(GenerateVariableObligationError::StoreFailed)
        ));
        assert_eq!(state(&open_database(&p).unwrap()), before);
        health(&open_database(&p).unwrap());
    }
}

#[test]
fn duplicate_period_or_obligation_identity_cannot_append_a_partial_generation() {
    let (_d, p) = setup();
    let rule = rule(0);
    let mut store = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
    let mut ids = FixedIds::new([0x41; 16], [0x61; 16]);
    generate_variable_obligation(&rule, (2026, 1), &mut ids, &FixedClock, &mut store).unwrap();
    drop(store);
    let before = state(&open_database(&p).unwrap());
    for (id, event, period) in [
        ([0x42; 16], [0x62; 16], (2026, 1)),
        ([0x41; 16], [0x63; 16], (2026, 2)),
        ([0x41; 16], [0x61; 16], (2026, 1)),
    ] {
        let mut store = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
        let mut ids = FixedIds::new(id, event);
        assert!(matches!(
            generate_variable_obligation(&rule, period, &mut ids, &FixedClock, &mut store),
            Err(GenerateVariableObligationError::StoreFailed)
        ));
        assert_eq!(state(&open_database(&p).unwrap()), before);
        health(&open_database(&p).unwrap());
    }
}
