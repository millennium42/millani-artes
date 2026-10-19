use super::{Context, DomainError, Movement, MovementKind, ObligationState};
fn payment(kind: u8, amount_cents: i64, id: [u8; 16], ctx: u8) -> Movement {
    Movement {
        id,
        amount_cents,
        context: if ctx == 0 {
            Context::Home
        } else {
            Context::Factory
        },
        kind: match kind {
            0 => MovementKind::Exit {
                account_id: [0x31; 16],
            },
            1 => MovementKind::Entry {
                account_id: [0x31; 16],
            },
            _ => MovementKind::Transfer {
                source_account_id: [0x31; 16],
                destination_account_id: [0x32; 16],
            },
        },
    }
}
#[test]
fn payable_accepts_only_matching_exit_and_preserves_full_movement_identity() {
    for amount in [1, i64::MAX] {
        for ctx in 0..2 {
            let mut id = [0; 16];
            id[if ctx == 0 { 0 } else { 15 }] = 1;
            let state = ObligationState::Payable {
                amount_cents: amount,
            };
            let movement = payment(0, amount, id, ctx);
            let paid = state.mark_paid(&movement).unwrap();
            assert_eq!(paid.validate(), Ok(()));
            assert!(
                matches!(&paid,ObligationState::Paid{amount_cents,movement_id}if *amount_cents==amount && *movement_id==id)
            );
            assert!(
                matches!(&state,ObligationState::Payable{amount_cents}if *amount_cents==amount)
            );
            assert_eq!(movement.id, id);
            assert_eq!(movement.amount_cents, amount);
            assert!(
                matches!(&movement.kind,MovementKind::Exit{account_id}if *account_id==[0x31;16])
            );
            assert!(matches!(
                (&movement.context, ctx),
                (Context::Home, 0) | (Context::Factory, 1)
            ));
        }
    }
}
#[test]
fn incompatible_or_invalid_movements_cannot_mark_payable_as_paid() {
    for ctx in 0..2 {
        for (kind, amount, id) in [
            (0, 0, [1; 16]),
            (0, -1, [1; 16]),
            (0, i64::MIN, [1; 16]),
            (0, 6, [1; 16]),
            (0, 8, [1; 16]),
            (0, 7, [0; 16]),
            (1, 7, [1; 16]),
            (2, 7, [1; 16]),
        ] {
            let state = ObligationState::Payable { amount_cents: 7 };
            let movement = payment(kind, amount, id, ctx);
            assert!(matches!(
                state.mark_paid(&movement),
                Err(DomainError::InvalidInput)
            ));
            assert!(matches!(
                &state,
                ObligationState::Payable { amount_cents: 7 }
            ));
            assert_eq!(movement.id, id);
            assert_eq!(movement.amount_cents, amount);
        }
    }
}
#[test]
fn only_a_valid_payable_state_can_transition_to_paid() {
    let movement = payment(0, 7, [1; 16], 0);
    for state in [
        ObligationState::AwaitingValue,
        ObligationState::Payable { amount_cents: 0 },
        ObligationState::Payable { amount_cents: -1 },
        ObligationState::Payable {
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
        assert!(matches!(
            state.mark_paid(&movement),
            Err(DomainError::InvalidInput)
        ));
    }
}
#[test]
fn directly_constructed_paid_requires_positive_amount_and_nonzero_movement_id() {
    for amount_cents in [0, -1, i64::MIN] {
        assert_eq!(
            ObligationState::Paid {
                amount_cents,
                movement_id: [1; 16]
            }
            .validate(),
            Err(DomainError::InvalidInput)
        );
    }
    for amount_cents in [1, i64::MAX] {
        assert_eq!(
            ObligationState::Paid {
                amount_cents,
                movement_id: [0; 16]
            }
            .validate(),
            Err(DomainError::InvalidInput)
        );
        let mut id = [0; 16];
        id[15] = 1;
        assert_eq!(
            ObligationState::Paid {
                amount_cents,
                movement_id: id
            }
            .validate(),
            Ok(())
        );
    }
}
