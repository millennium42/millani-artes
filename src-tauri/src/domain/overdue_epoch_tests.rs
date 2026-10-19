use super::{ports::Clock, DomainError, ObligationState};
use std::cell::Cell;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
struct Time {
    at: SystemTime,
    calls: Cell<usize>,
}
impl Clock for Time {
    fn now(&self) -> SystemTime {
        self.calls.set(self.calls.get() + 1);
        self.at
    }
}
fn clock(at: SystemTime) -> Time {
    Time {
        at,
        calls: Cell::new(0),
    }
}
#[test]
fn epoch_deadline_matches_existing_rule_at_before_equal_and_one_windows_tick_after() {
    for due_ms in [-10000i64, 0, 10000] {
        let due = if due_ms < 0 {
            UNIX_EPOCH - Duration::from_millis(due_ms.unsigned_abs())
        } else {
            UNIX_EPOCH + Duration::from_millis(due_ms as u64)
        };
        for amount in [1, i64::MAX] {
            for at in [
                due - Duration::from_nanos(100),
                due,
                due + Duration::from_nanos(100),
            ] {
                let old = clock(at);
                let new = clock(at);
                let state = ObligationState::Payable {
                    amount_cents: amount,
                };
                let a = state.mark_overdue(due, &old);
                let b = state.mark_overdue_ms(due_ms, &new);
                match (a, b) {
                    (
                        Ok(ObligationState::Overdue { amount_cents: x }),
                        Ok(ObligationState::Overdue { amount_cents: y }),
                    ) => assert_eq!((x, y), (amount, amount)),
                    (Err(DomainError::InvalidInput), Err(DomainError::InvalidInput)) => {}
                    _ => panic!("deadline rules differ"),
                };
                assert_eq!((old.calls.get(), new.calls.get()), (1, 1));
                assert!(
                    matches!(state,ObligationState::Payable{amount_cents}if amount_cents==amount)
                );
            }
        }
    }
}
#[test]
fn nonpayable_and_invalid_states_fail_before_observing_clock() {
    for state in [
        ObligationState::AwaitingValue,
        ObligationState::Payable { amount_cents: 0 },
        ObligationState::Payable { amount_cents: -1 },
        ObligationState::Payable {
            amount_cents: i64::MIN,
        },
        ObligationState::Overdue { amount_cents: 1 },
        ObligationState::Overdue { amount_cents: 0 },
        ObligationState::Paid {
            amount_cents: 1,
            movement_id: [1; 16],
        },
        ObligationState::Paid {
            amount_cents: 0,
            movement_id: [1; 16],
        },
        ObligationState::Paid {
            amount_cents: 1,
            movement_id: [0; 16],
        },
    ] {
        let source = clock(UNIX_EPOCH);
        let kind = std::mem::discriminant(&state);
        assert!(matches!(
            state.mark_overdue_ms(i64::MIN, &source),
            Err(DomainError::InvalidInput)
        ));
        assert_eq!(source.calls.get(), 0);
        assert_eq!(std::mem::discriminant(&state), kind);
    }
}
#[test]
fn ancient_future_and_extreme_epoch_deadlines_remain_exact_without_overflow() {
    for amount in [1, i64::MAX] {
        for (due, expired) in [
            (i64::MIN, true),
            (-62135586000000, true),
            (-1, true),
            (0, true),
            (10, false),
            (253402311600000, false),
            (i64::MAX, false),
        ] {
            let source = clock(UNIX_EPOCH + Duration::from_millis(10));
            let state = ObligationState::Payable {
                amount_cents: amount,
            };
            let result = state.mark_overdue_ms(due, &source);
            if expired {
                assert!(
                    matches!(result,Ok(ObligationState::Overdue{amount_cents})if amount_cents==amount)
                );
            } else {
                assert!(matches!(result, Err(DomainError::InvalidInput)));
            }
            assert_eq!(source.calls.get(), 1);
        }
    }
}
