use super::{Context, DomainError, Movement, MovementKind};

fn movement(kind: u8, amount: i64, context: u8, reverse: bool) -> Movement {
    let (source, destination) = if reverse {
        ([0xFF; 16], [0; 16])
    } else {
        ([0; 16], [0xFF; 16])
    };
    Movement {
        id: [37; 16],
        kind: match kind {
            0 => MovementKind::Entry {
                account_id: [71; 16],
            },
            1 => MovementKind::Exit {
                account_id: [71; 16],
            },
            _ => MovementKind::Transfer {
                source_account_id: source,
                destination_account_id: destination,
            },
        },
        amount_cents: amount,
        context: if context == 0 {
            Context::Home
        } else {
            Context::Factory
        },
    }
}

fn metadata(m: &Movement) -> ([u8; 16], i64, u8, u8, [u8; 16], [u8; 16]) {
    let (kind, from, to) = match &m.kind {
        MovementKind::Entry { account_id } => (0, *account_id, [0; 16]),
        MovementKind::Exit { account_id } => (1, *account_id, [0; 16]),
        MovementKind::Transfer {
            source_account_id,
            destination_account_id,
        } => (2, *source_account_id, *destination_account_id),
    };
    (
        m.id,
        m.amount_cents,
        if matches!(m.context, Context::Home) {
            0
        } else {
            1
        },
        kind,
        from,
        to,
    )
}

#[test]
fn entry_and_exit_have_signed_effect_with_extremes_and_preserve_metadata() {
    for kind in 0..2 {
        for amount in [1, i64::MAX] {
            for context in 0..2 {
                let m = movement(kind, amount, context, false);
                let before = metadata(&m);
                assert_eq!(
                    m.result_effect_cents(),
                    Ok(if kind == 0 { amount } else { -amount })
                );
                assert_eq!(metadata(&m), before);
            }
        }
    }
}

#[test]
fn transfers_have_zero_effect_both_contexts_extremes_and_directions() {
    for context in 0..2 {
        for amount in [1, i64::MAX] {
            for reverse in [false, true] {
                let m = movement(2, amount, context, reverse);
                let before = metadata(&m);
                assert_eq!(m.result_effect_cents(), Ok(0));
                assert_eq!(metadata(&m), before);
            }
        }
    }
}

#[test]
fn invalid_amounts_and_equal_endpoints_fail_before_any_negation() {
    for context in 0..2 {
        for kind in 0..3 {
            for amount in [0, -1, i64::MIN] {
                let m = movement(kind, amount, context, false);
                assert_eq!(m.result_effect_cents(), Err(DomainError::InvalidInput));
            }
        }
        for amount in [1, i64::MAX] {
            let mut m = movement(2, amount, context, false);
            m.kind = MovementKind::Transfer {
                source_account_id: [0; 16],
                destination_account_id: [0; 16],
            };
            assert_eq!(m.result_effect_cents(), Err(DomainError::InvalidInput));
        }
    }
}

#[test]
fn adding_or_removing_transfers_does_not_change_result_composition() {
    for context in 0..2 {
        let movements = [
            movement(0, 80, context, false),
            movement(1, 30, context, false),
            movement(2, i64::MAX, context, false),
            movement(2, i64::MAX, context, true),
        ];
        let result = |items: &[Movement]| {
            items
                .iter()
                .map(|m| i128::from(m.result_effect_cents().unwrap()))
                .sum::<i128>()
        };
        assert_eq!(result(&movements[..2]), 50);
        assert_eq!(result(&movements[..3]), 50);
        assert_eq!(result(&movements), 50);
    }
}

#[test]
fn errors_remain_static_and_without_source() {
    let error = movement(1, i64::MIN, 0, false)
        .result_effect_cents()
        .unwrap_err();
    assert_eq!(format!("{error}"), "DOMAIN_INVALID_INPUT");
    assert_eq!(format!("{error:?}"), "InvalidInput");
    assert!(std::error::Error::source(&error).is_none());
}
