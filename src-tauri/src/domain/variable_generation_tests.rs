use super::{Context, DomainError, FixedRecurrence, ObligationState, VariableRecurrence};
fn rule(day: u8, ctx: u8) -> VariableRecurrence {
    VariableRecurrence {
        id: [3; 16],
        name: "  Energia\0  ".into(),
        context: if ctx == 0 {
            Context::Home
        } else {
            Context::Factory
        },
        due_day: day,
    }
}
#[test]
fn variable_generation_reuses_real_calendar_and_preserves_empty_value_and_metadata() {
    for ctx in 0..2 {
        for day in 1..=31 {
            for period in [
                (1, 1),
                (1900, 2),
                (2000, 2),
                (2024, 2),
                (2026, 2),
                (2100, 2),
                (9999, 12),
            ] {
                let recurrence = rule(day, ctx);
                let mut id = [0; 16];
                id[if ctx == 0 { 0 } else { 15 }] = 1;
                let o = recurrence.generate_obligation(id, period).unwrap();
                let fixed = FixedRecurrence {
                    id: [9; 16],
                    name: "calendar reference".into(),
                    amount_cents: 1,
                    context: Context::Home,
                    due_day: day,
                }
                .generate_obligation([8; 16], period)
                .unwrap();
                assert_eq!(o.id, id);
                assert_eq!(o.recurrence_id, recurrence.id);
                assert_eq!((o.period_year, o.period_month), period);
                assert_eq!(o.due_at_ms, fixed.due_at_ms);
                assert!(matches!(o.state, ObligationState::AwaitingValue));
                assert!(matches!(
                    (&o.context, ctx),
                    (Context::Home, 0) | (Context::Factory, 1)
                ));
                assert_eq!(recurrence.name, "  Energia\0  ");
                assert_eq!(recurrence.due_day, day);
                assert!(recurrence.validate().is_ok());
            }
        }
    }
}
#[test]
fn invalid_variable_rules_cannot_generate_obligations() {
    for (name, day) in [
        ("", 1),
        (" \t", 1),
        ("\u{2003}\u{00a0}", 1),
        ("R", 0),
        ("R", 32),
        ("R", u8::MAX),
    ] {
        let mut recurrence = rule(day, 0);
        recurrence.name = name.into();
        assert!(matches!(
            recurrence.generate_obligation([4; 16], (2026, 1)),
            Err(DomainError::InvalidInput)
        ));
    }
}
#[test]
fn invalid_periods_are_rejected_without_changing_variable_rule() {
    let recurrence = rule(31, 1);
    for period in [
        (0, 1),
        (10000, 1),
        (u16::MAX, 1),
        (2026, 0),
        (2026, 13),
        (2026, u8::MAX),
    ] {
        assert!(matches!(
            recurrence.generate_obligation([4; 16], period),
            Err(DomainError::InvalidInput)
        ));
        assert_eq!(recurrence.due_day, 31);
        assert!(matches!(recurrence.context, Context::Factory));
    }
}
