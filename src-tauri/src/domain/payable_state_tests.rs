use super::{Context, DomainError, FixedRecurrence, ObligationState, VariableRecurrence};
fn fixed(name: &str, amount_cents: i64, day: u8, ctx: u8) -> FixedRecurrence {
    FixedRecurrence {
        id: [0x69; 16],
        name: name.into(),
        amount_cents,
        context: if ctx == 0 {
            Context::Home
        } else {
            Context::Factory
        },
        due_day: day,
    }
}
#[test]
fn valid_fixed_configuration_is_payable_with_exact_positive_cents() {
    for day in 1..=31 {
        for amount in [1, i64::MAX] {
            for ctx in 0..2 {
                let rule = fixed("  Aluguel mensal  ", amount, day, ctx);
                for _ in 0..2 {
                    let state = rule.initial_obligation_state().unwrap();
                    assert_eq!(state.validate(), Ok(()));
                    assert!(
                        matches!(&state,ObligationState::Payable{amount_cents}if *amount_cents==amount)
                    );
                }
                assert_eq!(rule.id, [0x69; 16]);
                assert_eq!(rule.name, "  Aluguel mensal  ");
                assert_eq!(rule.amount_cents, amount);
                assert_eq!(rule.due_day, day);
                assert!(matches!(
                    (&rule.context, ctx),
                    (Context::Home, 0) | (Context::Factory, 1)
                ));
            }
        }
    }
}
#[test]
fn invalid_fixed_configuration_never_becomes_payable() {
    for ctx in 0..2 {
        for name in ["", " ", "\t\r\n", "\u{00a0}\u{2003}"] {
            let rule = fixed(name, 1, 31, ctx);
            assert!(matches!(
                rule.initial_obligation_state(),
                Err(DomainError::InvalidInput)
            ));
            assert_eq!(rule.name, name);
            assert_eq!(rule.amount_cents, 1);
        }
        for amount in [0, -1, i64::MIN] {
            let rule = fixed("Aluguel", amount, 1, ctx);
            assert!(matches!(
                rule.initial_obligation_state(),
                Err(DomainError::InvalidInput)
            ));
            assert_eq!(rule.amount_cents, amount);
        }
        for day in [0, 32, u8::MAX] {
            let rule = fixed("Aluguel", 1, day, ctx);
            assert!(matches!(
                rule.initial_obligation_state(),
                Err(DomainError::InvalidInput)
            ));
            assert_eq!(rule.due_day, day);
        }
    }
}
#[test]
fn directly_constructed_payable_states_require_positive_cents() {
    for amount_cents in [0, -1, i64::MIN] {
        let state = ObligationState::Payable { amount_cents };
        assert_eq!(state.validate(), Err(DomainError::InvalidInput));
        assert!(
            matches!(&state,ObligationState::Payable{amount_cents:value}if *value==amount_cents)
        );
    }
    for amount_cents in [1, i64::MAX] {
        assert_eq!(ObligationState::Payable { amount_cents }.validate(), Ok(()));
    }
}
#[test]
fn awaiting_value_remains_valid_without_becoming_payable() {
    assert_eq!(ObligationState::AwaitingValue.validate(), Ok(()));
    for ctx in 0..2 {
        let rule = VariableRecurrence {
            id: [0x79; 16],
            name: "Energia".into(),
            context: if ctx == 0 {
                Context::Home
            } else {
                Context::Factory
            },
            due_day: 31,
        };
        let state = rule.initial_obligation_state().unwrap();
        assert!(matches!(&state, ObligationState::AwaitingValue));
        assert_eq!(state.validate(), Ok(()));
        assert_eq!(rule.id, [0x79; 16]);
        assert_eq!(rule.name, "Energia");
        assert_eq!(rule.due_day, 31);
    }
}
