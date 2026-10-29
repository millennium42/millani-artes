use super::{
    create_free_item, create_product_item, DiscountInput, DomainError, Product, Sale, SaleItem,
    SalePayment, SalePending, SaleStatus,
};

fn check(
    item: &SaleItem,
    id: [u8; 16],
    product: Option<[u8; 16]>,
    description: &str,
    amounts: [i64; 4],
) {
    assert_eq!(item.id, id);
    assert_eq!(item.product_id, product);
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
#[test]
fn sal009_groups_owned_item_snapshots_in_order_without_template_borrow() {
    let product = Product {
        id: [0x21; 16],
        name: "Produto sintético".into(),
        description: "  Descrição ‘Á’\r\n\0  ".into(),
        suggested_price_cents: 1500,
        is_active: true,
    };
    let description = product.description.clone();
    let first =
        create_product_item([0x31; 16], &product, 2, 1500, DiscountInput::AmountCents(0)).unwrap();
    let second = create_free_item(
        [0x32; 16],
        "Frete".into(),
        1,
        1000,
        DiscountInput::AmountCents(0),
    )
    .unwrap();
    let sale = Sale {
        id: [0x41; 16],
        status: SaleStatus::Open,
        items: vec![first, second],
    };
    drop(product);
    assert_eq!(sale.id, [0x41; 16]);
    assert!(matches!(&sale.status, SaleStatus::Open));
    assert_eq!(sale.items.len(), 2);
    check(
        &sale.items[0],
        [0x31; 16],
        Some([0x21; 16]),
        &description,
        [2, 1500, 0, 3000],
    );
    check(
        &sale.items[1],
        [0x32; 16],
        None,
        "Frete",
        [1, 1000, 0, 1000],
    );
}
#[test]
fn sal009_preserves_identity_both_states_and_empty_collection_as_metadata() {
    for (id, status, finalized) in [
        ([0; 16], SaleStatus::Open, false),
        ([u8::MAX; 16], SaleStatus::Finalized, true),
    ] {
        let sale = Sale {
            id,
            status,
            items: Vec::new(),
        };
        assert_eq!(sale.id, id);
        assert!(sale.items.is_empty());
        assert_eq!(matches!(&sale.status, SaleStatus::Finalized), finalized);
        assert_eq!(matches!(&sale.status, SaleStatus::Open), !finalized);
    }
}

#[test]
fn sal016_preserves_each_payment_account_amount_and_order() {
    let payments = Vec::from([
        SalePayment {
            account_id: [0x51; 16],
            amount_cents: 500,
        },
        SalePayment {
            account_id: [0x52; 16],
            amount_cents: 1000,
        },
    ]);

    assert_eq!(payments.len(), 2);
    assert_eq!(payments[0].account_id, [0x51; 16]);
    assert_eq!(payments[0].amount_cents, 500);
    assert_eq!(payments[1].account_id, [0x52; 16]);
    assert_eq!(payments[1].amount_cents, 1000);
}

#[test]
fn sal017_preserves_pending_remainder_in_cents() {
    let pending = SalePending {
        amount_cents: 1500,
        customer_id: [0x17; 16],
    };

    assert_eq!(pending.amount_cents, 1500);
}

#[test]
fn sal018_preserves_required_customer_id_for_pending_sale() {
    let customer_id = [0x61; 16];
    let pending = SalePending {
        amount_cents: 1500,
        customer_id,
    };

    assert_eq!(pending.customer_id, customer_id);
    assert_eq!(pending.amount_cents, 1500);
}

#[test]
fn sal015_sums_product_and_free_item_calculations_without_trusting_stored_totals() {
    let product = Product {
        id: [0x21; 16],
        name: "Produto sintético".into(),
        description: "Produto A".into(),
        suggested_price_cents: 1500,
        is_active: true,
    };
    let mut product_item =
        create_product_item([0x31; 16], &product, 2, 1500, DiscountInput::AmountCents(0)).unwrap();
    let mut free_item = create_free_item(
        [0x32; 16],
        "Frete".into(),
        1,
        1000,
        DiscountInput::AmountCents(0),
    )
    .unwrap();

    product_item.total_cents = 42;
    free_item.total_cents = 24;
    let sale = Sale {
        id: [0x41; 16],
        status: SaleStatus::Open,
        items: vec![product_item, free_item],
    };

    assert_eq!(sale.calculate_total(), Ok(4000));
    assert_eq!(sale.id, [0x41; 16]);
    assert!(matches!(&sale.status, SaleStatus::Open));
    check(
        &sale.items[0],
        [0x31; 16],
        Some([0x21; 16]),
        "Produto A",
        [2, 1500, 0, 42],
    );
    check(&sale.items[1], [0x32; 16], None, "Frete", [1, 1000, 0, 24]);
}

#[test]
fn sal015_returns_zero_for_empty_sale_and_sums_applied_discount_once() {
    let empty = Sale {
        id: [0x41; 16],
        status: SaleStatus::Open,
        items: Vec::new(),
    };
    assert_eq!(empty.calculate_total(), Ok(0));

    let item = create_free_item(
        [0x32; 16],
        "Desconto".into(),
        1,
        1000,
        DiscountInput::PercentageBasisPoints(1000),
    )
    .unwrap();
    assert_eq!([item.discount_cents, item.total_cents], [100, 900]);
    let sale = Sale {
        id: [0x42; 16],
        status: SaleStatus::Finalized,
        items: vec![item],
    };
    assert!(matches!(&sale.status, SaleStatus::Finalized));
    assert_eq!(sale.calculate_total(), Ok(900));
}

#[test]
fn sal015_accepts_exact_i64_max_and_rejects_aggregate_overflow() {
    let almost_max = create_free_item(
        [0x32; 16],
        "A".into(),
        1,
        i64::MAX - 1,
        DiscountInput::AmountCents(0),
    )
    .unwrap();
    let one =
        create_free_item([0x33; 16], "B".into(), 1, 1, DiscountInput::AmountCents(0)).unwrap();
    let mut sale = Sale {
        id: [0x41; 16],
        status: SaleStatus::Open,
        items: vec![almost_max, one],
    };
    assert_eq!(sale.calculate_total(), Ok(i64::MAX));

    let max = create_free_item(
        [0x34; 16],
        "C".into(),
        1,
        i64::MAX,
        DiscountInput::AmountCents(0),
    )
    .unwrap();
    let one_over =
        create_free_item([0x35; 16], "D".into(), 1, 1, DiscountInput::AmountCents(0)).unwrap();
    sale.items = vec![max, one_over];
    assert_eq!(sale.calculate_total(), Err(DomainError::InvalidInput));
}

#[test]
fn sal015_propagates_invalid_item_error() {
    let sale = Sale {
        id: [0x41; 16],
        status: SaleStatus::Open,
        items: vec![SaleItem {
            id: [0x32; 16],
            product_id: None,
            description: "Inválido".into(),
            quantity: 0,
            unit_price_cents: 1000,
            discount_cents: 0,
            total_cents: 0,
        }],
    };

    assert_eq!(sale.calculate_total(), Err(DomainError::InvalidInput));
}
