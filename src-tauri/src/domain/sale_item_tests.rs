use super::{validate_discount, DiscountInput, DomainError, SaleItem};

#[test]
fn sal001_snapshot_fields_preserve_owned_values() {
    for product_id in [None, Some([0x71; 16])] {
        let item = SaleItem {
            id: [0x72; 16],
            product_id,
            description: "  Item sintético ‘Á’ \"livre\"\r\n\0  ".into(),
            quantity: 2,
            unit_price_cents: 1500,
            discount_cents: 300,
            total_cents: 2700,
        };
        let id: [u8; 16] = item.id;
        let origin: Option<[u8; 16]> = item.product_id;
        let description: &String = &item.description;
        let quantity: i64 = item.quantity;
        let unit_price: i64 = item.unit_price_cents;
        let discount: i64 = item.discount_cents;
        let total: i64 = item.total_cents;
        assert_eq!(id, [0x72; 16]);
        assert_eq!(origin, product_id);
        assert_eq!(description, "  Item sintético ‘Á’ \"livre\"\r\n\0  ");
        assert_eq!(
            (quantity, unit_price, discount, total),
            (2, 1500, 300, 2700)
        );
    }
}

fn sal002_item(quantity: i64, product_id: Option<[u8; 16]>) -> SaleItem {
    SaleItem {
        id: [0x73; 16],
        product_id,
        description: "  Sintético ‘Á’\r\n\0  ".into(),
        quantity,
        unit_price_cents: -1,
        discount_cents: i64::MAX,
        total_cents: i64::MIN,
    }
}

fn sal002_assert_unchanged(item: &SaleItem, quantity: i64, product_id: Option<[u8; 16]>) {
    assert_eq!(item.id, [0x73; 16]);
    assert_eq!(item.product_id, product_id);
    assert_eq!(item.description, "  Sintético ‘Á’\r\n\0  ");
    assert_eq!(item.quantity, quantity);
    assert_eq!(item.unit_price_cents, -1);
    assert_eq!(item.discount_cents, i64::MAX);
    assert_eq!(item.total_cents, i64::MIN);
}

#[test]
fn sal002_rejects_nonpositive_quantity_without_mutation() {
    for quantity in [i64::MIN, -1, 0] {
        for product_id in [None, Some([0x74; 16])] {
            let item = sal002_item(quantity, product_id);
            assert_eq!(item.validate_quantity(), Err(DomainError::InvalidInput));
            sal002_assert_unchanged(&item, quantity, product_id);
        }
    }
}

#[test]
fn sal002_accepts_positive_quantity_without_validating_other_fields() {
    for quantity in [1, 2, i64::MAX] {
        for product_id in [None, Some([0x74; 16])] {
            let item = sal002_item(quantity, product_id);
            assert_eq!(item.validate_quantity(), Ok(()));
            sal002_assert_unchanged(&item, quantity, product_id);
        }
    }
}

fn sal004_assert_calculation(
    quantity: i64,
    price: i64,
    discount: i64,
    expected: std::result::Result<i64, DomainError>,
) {
    for product_id in [None, Some([0x76; 16])] {
        let item = SaleItem {
            id: [0x75; 16],
            product_id,
            description: "  Sintético ‘Á’\r\n\0  ".into(),
            quantity,
            unit_price_cents: price,
            discount_cents: discount,
            total_cents: 42,
        };
        assert_eq!(item.calculate_total(), expected);
        assert_eq!(item.id, [0x75; 16]);
        assert_eq!(item.product_id, product_id);
        assert_eq!(item.description, "  Sintético ‘Á’\r\n\0  ");
        assert_eq!(item.quantity, quantity);
        assert_eq!(item.unit_price_cents, price);
        assert_eq!(item.discount_cents, discount);
        assert_eq!(item.total_cents, 42);
    }
}

#[test]
fn sal004_computes_net_and_accepts_zero_or_full_discount() {
    for (q, price, discount, total) in [
        (2, 1500, 0, 3000),
        (2, 1500, 300, 2700),
        (2, 1500, 3000, 0),
        (1, 0, 0, 0),
    ] {
        sal004_assert_calculation(q, price, discount, Ok(total));
    }
}

#[test]
fn sal004_wide_intermediates_preserve_representable_i64_results() {
    for (q, price, discount, total) in [
        (2, i64::MAX, i64::MAX, i64::MAX),
        (i64::MAX, 2, i64::MAX, i64::MAX),
        (i64::MAX, 0, 0, 0),
        (1, i64::MAX, 0, i64::MAX),
        (1, i64::MAX, i64::MAX, 0),
    ] {
        sal004_assert_calculation(q, price, discount, Ok(total));
    }
}

#[test]
fn sal004_rejects_nonpositive_quantity_and_negative_prices() {
    for q in [i64::MIN, -1, 0] {
        for price in [i64::MIN, -1, 0, 1] {
            sal004_assert_calculation(q, price, 0, Err(DomainError::InvalidInput));
        }
    }
    for price in [i64::MIN, -1] {
        sal004_assert_calculation(1, price, 0, Err(DomainError::InvalidInput));
    }
}

#[test]
fn sal004_rejects_invalid_discount_and_unrepresentable_final_total() {
    for (q, price, discount) in [
        (1, 1000, -1),
        (1, 1000, i64::MIN),
        (1, 1000, 1001),
        (1, 0, 1),
        (2, i64::MAX, 0),
        (2, i64::MAX, i64::MAX - 1),
        (i64::MAX, i64::MAX, i64::MAX),
    ] {
        sal004_assert_calculation(q, price, discount, Err(DomainError::InvalidInput));
    }
}

#[test]
fn sal004_consumes_applied_discount_once_in_both_input_modes() {
    for input in [
        DiscountInput::AmountCents(1),
        DiscountInput::PercentageBasisPoints(1000),
    ] {
        let applied = validate_discount(5, input).unwrap();
        assert_eq!(applied, 1);
        sal004_assert_calculation(1, 5, applied, Ok(4));
    }
}
