use super::{Context, DomainError, FixedRecurrence};

fn rule(name: &str, amount_cents: i64, due_day: u8, context: u8) -> FixedRecurrence {
    FixedRecurrence {
        id: [0x39; 16],
        name: name.into(),
        amount_cents,
        context: if context == 0 {
            Context::Home
        } else {
            Context::Factory
        },
        due_day,
    }
}

#[test]
fn accepts_every_month_day_positive_amount_bounds_and_both_contexts() {
    for day in 1..=31 {
        for amount in [1, i64::MAX] {
            for ctx in 0..2 {
                let recurrence = rule("  Água mensal  ", amount, day, ctx);
                assert_eq!(recurrence.validate(), Ok(()));
                assert_eq!(recurrence.id, [0x39; 16]);
                assert_eq!(recurrence.name, "  Água mensal  ");
                assert_eq!(recurrence.amount_cents, amount);
                assert_eq!(recurrence.due_day, day);
                assert!(matches!(
                    (&recurrence.context, ctx),
                    (Context::Home, 0) | (Context::Factory, 1)
                ));
            }
        }
    }
}

#[test]
fn rejects_blank_names_without_rewriting_input() {
    for name in ["", " ", "\t\r\n", "\u{00a0}\u{2003}"] {
        for ctx in 0..2 {
            let recurrence = rule(name, 1, 1, ctx);
            assert_eq!(recurrence.validate(), Err(DomainError::InvalidInput));
            assert_eq!(recurrence.name, name);
            assert_eq!(recurrence.amount_cents, 1);
            assert_eq!(recurrence.due_day, 1);
        }
    }
    assert_eq!(rule(" \u{00a0}Água\u{2003} ", 1, 31, 0).validate(), Ok(()));
}

#[test]
fn rejects_zero_negative_and_minimum_amounts() {
    for amount in [0, -1, i64::MIN] {
        for ctx in 0..2 {
            let recurrence = rule("Água", amount, 31, ctx);
            assert_eq!(recurrence.validate(), Err(DomainError::InvalidInput));
            assert_eq!(recurrence.amount_cents, amount);
            assert_eq!(recurrence.due_day, 31);
        }
    }
}

#[test]
fn rejects_days_outside_monthly_configuration_range() {
    for day in [0, 32, u8::MAX] {
        for ctx in 0..2 {
            let recurrence = rule("Água", 1, day, ctx);
            assert_eq!(recurrence.validate(), Err(DomainError::InvalidInput));
            assert_eq!(recurrence.due_day, day);
            assert_eq!(recurrence.amount_cents, 1);
        }
    }
}
