use super::{Context, DomainError, ObligationState, VariableRecurrence};
fn rule(name: &str, day: u8, ctx: u8) -> VariableRecurrence {
    VariableRecurrence {
        id: [0x59; 16],
        name: name.into(),
        context: if ctx == 0 {
            Context::Home
        } else {
            Context::Factory
        },
        due_day: day,
    }
}
#[test]
fn valid_variable_configuration_initially_awaits_value_in_both_contexts() {
    for day in 1..=31 {
        for ctx in 0..2 {
            let recurrence = rule("  Energia variável  ", day, ctx);
            for _ in 0..2 {
                assert!(matches!(
                    recurrence.initial_obligation_state().unwrap(),
                    ObligationState::AwaitingValue
                ));
            }
            assert_eq!(recurrence.id, [0x59; 16]);
            assert_eq!(recurrence.name, "  Energia variável  ");
            assert_eq!(recurrence.due_day, day);
            assert!(matches!(
                (&recurrence.context, ctx),
                (Context::Home, 0) | (Context::Factory, 1)
            ));
        }
    }
}
#[test]
fn invalid_names_cannot_produce_an_initial_obligation_state() {
    for name in ["", " ", "\t\r\n", "\u{00a0}\u{2003}"] {
        for ctx in 0..2 {
            let recurrence = rule(name, 31, ctx);
            assert!(matches!(
                recurrence.initial_obligation_state(),
                Err(DomainError::InvalidInput)
            ));
            assert_eq!(recurrence.name, name);
            assert_eq!(recurrence.due_day, 31);
        }
    }
    assert!(matches!(
        rule(" \u{00a0}Energia\u{2003} ", 1, 0)
            .initial_obligation_state()
            .unwrap(),
        ObligationState::AwaitingValue
    ));
}
#[test]
fn invalid_days_cannot_produce_an_initial_obligation_state() {
    for day in [0, 32, u8::MAX] {
        for ctx in 0..2 {
            let recurrence = rule("Energia", day, ctx);
            assert!(matches!(
                recurrence.initial_obligation_state(),
                Err(DomainError::InvalidInput)
            ));
            assert_eq!(recurrence.name, "Energia");
            assert_eq!(recurrence.due_day, day);
        }
    }
}
