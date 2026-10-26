use super::{
    create_free_item, create_product_item, DiscountInput, Product, Sale, SaleItem, SaleStatus,
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
