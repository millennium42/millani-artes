use super::{Context, DomainError, FixedRecurrence, ObligationState};
fn rule(amount_cents: i64, due_day: u8, ctx: u8) -> FixedRecurrence {
    FixedRecurrence {
        id: [3; 16],
        name: "  Café\0  ".into(),
        amount_cents,
        context: if ctx == 0 {
            Context::Home
        } else {
            Context::Factory
        },
        due_day,
    }
}
#[test]
fn fixed_generation_matches_independent_gregorian_calendar_fixtures() {
    for (year, month, day, expected) in [
        (1970_u16, 1_u8, 1_u8, 97199999_i64),
        (1970_u16, 1_u8, 31_u8, 2689199999_i64),
        (2026_u16, 2_u8, 31_u8, 1772333999999_i64),
        (2024_u16, 2_u8, 31_u8, 1709261999999_i64),
        (2000_u16, 2_u8, 31_u8, 951879599999_i64),
        (1900_u16, 2_u8, 31_u8, -2203880400001_i64),
        (2100_u16, 2_u8, 31_u8, 4107553199999_i64),
        (2026_u16, 4_u8, 31_u8, 1777604399999_i64),
        (2026_u16, 12_u8, 31_u8, 1798772399999_i64),
        (1_u16, 1_u8, 1_u8, -62135499600001_i64),
        (9999_u16, 12_u8, 31_u8, 253402311599999_i64),
    ] {
        let generated = rule(7, day, 0)
            .generate_obligation([4; 16], (year, month))
            .unwrap();
        assert_eq!(
            generated.due_at_ms, expected,
            "period {year}/{month} day{day}"
        );
    }
}
#[test]
fn fixed_generation_preserves_metadata_and_positive_cents_for_both_contexts() {
    for amount in [1, i64::MAX] {
        for ctx in 0..2 {
            for day in 1..=31 {
                let recurrence = rule(amount, day, ctx);
                let mut id = [0; 16];
                id[if ctx == 0 { 0 } else { 15 }] = 1;
                let generated = recurrence.generate_obligation(id, (2026, 1)).unwrap();
                assert_eq!(generated.id, id);
                assert_eq!(generated.recurrence_id, recurrence.id);
                assert_eq!((generated.period_year, generated.period_month), (2026, 1));
                assert!(
                    matches!(generated.state,ObligationState::Payable{amount_cents}if amount_cents==amount)
                );
                assert!(matches!(
                    (&generated.context, ctx),
                    (Context::Home, 0) | (Context::Factory, 1)
                ));
                assert_eq!(recurrence.name, "  Café\0  ");
                assert_eq!(recurrence.amount_cents, amount);
                assert_eq!(recurrence.due_day, day);
                assert!(recurrence.validate().is_ok());
            }
        }
    }
}
#[test]
fn invalid_fixed_rules_cannot_generate_obligation() {
    for (name, amount, day) in [
        ("", 7, 1),
        (" \t", 7, 1),
        ("\u{2003}\u{00a0}", 7, 1),
        ("R", 0, 1),
        ("R", -1, 1),
        ("R", i64::MIN, 1),
        ("R", 7, 0),
        ("R", 7, 32),
        ("R", 7, u8::MAX),
    ] {
        let mut recurrence = rule(amount, day, 0);
        recurrence.name = name.into();
        assert!(matches!(
            recurrence.generate_obligation([4; 16], (2026, 1)),
            Err(DomainError::InvalidInput)
        ));
    }
}
#[test]
fn invalid_periods_are_rejected_without_changing_fixed_rule() {
    let recurrence = rule(7, 31, 1);
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
        assert_eq!(recurrence.amount_cents, 7);
        assert_eq!(recurrence.due_day, 31);
        assert!(matches!(recurrence.context, Context::Factory));
    }
}
