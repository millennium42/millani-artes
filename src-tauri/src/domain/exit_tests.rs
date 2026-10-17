use super::{Context, DomainError, Movement, MovementKind};

fn exit(amount: i64, context: Context) -> Movement {
    Movement {
        id: [0u8; 16],
        kind: MovementKind::Exit {
            account_id: [0u8; 16],
        },
        amount_cents: amount,
        context,
    }
}

#[test]
fn validate_exit_accepted_values_both_contexts() {
    let amounts = [1i64, 123456, i64::MAX];
    for &amount in &amounts {
        for context_index in 0..2 {
            let ctx = if context_index == 0 {
                Context::Home
            } else {
                Context::Factory
            };
            let m = exit(amount, ctx);
            let orig_id = m.id;
            let orig_amount = m.amount_cents;
            let orig_account_id = match &m.kind {
                MovementKind::Exit { account_id } => *account_id,
                _ => panic!("expected exit fixture"),
            };
            let res = m.validate_exit();
            assert!(res.is_ok());
            assert_eq!(m.id, orig_id);
            assert_eq!(m.amount_cents, orig_amount);
            match &m.kind {
                MovementKind::Exit { account_id } => assert_eq!(*account_id, orig_account_id),
                _ => panic!("exit kind changed"),
            }
            match context_index {
                0 => assert!(matches!(m.context, Context::Home)),
                _ => assert!(matches!(m.context, Context::Factory)),
            }
        }
    }
}

#[test]
fn validate_exit_rejected_zero_negative_min_both_contexts() {
    let amounts = [0i64, -1, i64::MIN];
    for &amount in &amounts {
        for context_index in 0..2 {
            let ctx = if context_index == 0 {
                Context::Home
            } else {
                Context::Factory
            };
            let m = exit(amount, ctx);
            let res = m.validate_exit();
            assert!(matches!(res, Err(DomainError::InvalidInput)));
        }
    }
}

#[test]
fn validate_exit_entry_kind_rejected() {
    let m = Movement {
        id: [0u8; 16],
        kind: MovementKind::Entry {
            account_id: [0u8; 16],
        },
        amount_cents: 100,
        context: Context::Home,
    };
    let res = m.validate_exit();
    assert!(matches!(res, Err(DomainError::InvalidInput)));
}

#[test]
fn validate_exit_transfer_kind_rejected() {
    let m = Movement {
        id: [0u8; 16],
        kind: MovementKind::Transfer {
            source_account_id: [0u8; 16],
            destination_account_id: [1u8; 16],
        },
        amount_cents: 100,
        context: Context::Home,
    };
    let res = m.validate_exit();
    assert!(matches!(res, Err(DomainError::InvalidInput)));
}

#[test]
fn validate_exit_error_display_debug_source() {
    let m = exit(0, Context::Home);
    let err = m.validate_exit().unwrap_err();
    assert_eq!(format!("{}", err), "DOMAIN_INVALID_INPUT");
    assert_eq!(format!("{:?}", err), "InvalidInput");
    assert!(std::error::Error::source(&err).is_none());
}
