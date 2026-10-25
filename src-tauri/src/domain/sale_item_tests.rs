use super::{DomainError, SaleItem};

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
