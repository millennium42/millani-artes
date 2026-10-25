use super::{validate_discount, DiscountInput, DomainError};

#[test]
fn sal003_amount_bounds_are_inclusive() {
    for gross in [0, 1, 1000, i64::MAX] {
        for amount in [0, gross / 2, gross] {
            assert_eq!(
                validate_discount(gross, DiscountInput::AmountCents(amount)),
                Ok(amount)
            );
        }
        for amount in [i64::MIN, -1] {
            assert_eq!(
                validate_discount(gross, DiscountInput::AmountCents(amount)),
                Err(DomainError::InvalidInput)
            );
        }
        if let Some(excess) = gross.checked_add(1) {
            assert_eq!(
                validate_discount(gross, DiscountInput::AmountCents(excess)),
                Err(DomainError::InvalidInput)
            );
        }
    }
}

#[test]
fn sal003_negative_gross_is_invalid_in_both_modes() {
    for gross in [i64::MIN, -1] {
        for input in [
            DiscountInput::AmountCents(0),
            DiscountInput::AmountCents(i64::MIN),
            DiscountInput::AmountCents(i64::MAX),
            DiscountInput::PercentageBasisPoints(0),
            DiscountInput::PercentageBasisPoints(10000),
        ] {
            assert_eq!(
                validate_discount(gross, input),
                Err(DomainError::InvalidInput)
            );
        }
    }
}

#[test]
fn sal003_percent_outside_closed_range_is_invalid_even_for_zero_gross() {
    for gross in [0, 5, i64::MAX] {
        for bp in [i64::MIN, -1, 10001, i64::MAX] {
            assert_eq!(
                validate_discount(gross, DiscountInput::PercentageBasisPoints(bp)),
                Err(DomainError::InvalidInput)
            );
        }
    }
}

#[test]
fn sal003_percentage_rounding_matches_independent_decimal_fixtures() {
    for (gross, bp, expected) in [
        (0, 0, 0),
        (0, 10000, 0),
        (5, 1000, 1),
        (5, 999, 0),
        (5, 1001, 1),
        (1, 4999, 0),
        (1, 5000, 1),
        (1, 5001, 1),
        (3, 5000, 2),
        (1, 1, 0),
        (1, 10000, 1),
        (1234, 1050, 130),
        (12345, 1250, 1543),
        (12345, 1050, 1296),
        (9223372036854775807, 10000, 9223372036854775807),
        (9223372036854775807, 5000, 4611686018427387904),
        (9223372036854775807, 1, 922337203685478),
    ] {
        assert_eq!(
            validate_discount(gross, DiscountInput::PercentageBasisPoints(bp)),
            Ok(expected)
        );
    }
}

#[test]
fn sal003_all_basis_points_preserve_bounds_endpoints_and_monotonicity() {
    for gross in [0, 1, 5, 99, 101, 10000, i64::MAX] {
        let mut previous = 0;
        for bp in 0..=10000 {
            let value = validate_discount(gross, DiscountInput::PercentageBasisPoints(bp)).unwrap();
            assert!((0..=gross).contains(&value));
            assert!(value >= previous);
            if bp == 0 {
                assert_eq!(value, 0);
            }
            if bp == 10000 {
                assert_eq!(value, gross);
            }
            previous = value;
        }
    }
}
