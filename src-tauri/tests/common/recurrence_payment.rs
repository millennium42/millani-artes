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
use millani_artes::application::create_exit::create_exit;
use millani_artes::application::generate_fixed_obligation::generate_fixed_obligation;
use millani_artes::application::mark_obligation_overdue::mark_obligation_overdue;
use millani_artes::application::pay_recurrence::{
    pay_recurrence, PayRecurrenceError, RecurrencePaymentStore,
};
use millani_artes::application::supply_obligation_amount::supply_obligation_amount;
use millani_artes::domain::{
    ports::Clock, FixedRecurrence, Movement, MovementKind, RecurrenceObligation,
};
use millani_artes::infrastructure::movements::AdapterSqliteMovementStore;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
struct At(SystemTime);
impl Clock for At {
    fn now(&self) -> SystemTime {
        self.0
    }
}
fn later(o: &RecurrenceObligation, ms: u64) -> At {
    At(UNIX_EPOCH + Duration::from_millis(o.due_at_ms as u64) + Duration::from_millis(ms))
}
fn seed(
    p: &std::path::Path,
    kind: u8,
    overdue: u8,
    ctx: u8,
    month: u8,
    amount: i64,
) -> RecurrenceObligation {
    let mut store = AdapterSqliteRecurrenceStore::new(open_database(p).unwrap());
    let r = rule(ctx);
    let mut ids = FixedIds::new([0x41 + month; 16], [0x61 + month; 16]);
    let o = if kind == 0 {
        generate_fixed_obligation(
            &FixedRecurrence {
                id: r.id,
                name: r.name,
                context: r.context,
                due_day: r.due_day,
                amount_cents: amount,
            },
            (2026, month),
            &mut ids,
            &FixedClock,
            &mut store,
        )
        .unwrap()
    } else {
        let o = generate_variable_obligation(&r, (2026, month), &mut ids, &FixedClock, &mut store)
            .unwrap();
        let mut audit = FixedIds::new([0x81 + month; 16], [0x88; 16]);
        supply_obligation_amount(o, amount, &mut audit, &FixedClock, &mut store).unwrap()
    };
    if overdue == 0 {
        o
    } else {
        let at = later(&o, 1);
        let mut audit = FixedIds::new([0x91 + month; 16], [0x98; 16]);
        mark_obligation_overdue(o, &mut audit, &at, &mut store).unwrap()
    }
}
fn original(ctx: u8) -> RecurrenceObligation {
    let mut o = rule(ctx)
        .generate_obligation([0x43; 16], (2026, 2))
        .unwrap();
    o.state = ObligationState::Payable { amount_cents: 7 };
    o
}
fn paid(ctx: u8, amount: i64, id: [u8; 16]) -> RecurrenceObligation {
    let mut o = original(ctx);
    o.state = ObligationState::Paid {
        amount_cents: amount,
        movement_id: id,
    };
    o
}
fn payment(account: [u8; 16], ctx: u8, amount: i64, id: [u8; 16]) -> Movement {
    Movement {
        id,
        kind: MovementKind::Exit {
            account_id: account,
        },
        amount_cents: amount,
        context: if ctx == 0 {
            Context::Home
        } else {
            Context::Factory
        },
    }
}
fn snapshot(p: &std::path::Path) -> State {
    state(&open_database(p).unwrap())
}
fn account(p: &std::path::Path) -> [u8; 16] {
    let v: Vec<u8> = open_database(p)
        .unwrap()
        .query_row("SELECT id FROM main.accounts LIMIT 1", [], |r| r.get(0))
        .unwrap();
    v.try_into().unwrap()
}
fn existing_exit(p: &std::path::Path, id: [u8; 16]) {
    let mut store = AdapterSqliteMovementStore::new(open_database(p).unwrap());
    let mut ids = FixedIds::new(id, [0xd1; 16]);
    create_exit(
        account(p),
        19,
        Context::Factory,
        &mut ids,
        &FixedClock,
        &mut store,
    )
    .unwrap();
}
fn opaque(e: &PayRecurrenceError) {
    assert!(e.source().is_none());
    assert!(!format!("{e:?}/{e}").contains("PRIVATE_SQL_MARKER"));
}
#[test]
fn payment_commits_one_exact_exit_paid_link_and_audit_for_both_rules_states_contexts_and_bounds() {
    for kind in 0..2 {
        for overdue in 0..2 {
            for ctx in 0..2 {
                for amount in [1, i64::MAX] {
                    let (_d, p) = setup();
                    let o = seed(&p, kind, overdue, ctx, 2, amount);
                    let _other = seed(&p, kind, 0, ctx, 3, amount);
                    let due = o.due_at_ms;
                    let at = later(&o, 2);
                    let before = snapshot(&p);
                    let acc = account(&p);
                    let mut store = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
                    let mut ids = FixedIds::new([0xa0; 16], [0xb0; 16]);
                    let result = pay_recurrence(o, acc, &mut ids, &at, &mut store).unwrap();
                    assert_eq!(ids.calls, 2);
                    assert_eq!(
                        (
                            result.id,
                            result.recurrence_id,
                            result.period_year,
                            result.period_month,
                            result.due_at_ms
                        ),
                        ([0x43; 16], rule(ctx).id, 2026, 2, due)
                    );
                    assert!(
                        matches!(result.state,ObligationState::Paid{amount_cents,movement_id}if amount_cents==amount&&movement_id==[0xa0;16])
                    );
                    assert!(matches!(
                        (result.context, ctx),
                        (Context::Home, 0) | (Context::Factory, 1)
                    ));
                    drop(store);
                    let c = open_database(&p).unwrap();
                    health(&c);
                    let after = state(&c);
                    let mut expected = before.2.clone();
                    expected[0][5] = Value::Text("paid".into());
                    expected[0][7] = Value::Blob(vec![0xa0; 16]);
                    assert_eq!(after.2, expected);
                    assert_eq!(after.1, before.1);
                    assert_eq!(after.0 .0, before.0 .0);
                    assert_eq!(after.0 .3, before.0 .3);
                    assert_eq!(after.3, before.3);
                    assert_eq!(after.0 .1.len(), before.0 .1.len() + 1);
                    assert_eq!(rows(&c,"SELECT id,kind,amount_cents,context,account_id,source_account_id,destination_account_id,category_id FROM main.movements ORDER BY id"),vec![vec![Value::Blob(vec![0xa0;16]),Value::Text("exit".into()),Value::Integer(amount),Value::Text(if ctx==0{"home"}else{"factory"}.into()),Value::Blob(acc.to_vec()),Value::Null,Value::Null,Value::Null]]);
                    assert_eq!(after.0 .2.len(), before.0 .2.len() + 1);
                    for event in &before.0 .2 {
                        assert_eq!(after.0 .2.iter().filter(|r| *r == event).count(), 1);
                    }
                    assert_eq!(
                        after
                            .0
                             .2
                            .iter()
                            .filter(|r| r[0] == Value::Blob(vec![0xb0; 16]))
                            .collect::<Vec<_>>(),
                        vec![&vec![
                            Value::Blob(vec![0xb0; 16]),
                            Value::Integer(due + 2),
                            Value::Text("recurrence_pay".into()),
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
fn invalid_direct_pairs_preserve_data_and_adapter_usability_before_any_sql() {
    let (_d, p) = setup();
    let _o = seed(&p, 1, 0, 0, 2, 7);
    let acc = account(&p);
    let before = snapshot(&p);
    let mut store = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
    for case in 0..12 {
        let mut o = paid(0, 7, [0xa0; 16]);
        let mut m = payment(acc, 0, 7, [0xa0; 16]);
        match case {
            0 => o.state = ObligationState::AwaitingValue,
            1 => o.state = ObligationState::Payable { amount_cents: 7 },
            2 => o.state = ObligationState::Overdue { amount_cents: 7 },
            3 => {
                o.state = ObligationState::Paid {
                    amount_cents: 0,
                    movement_id: [0xa0; 16],
                }
            }
            4 => {
                o.state = ObligationState::Paid {
                    amount_cents: -1,
                    movement_id: [0xa0; 16],
                }
            }
            5 => {
                o.state = ObligationState::Paid {
                    amount_cents: i64::MIN,
                    movement_id: [0xa0; 16],
                }
            }
            6 => {
                o.state = ObligationState::Paid {
                    amount_cents: 7,
                    movement_id: [0; 16],
                }
            }
            7 => m.kind = MovementKind::Entry { account_id: acc },
            8 => {
                m.kind = MovementKind::Transfer {
                    source_account_id: acc,
                    destination_account_id: [9; 16],
                }
            }
            9 => m.amount_cents = 6,
            10 => m.id = [0xa1; 16],
            _ => m.context = Context::Factory,
        };
        let e = store
            .persist_recurrence_payment_with_audit(&o, &m, [0xb0; 16], 1234)
            .unwrap_err();
        assert!(e.source().is_none());
        assert!(!format!("{e:?}/{e}").contains(&rule(0).name));
        assert_eq!(snapshot(&p), before);
    }
    assert!(store
        .persist_recurrence_payment_with_audit(
            &paid(0, 7, [0xa0; 16]),
            &payment(acc, 0, 7, [0xa0; 16]),
            [0xb0; 16],
            1234
        )
        .is_ok());
    health(&open_database(&p).unwrap());
}
#[test]
fn stale_or_adulterated_obligation_snapshots_never_leave_an_inserted_exit_or_audit() {
    for case in 0..10 {
        let (_d, p) = setup();
        let _o = seed(&p, 1, 0, 0, 2, 7);
        let acc = account(&p);
        let mut o = paid(0, 7, [0xa0; 16]);
        let mut m = payment(acc, 0, 7, [0xa0; 16]);
        match case {
            0 => o.id = [9; 16],
            1 => o.recurrence_id = [9; 16],
            2 => o.period_year = 2025,
            3 => o.period_month = 3,
            4 => o.due_at_ms += 1,
            5 => {
                o.context = Context::Factory;
                m.context = Context::Factory;
            }
            6 => {
                o.state = ObligationState::Paid {
                    amount_cents: 8,
                    movement_id: [0xa0; 16],
                };
                m.amount_cents = 8;
            }
            7 => {
                open_database(&p)
                    .unwrap()
                    .execute("DELETE FROM main.recurrence_obligations", [])
                    .unwrap();
            }
            8 => {
                open_database(&p).unwrap().execute("UPDATE main.recurrence_obligations SET state='awaiting_value',amount_cents=NULL",[]).unwrap();
            }
            _ => {
                existing_exit(&p, [0xd0; 16]);
                open_database(&p)
                    .unwrap()
                    .execute(
                        "UPDATE main.recurrence_obligations SET state='paid',movement_id=?1",
                        params![[0xd0u8; 16].as_slice()],
                    )
                    .unwrap();
            }
        }
        let before = snapshot(&p);
        let mut store = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
        assert!(store
            .persist_recurrence_payment_with_audit(&o, &m, [0xb0; 16], 1234)
            .is_err());
        assert_eq!(snapshot(&p), before);
        assert!(store
            .persist_recurrence_payment_with_audit(
                &paid(0, 7, [0xa1; 16]),
                &payment(acc, 0, 7, [0xa1; 16]),
                [0xb1; 16],
                1234
            )
            .is_err());
        assert_eq!(snapshot(&p), before);
        health(&open_database(&p).unwrap());
        if case < 7 {
            let mut fresh = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
            assert!(fresh
                .persist_recurrence_payment_with_audit(
                    &paid(0, 7, [0xa2; 16]),
                    &payment(acc, 0, 7, [0xa2; 16]),
                    [0xb3; 16],
                    1234
                )
                .is_ok());
        }
    }
}
#[test]
fn movement_state_and_audit_abort_or_ignore_faults_rollback_every_payment_write() {
    for case in 0..7 {
        let (_d, p) = setup();
        let pending = seed(&p, 1, 0, 0, 2, 7);
        let acc = account(&p);
        let trigger=match case{0=>Some("CREATE TRIGGER main.rec012_fail BEFORE INSERT ON movements BEGIN SELECT RAISE(ABORT,'PRIVATE_SQL_MARKER'); END"),1=>Some("CREATE TRIGGER main.rec012_fail BEFORE INSERT ON movements BEGIN SELECT RAISE(IGNORE); END"),2=>Some("CREATE TRIGGER main.rec012_fail BEFORE UPDATE ON recurrence_obligations BEGIN SELECT RAISE(ABORT,'PRIVATE_SQL_MARKER'); END"),3=>Some("CREATE TRIGGER main.rec012_fail BEFORE UPDATE ON recurrence_obligations BEGIN SELECT RAISE(IGNORE); END"),4=>Some("CREATE TRIGGER main.rec012_fail BEFORE INSERT ON audit_event WHEN NEW.operation='recurrence_pay' BEGIN SELECT RAISE(ABORT,'PRIVATE_SQL_MARKER'); END"),5=>Some("CREATE TRIGGER main.rec012_fail BEFORE INSERT ON audit_event WHEN NEW.operation='recurrence_pay' BEGIN SELECT RAISE(IGNORE); END"),_=>None};
        let c = open_database(&p).unwrap();
        if let Some(sql) = trigger {
            c.execute_batch(sql).unwrap();
        }
        let before = state(&c);
        drop(c);
        let mut store = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
        let mut ids = FixedIds::new([0xa0; 16], if case == 6 { [0x63; 16] } else { [0xb0; 16] });
        let e = pay_recurrence(pending, acc, &mut ids, &FixedClock, &mut store)
            .err()
            .unwrap();
        opaque(&e);
        assert_eq!(snapshot(&p), before);
        let c = open_database(&p).unwrap();
        health(&c);
        if trigger.is_some() {
            c.execute_batch("DROP TRIGGER main.rec012_fail").unwrap();
        }
        drop(c);
        assert!(store
            .persist_recurrence_payment_with_audit(
                &paid(0, 7, [0xa1; 16]),
                &payment(acc, 0, 7, [0xa1; 16]),
                [0xb1; 16],
                1234
            )
            .is_err());
        assert_eq!(snapshot(&p), before);
        let mut fresh = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
        assert!(fresh
            .persist_recurrence_payment_with_audit(
                &paid(0, 7, [0xa2; 16]),
                &payment(acc, 0, 7, [0xa2; 16]),
                [0xb3; 16],
                1234
            )
            .is_ok());
        health(&open_database(&p).unwrap());
    }
}
#[test]
fn missing_account_existing_movement_and_repeated_payment_do_not_create_extra_exits() {
    for case in 0..2 {
        let (_d, p) = setup();
        let pending = seed(&p, 1, 0, 0, 2, 7);
        let acc = account(&p);
        if case == 1 {
            existing_exit(&p, [0xa0; 16]);
        }
        let before = snapshot(&p);
        let mut store = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
        let mut ids = FixedIds::new([0xa0; 16], [0xb0; 16]);
        let e = pay_recurrence(
            pending,
            if case == 0 { [0xfe; 16] } else { acc },
            &mut ids,
            &FixedClock,
            &mut store,
        )
        .err()
        .unwrap();
        opaque(&e);
        assert_eq!(snapshot(&p), before);
        assert!(store
            .persist_recurrence_payment_with_audit(
                &paid(0, 7, [0xa1; 16]),
                &payment(acc, 0, 7, [0xa1; 16]),
                [0xb1; 16],
                1234
            )
            .is_err());
        assert_eq!(snapshot(&p), before);
        let mut fresh = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
        assert!(fresh
            .persist_recurrence_payment_with_audit(
                &paid(0, 7, [0xa2; 16]),
                &payment(acc, 0, 7, [0xa2; 16]),
                [0xb3; 16],
                1234
            )
            .is_ok());
        health(&open_database(&p).unwrap());
    }
    let (_d, p) = setup();
    let pending = seed(&p, 0, 0, 0, 2, 7);
    let acc = account(&p);
    let mut store = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
    let mut ids = FixedIds::new([0xa0; 16], [0xb0; 16]);
    let paid_result = pay_recurrence(pending, acc, &mut ids, &FixedClock, &mut store).unwrap();
    let before = snapshot(&p);
    let mut retry = FixedIds::new([0xa1; 16], [0xb1; 16]);
    assert!(matches!(
        pay_recurrence(original(0), acc, &mut retry, &FixedClock, &mut store),
        Err(PayRecurrenceError::StoreFailed)
    ));
    assert_eq!(snapshot(&p), before);
    let mut blocked = FixedIds::new([0xa2; 16], [0xb3; 16]);
    assert!(matches!(
        pay_recurrence(paid_result, acc, &mut blocked, &FixedClock, &mut store),
        Err(PayRecurrenceError::InvalidInput)
    ));
    assert_eq!(blocked.calls, 0);
    assert_eq!(snapshot(&p), before);
    assert_eq!(before.0 .1.len(), 1);
    health(&open_database(&p).unwrap());
}

#[test]
fn rec013_each_paid_month_has_one_exit_and_exact_financial_effect_after_reopen() {
    use millani_artes::application::{
        calculate_context_result::calculate_context_result, list_accounts::list_accounts,
        list_movements::list_movements,
    };
    use millani_artes::infrastructure::accounts::AdapterSqliteAccountStore;

    for kind in 0..2 {
        for overdue in 0..2 {
            for ctx in 0..2 {
                let (_d, p) = setup();
                let obligations = [
                    seed(&p, kind, overdue, ctx, 2, 1000),
                    seed(&p, kind, overdue, ctx, 3, 1000),
                ];
                let selected = account(&p);
                let other_account = [0xe0u8; 16];
                open_database(&p).unwrap().execute(
                    "INSERT INTO main.accounts(id,name,account_type,initial_balance_cents) VALUES(?1,'Outra conta sintética','bank_pix',7654)",
                    params![other_account.as_slice()],
                ).unwrap();
                existing_exit(&p, [0xd0; 16]);
                let before = snapshot(&p);
                let mut expected_obligations = before.2.clone();
                let mut previous = before.clone();
                for (index, obligation) in obligations.into_iter().enumerate() {
                    let movement_id = [0xa0 + index as u8; 16];
                    let audit_id = [0xb0 + index as u8; 16];
                    let obligation_id = obligation.id;
                    let mut store = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
                    let result = pay_recurrence(
                        obligation,
                        selected,
                        &mut FixedIds::new(movement_id, audit_id),
                        &FixedClock,
                        &mut store,
                    )
                    .unwrap();
                    assert!(
                        matches!(result.state, ObligationState::Paid { amount_cents: 1000, movement_id: id } if id == movement_id)
                    );
                    drop(store);
                    let reopened = open_database(&p).unwrap();
                    health(&reopened);
                    let after = state(&reopened);
                    assert_eq!(
                        after.0 .1.len(),
                        before.0 .1.len() + index + 1,
                        "one new exit per paid obligation"
                    );
                    assert_eq!(
                        after.0 .2.len(),
                        before.0 .2.len() + index + 1,
                        "one new audit per paid obligation"
                    );
                    for old in &previous.0 .1 {
                        assert_eq!(
                            after.0 .1.iter().filter(|row| *row == old).count(),
                            1,
                            "previous movement retained"
                        );
                    }
                    for old in &previous.0 .2 {
                        assert_eq!(
                            after.0 .2.iter().filter(|row| *row == old).count(),
                            1,
                            "previous audit retained"
                        );
                    }
                    assert_eq!(after.0 .0, before.0 .0);
                    assert_eq!(after.0 .3, before.0 .3);
                    assert_eq!(after.1, before.1);
                    assert_eq!(after.3, before.3);
                    expected_obligations[index][5] = Value::Text("paid".into());
                    expected_obligations[index][7] = Value::Blob(movement_id.to_vec());
                    assert_eq!(after.2, expected_obligations);
                    let joined: (Vec<u8>, String, i64, String, Vec<u8>, i64) = reopened.query_row(
                        "SELECT m.id,m.kind,m.amount_cents,m.context,m.account_id,o.amount_cents FROM main.recurrence_obligations o JOIN main.movements m ON m.id=o.movement_id WHERE o.id=?1 AND o.state='paid'",
                        params![obligation_id.as_slice()], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?)),
                    ).unwrap();
                    assert_eq!(
                        joined,
                        (
                            movement_id.to_vec(),
                            "exit".into(),
                            1000,
                            if ctx == 0 { "home" } else { "factory" }.into(),
                            selected.to_vec(),
                            1000
                        ),
                        "exact persisted exit and obligation amounts"
                    );
                    let audit: (String, String, Vec<u8>) = reopened.query_row(
                        "SELECT operation,entity,entity_id FROM main.audit_event WHERE event_id=?1",params![audit_id.as_slice()],
                        |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?)),
                    ).unwrap();
                    assert_eq!(
                        audit,
                        (
                            "recurrence_pay".into(),
                            "recurrence_obligation".into(),
                            obligation_id.to_vec()
                        )
                    );
                    let accounts =
                        list_accounts(&AdapterSqliteAccountStore::new(open_database(&p).unwrap()))
                            .unwrap();
                    let reader = AdapterSqliteMovementStore::new(open_database(&p).unwrap());
                    let movements = list_movements(&reader).unwrap();
                    assert_eq!(accounts.len(), 2);
                    assert_eq!(movements.len(), index + 2);
                    let paid_cents = 1000 * (index as i64 + 1);
                    for account in accounts {
                        if account.id == selected {
                            assert_eq!(account.initial_balance_cents, -1234);
                            assert_eq!(account.balance_cents(&movements), Ok(-1253 - paid_cents));
                        } else {
                            assert_eq!(account.id, other_account);
                            assert_eq!(account.initial_balance_cents, 7654);
                            assert_eq!(account.balance_cents(&movements), Ok(7654));
                        }
                    }
                    assert_eq!(
                        calculate_context_result(&Context::Home, &reader).unwrap(),
                        if ctx == 0 { -paid_cents } else { 0 }
                    );
                    assert_eq!(
                        calculate_context_result(&Context::Factory, &reader).unwrap(),
                        -19 - if ctx == 1 { paid_cents } else { 0 }
                    );
                    previous = after;
                }
            }
        }
    }
}

#[test]
fn rec014_payment_failures_rollback_all_state_and_preserve_financial_projections() {
    for kind in 0..2 {
        for overdue in 0..2 {
            for ctx in 0..2 {
                for case in [4, 2, 0, 1, 3, 5, 6, 7] {
                    let (_d, p) = setup();
                    let pending = seed(&p, kind, overdue, ctx, 2, 1000);
                    let _other_month = seed(&p, kind, 0, ctx, 3, 1000);
                    let acc = account(&p);
                    let other_account = [0xe0u8; 16];
                    open_database(&p).unwrap().execute("INSERT INTO main.accounts(id,name,account_type,initial_balance_cents) VALUES(?1,'Outra conta sintética','bank_pix',7654)",params![other_account.as_slice()]).unwrap();
                    existing_exit(&p, [0xd0; 16]);
                    let assert_projection = |paid_cents: i64| {
                        use millani_artes::application::{
                            calculate_context_result::calculate_context_result,
                            list_accounts::list_accounts, list_movements::list_movements,
                        };
                        use millani_artes::infrastructure::accounts::AdapterSqliteAccountStore;
                        let accounts = list_accounts(&AdapterSqliteAccountStore::new(
                            open_database(&p).unwrap(),
                        ))
                        .unwrap();
                        let reader = AdapterSqliteMovementStore::new(open_database(&p).unwrap());
                        let movements = list_movements(&reader).unwrap();
                        assert_eq!(accounts.len(), 2);
                        assert_eq!(movements.len(), 1 + usize::from(paid_cents > 0));
                        for account in accounts {
                            if account.id == acc {
                                assert_eq!(account.initial_balance_cents, -1234);
                                assert_eq!(
                                    account.balance_cents(&movements),
                                    Ok(-1253 - paid_cents)
                                );
                            } else {
                                assert_eq!(account.id, other_account);
                                assert_eq!(account.initial_balance_cents, 7654);
                                assert_eq!(account.balance_cents(&movements), Ok(7654));
                            }
                        }
                        assert_eq!(
                            calculate_context_result(&Context::Home, &reader).unwrap(),
                            if ctx == 0 { -paid_cents } else { 0 }
                        );
                        assert_eq!(
                            calculate_context_result(&Context::Factory, &reader).unwrap(),
                            -19 - if ctx == 1 { paid_cents } else { 0 }
                        );
                    };
                    let trigger=match case{0=>Some("CREATE TRIGGER main.rec014_fail BEFORE INSERT ON movements BEGIN SELECT RAISE(ABORT,'PRIVATE_SQL_MARKER'); END"),1=>Some("CREATE TRIGGER main.rec014_fail BEFORE INSERT ON movements BEGIN SELECT RAISE(IGNORE); END"),2=>Some("CREATE TRIGGER main.rec014_fail BEFORE UPDATE ON recurrence_obligations BEGIN SELECT RAISE(ABORT,'PRIVATE_SQL_MARKER'); END"),3=>Some("CREATE TRIGGER main.rec014_fail BEFORE UPDATE ON recurrence_obligations BEGIN SELECT RAISE(IGNORE); END"),4=>Some("CREATE TRIGGER main.rec014_fail BEFORE INSERT ON audit_event WHEN NEW.operation='recurrence_pay' BEGIN SELECT RAISE(ABORT,'PRIVATE_SQL_MARKER'); END"),5=>Some("CREATE TRIGGER main.rec014_fail BEFORE INSERT ON audit_event WHEN NEW.operation='recurrence_pay' BEGIN SELECT RAISE(IGNORE); END"),7=>Some("CREATE TABLE main.rec014_deferred(account_id BLOB REFERENCES accounts(id) DEFERRABLE INITIALLY DEFERRED) STRICT; CREATE TRIGGER main.rec014_fail AFTER INSERT ON audit_event WHEN NEW.operation='recurrence_pay' BEGIN INSERT INTO rec014_deferred(account_id) VALUES(X'FEFEFEFEFEFEFEFEFEFEFEFEFEFEFEFE'); END"),_=>None};
                    let c = open_database(&p).unwrap();
                    if let Some(sql) = trigger {
                        c.execute_batch(sql).unwrap();
                    }
                    if case == 7 {
                        let tx = c.unchecked_transaction().unwrap();
                        tx.execute(
                            "INSERT INTO main.rec014_deferred(account_id) VALUES(?1)",
                            params![[0xfeu8; 16].as_slice()],
                        )
                        .unwrap();
                        assert_eq!(
                            rows(&tx, "SELECT count(*) FROM main.rec014_deferred"),
                            vec![vec![Value::Integer(1)]]
                        );
                        assert!(tx.commit().is_err(), "deferred FK fails at commit");
                        assert_eq!(
                            rows(&c, "SELECT count(*) FROM main.rec014_deferred"),
                            vec![vec![Value::Integer(0)]]
                        );
                    }
                    let before = state(&c);
                    drop(c);
                    let mut store = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
                    let mut ids =
                        FixedIds::new([0xa0; 16], if case == 6 { [0x63; 16] } else { [0xb0; 16] });
                    let e = pay_recurrence(pending, acc, &mut ids, &FixedClock, &mut store)
                        .err()
                        .unwrap();
                    assert_eq!(e, PayRecurrenceError::StoreFailed);
                    assert_eq!(ids.calls, 2);
                    opaque(&e);
                    assert_eq!(
                        snapshot(&p),
                        before,
                        "rollback preserves whole payment state"
                    );
                    assert_projection(0);
                    let c = open_database(&p).unwrap();
                    health(&c);
                    if trigger.is_some() {
                        c.execute_batch("DROP TRIGGER main.rec014_fail").unwrap();
                    }
                    if case == 7 {
                        assert_eq!(
                            rows(&c, "SELECT count(*) FROM main.rec014_deferred"),
                            vec![vec![Value::Integer(0)]]
                        );
                        c.execute_batch("DROP TABLE main.rec014_deferred").unwrap();
                    }
                    drop(c);
                    assert!(store
                        .persist_recurrence_payment_with_audit(
                            &paid(ctx, 1000, [0xa1; 16]),
                            &payment(acc, ctx, 1000, [0xa1; 16]),
                            [0xb1; 16],
                            1234
                        )
                        .is_err());
                    assert_eq!(
                        snapshot(&p),
                        before,
                        "rollback preserves whole payment state"
                    );
                    let mut fresh = AdapterSqliteRecurrenceStore::new(open_database(&p).unwrap());
                    assert!(fresh
                        .persist_recurrence_payment_with_audit(
                            &paid(ctx, 1000, [0xa2; 16]),
                            &payment(acc, ctx, 1000, [0xa2; 16]),
                            [0xb3; 16],
                            1234
                        )
                        .is_ok());
                    let after = snapshot(&p);
                    let mut expected_obligations = before.2.clone();
                    expected_obligations[0][5] = Value::Text("paid".into());
                    expected_obligations[0][7] = Value::Blob(vec![0xa2; 16]);
                    assert_eq!(after.2, expected_obligations);
                    assert_eq!(after.1, before.1);
                    assert_eq!(after.3, before.3);
                    assert_eq!(after.0 .0, before.0 .0);
                    assert_eq!(after.0 .3, before.0 .3);
                    assert_eq!(after.0 .1.len(), before.0 .1.len() + 1);
                    assert_eq!(after.0 .2.len(), before.0 .2.len() + 1);
                    for old in &before.0 .1 {
                        assert_eq!(after.0 .1.iter().filter(|r| *r == old).count(), 1);
                    }
                    for old in &before.0 .2 {
                        assert_eq!(after.0 .2.iter().filter(|r| *r == old).count(), 1);
                    }
                    assert_projection(1000);
                    health(&open_database(&p).unwrap());
                }
            }
        }
    }
}
