use super::{DomainError, ObligationState};
#[test]
fn awaiting_accepts_exact_positive_cents_and_remains_unchanged() {
    for amount in [1, 7, i64::MAX] {
        let before = ObligationState::AwaitingValue;
        let after = before.supply_amount(amount).unwrap();
        assert!(matches!(after,ObligationState::Payable{amount_cents} if amount_cents==amount));
        assert_eq!(after.validate(), Ok(()));
        assert!(matches!(before, ObligationState::AwaitingValue));
    }
}
#[test]
fn nonpositive_amounts_leave_pending_unchanged() {
    for amount in [0, -1, i64::MIN] {
        let before = ObligationState::AwaitingValue;
        assert!(matches!(
            before.supply_amount(amount),
            Err(DomainError::InvalidInput)
        ));
        assert!(matches!(before, ObligationState::AwaitingValue));
    }
}
#[test]
fn existing_and_invalid_financial_states_cannot_receive_a_new_amount() {
    for value in [0, -1, i64::MIN, 1, 7, i64::MAX] {
        for state in [
            ObligationState::Payable {
                amount_cents: value,
            },
            ObligationState::Overdue {
                amount_cents: value,
            },
            ObligationState::Paid {
                amount_cents: value,
                movement_id: [1; 16],
            },
            ObligationState::Paid {
                amount_cents: value,
                movement_id: [0; 16],
            },
        ] {
            let kind = std::mem::discriminant(&state);
            for amount in [0, -1, 1, i64::MAX] {
                assert!(matches!(
                    state.supply_amount(amount),
                    Err(DomainError::InvalidInput)
                ));
                assert_eq!(std::mem::discriminant(&state), kind);
                match &state {
                    ObligationState::Payable { amount_cents }
                    | ObligationState::Overdue { amount_cents }
                    | ObligationState::Paid { amount_cents, .. } => {
                        assert_eq!(*amount_cents, value)
                    }
                    _ => panic!("unexpected state"),
                }
            }
        }
    }
}
