use super::{Context, DomainError, Movement, MovementKind};

fn transfer(amount: i64, context: Context) -> Movement {
    Movement {
        id: [0u8; 16],
        kind: MovementKind::Transfer {
            source_account_id: [0u8; 16],
            destination_account_id: [1u8; 16],
        },
        amount_cents: amount,
        context,
    }
}

#[test]
fn validate_transfer_accepted_values_both_contexts() {
    let amounts = [1i64, 123456, i64::MAX];
    for &amount in &amounts {
        for context_index in 0..2 {
            let ctx = if context_index == 0 {
                Context::Home
            } else {
                Context::Factory
            };
            let m = transfer(amount, ctx);
            let orig_id = m.id;
            let orig_amount = m.amount_cents;
            let orig_account_id = match &m.kind {
                MovementKind::Transfer {
                    source_account_id,
                    destination_account_id,
                } => (*source_account_id, *destination_account_id),
                _ => panic!("expected transfer fixture"),
            };
            let res = m.validate_transfer();
            assert!(res.is_ok());
            assert_eq!(m.id, orig_id);
            assert_eq!(m.amount_cents, orig_amount);
            match &m.kind {
                MovementKind::Transfer {
                    source_account_id,
                    destination_account_id,
                } => assert_eq!(
                    (*source_account_id, *destination_account_id),
                    orig_account_id
                ),
                _ => panic!("transfer kind changed"),
            }
            match context_index {
                0 => assert!(matches!(m.context, Context::Home)),
                _ => assert!(matches!(m.context, Context::Factory)),
            }
        }
    }
}

#[test]
fn validate_transfer_rejected_zero_negative_min_both_contexts() {
    let amounts = [0i64, -1, i64::MIN];
    for &amount in &amounts {
        for context_index in 0..2 {
            let ctx = if context_index == 0 {
                Context::Home
            } else {
                Context::Factory
            };
            let m = transfer(amount, ctx);
            let res = m.validate_transfer();
            assert!(matches!(res, Err(DomainError::InvalidInput)));
        }
    }
}

#[test]
fn validate_transfer_entry_kind_rejected() {
    let m = Movement {
        id: [0u8; 16],
        kind: MovementKind::Entry {
            account_id: [0u8; 16],
        },
        amount_cents: 100,
        context: Context::Home,
    };
    let res = m.validate_transfer();
    assert!(matches!(res, Err(DomainError::InvalidInput)));
}

#[test]
fn validate_transfer_exit_kind_rejected() {
    let m = Movement {
        id: [0u8; 16],
        kind: MovementKind::Exit {
            account_id: [0u8; 16],
        },
        amount_cents: 100,
        context: Context::Home,
    };
    let res = m.validate_transfer();
    assert!(matches!(res, Err(DomainError::InvalidInput)));
}

#[test]
fn validate_transfer_error_display_debug_source() {
    let m = transfer(0, Context::Home);
    let err = m.validate_transfer().unwrap_err();
    assert_eq!(format!("{}", err), "DOMAIN_INVALID_INPUT");
    assert_eq!(format!("{:?}", err), "InvalidInput");
    assert!(std::error::Error::source(&err).is_none());
}

#[test]
fn validate_transfer_same_account_rejected_both_contexts() {
    for account in [[0u8; 16], [u8::MAX; 16]] {
        for amount in [1, i64::MAX] {
            for context in [Context::Home, Context::Factory] {
                let mut m = transfer(amount, context);
                m.kind = MovementKind::Transfer {
                    source_account_id: account,
                    destination_account_id: account,
                };
                assert_eq!(m.validate_transfer(), Err(DomainError::InvalidInput));
            }
        }
    }
}

#[test]
fn validate_transfer_distinct_last_byte_preserved_both_directions() {
    let a = [u8::MAX; 16];
    let mut b = a;
    b[15] = 0;
    for (source, destination) in [(a, b), (b, a)] {
        for amount in [1, i64::MAX] {
            for context_index in 0..2 {
                let context = if context_index == 0 {
                    Context::Home
                } else {
                    Context::Factory
                };
                let m = Movement {
                    id: [37; 16],
                    kind: MovementKind::Transfer {
                        source_account_id: source,
                        destination_account_id: destination,
                    },
                    amount_cents: amount,
                    context,
                };
                assert_eq!(m.validate_transfer(), Ok(()));
                assert_eq!(m.id, [37; 16]);
                assert_eq!(m.amount_cents, amount);
                match &m.kind {
                    MovementKind::Transfer {
                        source_account_id,
                        destination_account_id,
                    } => {
                        assert_eq!(*source_account_id, source);
                        assert_eq!(*destination_account_id, destination);
                    }
                    _ => panic!("transfer kind changed"),
                }
                assert!(matches!(
                    (&m.context, context_index),
                    (Context::Home, 0) | (Context::Factory, 1)
                ));
            }
        }
    }
}
