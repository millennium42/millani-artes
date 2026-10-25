use super::{create_free_item, validate_discount_wide, DiscountInput, DomainError, SaleItem};

fn sal005_check(item: &SaleItem, description: &str, q: i64, price: i64, discount: i64, total: i64) {
    assert_eq!(item.id, [0x77; 16]);
    assert_eq!(item.product_id, None);
    assert_eq!(item.description, description);
    assert_eq!(item.quantity, q);
    assert_eq!(item.unit_price_cents, price);
    assert_eq!(item.discount_cents, discount);
    assert_eq!(item.total_cents, total);
}

#[test]
fn sal005_free_item_preserves_owned_fields_and_no_product_origin() {
    for text in ["Frete", "  Serviço ‘Á’ \"livre\"\r\n\0  ", "e\u{0301}"] {
        let item: SaleItem = create_free_item(
            [0x77; 16],
            text.into(),
            2,
            1500,
            DiscountInput::AmountCents(300),
        )
        .unwrap();
        sal005_check(&item, text, 2, 1500, 300, 2700);
    }
    let item: SaleItem =
        create_free_item([0; 16], "Livre".into(), 1, 0, DiscountInput::AmountCents(0)).unwrap();
    assert_eq!(item.id, [0; 16]);
    assert_eq!(item.product_id, None);
}

#[test]
fn sal005_rejects_blank_description() {
    for text in ["", "  ", "\t\r\n", "\u{00a0}\u{2003}", "\u{3000}"] {
        assert_eq!(
            create_free_item(
                [0x77; 16],
                text.into(),
                1,
                1000,
                DiscountInput::AmountCents(0)
            )
            .err(),
            Some(DomainError::InvalidInput)
        );
    }
}

#[test]
fn sal005_rejects_invalid_quantity_and_negative_price() {
    for (q, price) in [(i64::MIN, 0), (-1, 1000), (0, 1000), (1, i64::MIN), (1, -1)] {
        assert_eq!(
            create_free_item(
                [0x77; 16],
                "Livre".into(),
                q,
                price,
                DiscountInput::AmountCents(0)
            )
            .err(),
            Some(DomainError::InvalidInput)
        );
    }
}

#[test]
fn sal005_accepts_currency_percent_zero_and_full_discounts() {
    for (q, price, input, discount, total) in [
        (2, 1500, DiscountInput::AmountCents(0), 0, 3000),
        (2, 1500, DiscountInput::AmountCents(3000), 3000, 0),
        (2, 1500, DiscountInput::PercentageBasisPoints(0), 0, 3000),
        (
            2,
            1500,
            DiscountInput::PercentageBasisPoints(10000),
            3000,
            0,
        ),
        (1, 5, DiscountInput::PercentageBasisPoints(1000), 1, 4),
        (
            i64::MAX,
            0,
            DiscountInput::PercentageBasisPoints(10000),
            0,
            0,
        ),
    ] {
        let item: SaleItem = create_free_item([0x77; 16], "Livre".into(), q, price, input).unwrap();
        sal005_check(&item, "Livre", q, price, discount, total);
    }
}

#[test]
fn sal005_rejects_invalid_discount_or_unrepresentable_snapshots() {
    for input in [
        DiscountInput::AmountCents(i64::MIN),
        DiscountInput::AmountCents(-1),
        DiscountInput::AmountCents(1001),
        DiscountInput::PercentageBasisPoints(i64::MIN),
        DiscountInput::PercentageBasisPoints(-1),
        DiscountInput::PercentageBasisPoints(10001),
        DiscountInput::PercentageBasisPoints(i64::MAX),
    ] {
        assert_eq!(
            create_free_item([0x77; 16], "Livre".into(), 1, 1000, input).err(),
            Some(DomainError::InvalidInput)
        );
    }
    for (q, price, input) in [
        (2, i64::MAX, DiscountInput::AmountCents(0)),
        (2, i64::MAX, DiscountInput::AmountCents(i64::MAX - 1)),
        (2, i64::MAX, DiscountInput::PercentageBasisPoints(10000)),
        (
            3,
            6148914691236517205,
            DiscountInput::PercentageBasisPoints(5000),
        ),
    ] {
        assert_eq!(
            create_free_item([0x77; 16], "Livre".into(), q, price, input).err(),
            Some(DomainError::InvalidInput)
        );
    }
}

#[test]
fn sal005_accepts_wide_gross_when_both_snapshot_amounts_fit() {
    for (q, price, input, discount, total) in [
        (
            2,
            i64::MAX,
            DiscountInput::AmountCents(i64::MAX),
            i64::MAX,
            i64::MAX,
        ),
        (
            2,
            i64::MAX,
            DiscountInput::PercentageBasisPoints(5000),
            i64::MAX,
            i64::MAX,
        ),
        (
            2,
            4611686018427387904,
            DiscountInput::PercentageBasisPoints(5000),
            4611686018427387904,
            4611686018427387904,
        ),
        (
            i64::MAX,
            2,
            DiscountInput::AmountCents(i64::MAX),
            i64::MAX,
            i64::MAX,
        ),
    ] {
        let item: SaleItem = create_free_item([0x77; 16], "Livre".into(), q, price, input).unwrap();
        sal005_check(&item, "Livre", q, price, discount, total);
    }
}

#[test]
fn sal005_shared_discount_supports_wide_inputs_without_overflow() {
    assert_eq!(
        validate_discount_wide(i128::MAX, DiscountInput::AmountCents(i64::MAX)),
        Ok(i64::MAX)
    );
    assert_eq!(
        validate_discount_wide(i128::MAX, DiscountInput::PercentageBasisPoints(0)),
        Ok(0)
    );
    for bp in [1, 5000, 10000] {
        assert_eq!(
            validate_discount_wide(i128::MAX, DiscountInput::PercentageBasisPoints(bp)),
            Err(DomainError::InvalidInput)
        );
    }
    assert_eq!(
        validate_discount_wide(i128::MIN, DiscountInput::AmountCents(0)),
        Err(DomainError::InvalidInput)
    );
    assert_eq!(
        validate_discount_wide(i128::MAX, DiscountInput::AmountCents(-1)),
        Err(DomainError::InvalidInput)
    );
}

#[test]
fn sal005_wide_rounding_matches_high_precision_decimal_fixtures() {
    let cases: [(i128, i64, Option<i64>); 9] = [
        (18446744073709551614, 5000, Some(9223372036854775807)),
        (9223372036854775808, 5000, Some(4611686018427387904)),
        (18446744073709551614, 4999, Some(9221527362447404852)),
        (9223372036854775808, 1, Some(922337203685478)),
        (9223372036854788152, 1000, Some(922337203685478815)),
        (i128::MAX, 0, Some(0)),
        (i128::MAX, 1, None),
        (i128::MAX, 10000, None),
        (i128::MIN, 0, None),
    ];
    for (gross, bp, expected) in cases {
        let result = validate_discount_wide(gross, DiscountInput::PercentageBasisPoints(bp));
        match expected {
            Some(value) => assert_eq!(result, Ok(value)),
            None => assert_eq!(result, Err(DomainError::InvalidInput)),
        }
    }
}
