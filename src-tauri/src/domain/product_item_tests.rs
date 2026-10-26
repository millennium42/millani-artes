use super::{create_product_item, DiscountInput, DomainError, Product, SaleItem};

fn template() -> Product {
    Product {
        id: [0x21; 16],
        name: "Produto sintético".into(),
        description: "  Descrição ‘Á’ \"produto\"\r\n\0  ".into(),
        suggested_price_cents: 1500,
        is_active: true,
    }
}
fn check(
    item: &SaleItem,
    id: [u8; 16],
    product_id: [u8; 16],
    description: &str,
    amounts: [i64; 4],
) {
    assert_eq!(item.id, id);
    assert_eq!(item.product_id, Some(product_id));
    assert_eq!(item.description, description);
    assert_eq!(
        [
            item.quantity,
            item.unit_price_cents,
            item.discount_cents,
            item.total_cents
        ],
        amounts
    );
}
fn snapshot(product: &Product) -> ([u8; 16], String, String, i64, bool) {
    (
        product.id,
        product.name.clone(),
        product.description.clone(),
        product.suggested_price_cents,
        product.is_active,
    )
}
#[test]
fn sal007_copies_owned_fields_and_selected_price_without_template_borrow() {
    let product = template();
    let description = product.description.clone();
    let item = create_product_item(
        [0x61; 16],
        &product,
        2,
        1700,
        DiscountInput::AmountCents(300),
    )
    .unwrap();
    drop(product);
    check(
        &item,
        [0x61; 16],
        [0x21; 16],
        &description,
        [2, 1700, 300, 3100],
    );
}
#[test]
fn sal007_preserves_empty_whitespace_unicode_and_zero_identity() {
    for description in ["", " \t\r\n", "e\u{0301}", "Descrição ‘🧵’\0"] {
        let mut product = template();
        product.id = [0; 16];
        product.description = description.into();
        let item =
            create_product_item([0; 16], &product, 1, 1500, DiscountInput::AmountCents(0)).unwrap();
        check(&item, [0; 16], [0; 16], description, [1, 1500, 0, 1500]);
    }
}
#[test]
fn sal007_rejects_inactive_or_invalid_templates() {
    for (name, price, active) in [
        ("", 1500, true),
        (" \u{2003}", 1500, true),
        ("Válido", -1, true),
        ("Válido", 1500, false),
    ] {
        let mut product = template();
        product.name = name.into();
        product.suggested_price_cents = price;
        product.is_active = active;
        assert_eq!(
            create_product_item([0x61; 16], &product, 1, 1000, DiscountInput::AmountCents(0)).err(),
            Some(DomainError::InvalidInput)
        );
    }
}
#[test]
fn sal007_reuses_zero_full_rounding_and_wide_gross_policies() {
    let mut product = template();
    product.suggested_price_cents = 0;
    for (q, p, input, d, t) in [
        (i64::MAX, 0, DiscountInput::AmountCents(0), 0, 0),
        (1, 1500, DiscountInput::AmountCents(1500), 1500, 0),
        (1, 5, DiscountInput::PercentageBasisPoints(1000), 1, 4),
        (
            2,
            i64::MAX,
            DiscountInput::PercentageBasisPoints(5000),
            i64::MAX,
            i64::MAX,
        ),
        (
            1,
            1500,
            DiscountInput::PercentageBasisPoints(10000),
            1500,
            0,
        ),
    ] {
        let item = create_product_item([0x61; 16], &product, q, p, input).unwrap();
        check(
            &item,
            [0x61; 16],
            product.id,
            &product.description,
            [q, p, d, t],
        );
    }
}
#[test]
fn sal007_rejects_invalid_quantity_price_discounts_and_final_amount() {
    let product = template();
    for (q, p, input) in [
        (0, 1000, DiscountInput::AmountCents(0)),
        (-1, 1000, DiscountInput::AmountCents(0)),
        (i64::MIN, 0, DiscountInput::AmountCents(0)),
        (1, -1, DiscountInput::AmountCents(0)),
        (1, 1000, DiscountInput::AmountCents(-1)),
        (1, 1000, DiscountInput::AmountCents(1001)),
        (1, 1000, DiscountInput::PercentageBasisPoints(-1)),
        (1, 1000, DiscountInput::PercentageBasisPoints(10001)),
        (2, i64::MAX, DiscountInput::AmountCents(0)),
        (2, i64::MAX, DiscountInput::PercentageBasisPoints(10000)),
    ] {
        assert_eq!(
            create_product_item([0x61; 16], &product, q, p, input).err(),
            Some(DomainError::InvalidInput)
        );
    }
}
#[test]
fn sal007_leaves_template_unchanged_on_success_and_failure() {
    let product = template();
    let before = snapshot(&product);
    let item = create_product_item(
        [0x61; 16],
        &product,
        2,
        product.suggested_price_cents,
        DiscountInput::PercentageBasisPoints(5000),
    )
    .unwrap();
    check(
        &item,
        [0x61; 16],
        product.id,
        &product.description,
        [2, 1500, 1500, 1500],
    );
    assert_eq!(snapshot(&product), before);
    assert_eq!(
        create_product_item([0x61; 16], &product, 0, 1500, DiscountInput::AmountCents(0)).err(),
        Some(DomainError::InvalidInput)
    );
    assert_eq!(snapshot(&product), before);
}
