use super::SaleReturnItem;

#[test]
fn ret001_preserves_sale_item_id_and_quantity() {
    let items = [
        SaleReturnItem {
            sale_item_id: [0x31; 16],
            quantity: 1,
        },
        SaleReturnItem {
            sale_item_id: [u8::MAX; 16],
            quantity: 2,
        },
    ];

    assert_eq!(items[0].sale_item_id, [0x31; 16]);
    assert_eq!(items[0].quantity, 1);
    assert_eq!(items[1].sale_item_id, [u8::MAX; 16]);
    assert_eq!(items[1].quantity, 2);
}
