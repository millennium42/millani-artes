use super::SaleItem;

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
