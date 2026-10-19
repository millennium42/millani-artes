use super::{ports::Clock, Context, DomainError, Movement, MovementKind, ObligationState};
use std::cell::Cell;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

struct CountingClock {
    instant: SystemTime,
    calls: Cell<usize>,
}
impl Clock for CountingClock {
    fn now(&self) -> SystemTime {
        self.calls.set(self.calls.get() + 1);
        self.instant
    }
}
fn clock(instant: SystemTime) -> CountingClock {
    CountingClock {
        instant,
        calls: Cell::new(0),
    }
}

#[test]
fn overdue_requires_strictly_later_injected_time_and_preserves_payable() {
    for due_at in [
        UNIX_EPOCH - Duration::from_secs(10),
        UNIX_EPOCH,
        UNIX_EPOCH + Duration::from_secs(10),
    ] {
        for amount in [1, i64::MAX] {
            let state = ObligationState::Payable {
                amount_cents: amount,
            };
            let source = clock(due_at + Duration::from_nanos(100));
            let overdue = state.mark_overdue(due_at, &source).unwrap();
            assert_eq!(source.calls.get(), 1);
            assert_eq!(overdue.validate(), Ok(()));
            assert!(
                matches!(overdue, ObligationState::Overdue {amount_cents} if amount_cents==amount)
            );
            assert!(
                matches!(state, ObligationState::Payable {amount_cents} if amount_cents==amount)
            );
        }
    }
}

#[test]
fn at_or_before_due_time_is_not_overdue() {
    for due_at in [
        UNIX_EPOCH - Duration::from_secs(10),
        UNIX_EPOCH,
        UNIX_EPOCH + Duration::from_secs(10),
    ] {
        for instant in [due_at - Duration::from_nanos(100), due_at] {
            let source = clock(instant);
            let state = ObligationState::Payable { amount_cents: 7 };
            assert!(matches!(
                state.mark_overdue(due_at, &source),
                Err(DomainError::InvalidInput)
            ));
            assert_eq!(source.calls.get(), 1);
            assert!(matches!(
                state,
                ObligationState::Payable { amount_cents: 7 }
            ));
        }
    }
}

#[test]
fn nonpayable_and_invalid_states_fail_before_reading_clock() {
    for state in [
        ObligationState::AwaitingValue,
        ObligationState::Payable { amount_cents: 0 },
        ObligationState::Payable { amount_cents: -1 },
        ObligationState::Payable {
            amount_cents: i64::MIN,
        },
        ObligationState::Overdue { amount_cents: 7 },
        ObligationState::Overdue { amount_cents: 0 },
        ObligationState::Overdue { amount_cents: -1 },
        ObligationState::Overdue {
            amount_cents: i64::MIN,
        },
        ObligationState::Paid {
            amount_cents: 7,
            movement_id: [1; 16],
        },
        ObligationState::Paid {
            amount_cents: 0,
            movement_id: [1; 16],
        },
        ObligationState::Paid {
            amount_cents: 7,
            movement_id: [0; 16],
        },
    ] {
        let source = clock(UNIX_EPOCH + Duration::from_secs(1));
        let before = std::mem::discriminant(&state);
        assert!(matches!(
            state.mark_overdue(UNIX_EPOCH, &source),
            Err(DomainError::InvalidInput)
        ));
        assert_eq!(source.calls.get(), 0);
        assert_eq!(std::mem::discriminant(&state), before);
    }
}

#[test]
fn overdue_keeps_positive_amount_and_uses_existing_payment_validation() {
    for amount_cents in [0, -1, i64::MIN] {
        let state = ObligationState::Overdue { amount_cents };
        assert_eq!(state.validate(), Err(DomainError::InvalidInput));
        let payment = Movement {
            id: [1; 16],
            amount_cents: 7,
            context: Context::Home,
            kind: MovementKind::Exit {
                account_id: [3; 16],
            },
        };
        assert!(matches!(
            state.mark_paid(&payment),
            Err(DomainError::InvalidInput)
        ));
    }
    for amount in [1, i64::MAX] {
        for ctx in 0..2 {
            let mut id = [0; 16];
            id[if ctx == 0 { 0 } else { 15 }] = 1;
            let state = ObligationState::Overdue {
                amount_cents: amount,
            };
            let payment = Movement {
                id,
                amount_cents: amount,
                context: if ctx == 0 {
                    Context::Home
                } else {
                    Context::Factory
                },
                kind: MovementKind::Exit {
                    account_id: [3; 16],
                },
            };
            assert_eq!(state.validate(), Ok(()));
            let paid = state.mark_paid(&payment).unwrap();
            assert_eq!(paid.validate(), Ok(()));
            assert!(
                matches!(paid, ObligationState::Paid {amount_cents, movement_id} if amount_cents==amount && movement_id==id)
            );
            assert!(
                matches!(state, ObligationState::Overdue {amount_cents} if amount_cents==amount)
            );
            assert_eq!(payment.id, id);
            assert_eq!(payment.amount_cents, amount);
            assert!(matches!(
                (&payment.context, ctx),
                (Context::Home, 0) | (Context::Factory, 1)
            ));
            assert!(matches!(payment.kind,MovementKind::Exit {account_id} if account_id==[3;16]));
        }
    }
    for (kind, amount_cents, id) in [
        (
            MovementKind::Entry {
                account_id: [3; 16],
            },
            7,
            [1; 16],
        ),
        (
            MovementKind::Transfer {
                source_account_id: [3; 16],
                destination_account_id: [4; 16],
            },
            7,
            [1; 16],
        ),
        (
            MovementKind::Exit {
                account_id: [3; 16],
            },
            0,
            [1; 16],
        ),
        (
            MovementKind::Exit {
                account_id: [3; 16],
            },
            -1,
            [1; 16],
        ),
        (
            MovementKind::Exit {
                account_id: [3; 16],
            },
            i64::MIN,
            [1; 16],
        ),
        (
            MovementKind::Exit {
                account_id: [3; 16],
            },
            6,
            [1; 16],
        ),
        (
            MovementKind::Exit {
                account_id: [3; 16],
            },
            8,
            [1; 16],
        ),
        (
            MovementKind::Exit {
                account_id: [3; 16],
            },
            7,
            [0; 16],
        ),
    ] {
        let state = ObligationState::Overdue { amount_cents: 7 };
        let payment = Movement {
            id,
            amount_cents,
            context: Context::Factory,
            kind,
        };
        assert!(matches!(
            state.mark_paid(&payment),
            Err(DomainError::InvalidInput)
        ));
        assert!(matches!(
            state,
            ObligationState::Overdue { amount_cents: 7 }
        ));
        assert_eq!(payment.id, id);
        assert_eq!(payment.amount_cents, amount_cents);
    }
}
