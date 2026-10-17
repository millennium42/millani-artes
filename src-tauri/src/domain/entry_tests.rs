use super::{Context, DomainError, Movement, MovementKind};

fn entry(amount: i64, context: Context) -> Movement {
    Movement {
        id: [0u8; 16],
        kind: MovementKind::Entry {
            account_id: [0u8; 16],
        },
        amount_cents: amount,
        context,
    }
}

#[test]
fn validate_entry_accepted_values_both_contexts() {
    let amounts = [1i64, 123456, i64::MAX];
    for &amount in &amounts {
        for context_index in 0..2 {
            let ctx = if context_index == 0 {
                Context::Home
            } else {
                Context::Factory
            };
            let m = entry(amount, ctx);
            let orig_id = m.id;
            let orig_amount = m.amount_cents;
            let orig_account_id = match &m.kind {
                MovementKind::Entry { account_id } => *account_id,
                _ => panic!("expected entry fixture"),
            };
            let res = m.validate_entry();
            assert!(res.is_ok());
            assert_eq!(m.id, orig_id);
            assert_eq!(m.amount_cents, orig_amount);
            match &m.kind {
                MovementKind::Entry { account_id } => assert_eq!(*account_id, orig_account_id),
                _ => panic!("entry kind changed"),
            }
            match context_index {
                0 => assert!(matches!(m.context, Context::Home)),
                _ => assert!(matches!(m.context, Context::Factory)),
            }
        }
    }
}

#[test]
fn validate_entry_rejected_zero_negative_min_both_contexts() {
    let amounts = [0i64, -1, i64::MIN];
    for &amount in &amounts {
        for context_index in 0..2 {
            let ctx = if context_index == 0 {
                Context::Home
            } else {
                Context::Factory
            };
            let m = entry(amount, ctx);
            let res = m.validate_entry();
            assert!(matches!(res, Err(DomainError::InvalidInput)));
        }
    }
}

#[test]
fn validate_entry_exit_kind_rejected() {
    let m = Movement {
        id: [0u8; 16],
        kind: MovementKind::Exit {
            account_id: [0u8; 16],
        },
        amount_cents: 100,
        context: Context::Home,
    };
    let res = m.validate_entry();
    assert!(matches!(res, Err(DomainError::InvalidInput)));
}

#[test]
fn validate_entry_transfer_kind_rejected() {
    let m = Movement {
        id: [0u8; 16],
        kind: MovementKind::Transfer {
            source_account_id: [0u8; 16],
            destination_account_id: [1u8; 16],
        },
        amount_cents: 100,
        context: Context::Home,
    };
    let res = m.validate_entry();
    assert!(matches!(res, Err(DomainError::InvalidInput)));
}

#[test]
fn validate_entry_error_display_debug_source() {
    let m = entry(0, Context::Home);
    let err = m.validate_entry().unwrap_err();
    assert_eq!(format!("{}", err), "DOMAIN_INVALID_INPUT");
    assert_eq!(format!("{:?}", err), "InvalidInput");
    assert!(std::error::Error::source(&err).is_none());
}
