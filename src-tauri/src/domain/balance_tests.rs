use super::{Account, AccountType, Context, DomainError, Movement, MovementKind};

fn account(id: u8, opening: i64) -> Account {
    Account {
        id: [id; 16],
        name: "Conta ' sintética".into(),
        account_type: AccountType::Other,
        initial_balance_cents: opening,
    }
}
fn movement(id: u8, kind: u8, amount: i64, from: u8, to: u8, context: u8) -> Movement {
    Movement {
        id: [id; 16],
        amount_cents: amount,
        context: if context == 0 {
            Context::Home
        } else {
            Context::Factory
        },
        kind: match kind {
            0 => MovementKind::Entry {
                account_id: [from; 16],
            },
            1 => MovementKind::Exit {
                account_id: [from; 16],
            },
            _ => MovementKind::Transfer {
                source_account_id: [from; 16],
                destination_account_id: [to; 16],
            },
        },
    }
}
#[test]
fn empty_ledger_preserves_all_opening_balances_and_account_metadata() {
    for opening in [i64::MIN, -1234, 0, i64::MAX] {
        let a = account(1, opening);
        assert_eq!(a.balance_cents(&[]), Ok(opening));
        assert_eq!(a.id, [1; 16]);
        assert_eq!(a.name, "Conta ' sintética");
        assert!(matches!(a.account_type, AccountType::Other));
        assert_eq!(a.initial_balance_cents, opening);
    }
}
#[test]
fn entries_and_exits_include_both_contexts_and_ignore_other_accounts() {
    for context in 0..2 {
        let a = account(1, 10);
        let movements = [
            movement(1, 0, 100, 1, 0, context),
            movement(2, 1, 30, 1, 0, 1 - context),
            movement(3, 0, i64::MAX, 2, 0, context),
            movement(4, 1, i64::MAX, 2, 0, 1 - context),
        ];
        assert_eq!(a.balance_cents(&movements), Ok(80));
        assert_eq!(a.initial_balance_cents, 10);
        assert_eq!(movements[0].amount_cents, 100);
        assert_eq!(movements[1].id, [2; 16]);
        assert!(
            matches!(&movements[0].kind, MovementKind::Entry { account_id } if *account_id == [1;16])
        );
        assert!(matches!(
            (&movements[0].context, context),
            (Context::Home, 0) | (Context::Factory, 1)
        ));
    }
}
#[test]
fn transfer_debits_source_credits_destination_and_conserves_total() {
    let movements = [movement(1, 2, 25, 1, 2, 0), movement(2, 2, 5, 2, 1, 1)];
    assert_eq!(account(1, 100).balance_cents(&movements), Ok(80));
    assert_eq!(account(2, 200).balance_cents(&movements), Ok(220));
    assert_eq!(account(3, 300).balance_cents(&movements), Ok(300));
    for context in 0..2 {
        let movements = [movement(1, 2, i64::MAX, 1, 2, context)];
        let source = account(1, 0).balance_cents(&movements).unwrap();
        let destination = account(2, 0).balance_cents(&movements).unwrap();
        assert_eq!(source, -i64::MAX);
        assert_eq!(destination, i64::MAX);
        assert_eq!(i128::from(source) + i128::from(destination), 0);
    }
}
#[test]
fn final_overflow_is_rejected_without_false_intermediate_overflow() {
    assert_eq!(
        account(1, i64::MAX).balance_cents(&[movement(1, 0, 1, 1, 0, 0)]),
        Err(DomainError::InvalidInput)
    );
    assert_eq!(
        account(1, i64::MIN).balance_cents(&[movement(1, 1, 1, 1, 0, 1)]),
        Err(DomainError::InvalidInput)
    );
    assert_eq!(
        account(1, i64::MIN).balance_cents(&[movement(1, 0, i64::MAX, 1, 0, 0)]),
        Ok(-1)
    );
    assert_eq!(
        account(1, i64::MAX).balance_cents(&[movement(1, 1, i64::MAX, 1, 0, 1)]),
        Ok(0)
    );
    let movements = [
        movement(1, 0, i64::MAX, 1, 0, 0),
        movement(2, 0, i64::MAX, 1, 0, 1),
        movement(3, 1, i64::MAX, 1, 0, 0),
    ];
    assert_eq!(account(1, 0).balance_cents(&movements), Ok(i64::MAX));
    assert_eq!(
        account(1, i64::MAX).balance_cents(&[movement(1, 2, 1, 2, 1, 0)]),
        Err(DomainError::InvalidInput)
    );
    assert_eq!(
        account(1, i64::MIN).balance_cents(&[movement(1, 2, 1, 1, 2, 1)]),
        Err(DomainError::InvalidInput)
    );
}
#[test]
fn invalid_movements_even_unrelated_and_equal_endpoints_are_rejected_opaqely() {
    for context in 0..2 {
        for kind in 0..3 {
            for amount in [0, -1, i64::MIN] {
                for from in [1, 3] {
                    let error = account(1, 0)
                        .balance_cents(&[movement(1, kind, amount, from, 2, context)])
                        .unwrap_err();
                    assert_eq!(error, DomainError::InvalidInput);
                    assert_eq!(format!("{error}"), "DOMAIN_INVALID_INPUT");
                    assert_eq!(format!("{error:?}"), "InvalidInput");
                    assert!(std::error::Error::source(&error).is_none());
                }
            }
        }
        assert_eq!(
            account(1, 0).balance_cents(&[movement(1, 2, 1, 1, 1, context)]),
            Err(DomainError::InvalidInput)
        );
        assert_eq!(
            account(1, 0).balance_cents(&[movement(1, 2, i64::MAX, 3, 3, context)]),
            Err(DomainError::InvalidInput)
        );
    }
}
#[test]
fn current_history_is_replayed_without_retaining_prior_balance() {
    let first = account(1, 100);
    let second = account(2, 200);
    let mut history = [
        movement(1, 0, 50, 1, 0, 0),
        movement(2, 1, 20, 1, 0, 1),
        movement(3, 2, 30, 1, 2, 0),
        movement(4, 2, 7, 2, 1, 1),
    ];
    for (length, expected_first, expected_second) in [
        (4, 107, 223),
        (0, 100, 200),
        (1, 150, 200),
        (2, 130, 200),
        (3, 100, 230),
        (4, 107, 223),
        (1, 150, 200),
    ] {
        assert_eq!(first.balance_cents(&history[..length]), Ok(expected_first));
        assert_eq!(
            second.balance_cents(&history[..length]),
            Ok(expected_second)
        );
    }
    history.reverse();
    assert_eq!(first.balance_cents(&history), Ok(107));
    assert_eq!(second.balance_cents(&history), Ok(223));
    assert_eq!(first.balance_cents(&[]), Ok(100));
    assert_eq!(second.balance_cents(&[]), Ok(200));
    assert_eq!(first.balance_cents(&history), Ok(107));
    assert_eq!(second.balance_cents(&history), Ok(223));
    assert_eq!(first.initial_balance_cents, 100);
    assert_eq!(second.initial_balance_cents, 200);
    assert_eq!(first.id, [1; 16]);
    assert_eq!(second.id, [2; 16]);
    assert!(matches!(history[0].amount_cents, 7));
    assert!(matches!(history[1].amount_cents, 30));
    assert!(matches!(history[2].amount_cents, 20));
    assert!(matches!(history[3].amount_cents, 50));
    assert!(matches!(history[0].context, Context::Factory));
    assert!(matches!(history[1].context, Context::Home));
    assert!(matches!(history[2].context, Context::Factory));
    assert!(matches!(history[3].context, Context::Home));
}
