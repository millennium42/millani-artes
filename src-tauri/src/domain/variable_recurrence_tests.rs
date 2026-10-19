use super::{Context, DomainError, VariableRecurrence};

fn rule(name: &str, due_day: u8, context: u8) -> VariableRecurrence {
    VariableRecurrence {
        id: [0x49; 16],
        name: name.into(),
        context: if context == 0 {
            Context::Home
        } else {
            Context::Factory
        },
        due_day,
    }
}
#[test]
fn accepts_every_month_day_and_both_contexts_without_preset_amount() {
    for day in 1..=31 {
        for ctx in 0..2 {
            let recurrence = rule("  Energia mensal  ", day, ctx);
            assert_eq!(recurrence.validate(), Ok(()));
            assert_eq!(recurrence.id, [0x49; 16]);
            assert_eq!(recurrence.name, "  Energia mensal  ");
            assert_eq!(recurrence.due_day, day);
            assert!(matches!(
                (&recurrence.context, ctx),
                (Context::Home, 0) | (Context::Factory, 1)
            ));
        }
    }
}
#[test]
fn rejects_blank_names_without_rewriting_input() {
    for name in ["", " ", "\t\r\n", "\u{00a0}\u{2003}"] {
        for ctx in 0..2 {
            let recurrence = rule(name, 31, ctx);
            assert_eq!(recurrence.validate(), Err(DomainError::InvalidInput));
            assert_eq!(recurrence.name, name);
            assert_eq!(recurrence.due_day, 31);
        }
    }
    assert_eq!(rule(" \u{00a0}Energia\u{2003} ", 1, 0).validate(), Ok(()));
}
#[test]
fn rejects_days_outside_monthly_configuration_range() {
    for day in [0, 32, u8::MAX] {
        for ctx in 0..2 {
            let recurrence = rule("Energia", day, ctx);
            assert_eq!(recurrence.validate(), Err(DomainError::InvalidInput));
            assert_eq!(recurrence.due_day, day);
            assert_eq!(recurrence.name, "Energia");
        }
    }
}
