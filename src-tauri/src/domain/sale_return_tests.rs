use super::{DomainError, SaleItem, SaleReturnItem};

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

fn ret002_sale_item(quantity: i64) -> SaleItem {
    SaleItem {
        id: [0x41; 16],
        product_id: None,
        description: "synthetic return item".into(),
        quantity,
        unit_price_cents: 0,
        discount_cents: 0,
        total_cents: 0,
    }
}

fn ret002_return(sale_item_id: [u8; 16], quantity: i64) -> SaleReturnItem {
    SaleReturnItem {
        sale_item_id,
        quantity,
    }
}

#[test]
fn ret002_calculates_available_quantity_for_matching_sale_item() {
    let sale_item = ret002_sale_item(2);
    let partial = [
        ret002_return(sale_item.id, 1),
        ret002_return([0x42; 16], i64::MIN),
    ];
    let zero = [ret002_return(sale_item.id, 0)];
    let full = [
        ret002_return(sale_item.id, 1),
        ret002_return(sale_item.id, 1),
    ];

    assert_eq!(sale_item.available_return_quantity(&[]), Ok(2));
    assert_eq!(sale_item.available_return_quantity(&zero), Ok(2));
    assert_eq!(sale_item.available_return_quantity(&partial), Ok(1));
    assert_eq!(sale_item.available_return_quantity(&full), Ok(0));
    assert_eq!(sale_item.quantity, 2);

    let maximum_item = ret002_sale_item(i64::MAX);
    let maximum_return = [ret002_return(maximum_item.id, i64::MAX)];
    assert_eq!(
        maximum_item.available_return_quantity(&maximum_return),
        Ok(0)
    );
}

#[test]
fn ret002_rejects_invalid_return_quantities_and_history() {
    let sale_item = ret002_sale_item(2);

    for quantity in [i64::MIN, -1] {
        let returned = [ret002_return(sale_item.id, quantity)];
        assert_eq!(
            sale_item.available_return_quantity(&returned),
            Err(DomainError::InvalidInput)
        );
    }

    let excessive = [ret002_return(sale_item.id, 3)];
    assert_eq!(
        sale_item.available_return_quantity(&excessive),
        Err(DomainError::InvalidInput)
    );

    let excessive_history = [
        ret002_return(sale_item.id, 1),
        ret002_return(sale_item.id, 2),
    ];
    assert_eq!(
        sale_item.available_return_quantity(&excessive_history),
        Err(DomainError::InvalidInput)
    );

    for quantity in [i64::MIN, -1, 0] {
        let invalid_sale_item = ret002_sale_item(quantity);
        assert_eq!(
            invalid_sale_item.available_return_quantity(&[]),
            Err(DomainError::InvalidInput)
        );
    }
}

#[test]
fn ret003_rejects_requested_quantity_above_available_without_mutation() {
    let sale_item = ret002_sale_item(2);
    let returns = [
        ret002_return(sale_item.id, 1),
        ret002_return([0x42; 16], i64::MAX),
    ];

    assert_eq!(sale_item.validate_return_quantity(1, &returns), Ok(()));
    assert_eq!(
        sale_item.validate_return_quantity(2, &returns),
        Err(DomainError::InvalidInput)
    );
    assert_eq!(sale_item.quantity, 2);
    assert_eq!(returns[0].quantity, 1);
    assert_eq!(returns[1].quantity, i64::MAX);
}

#[test]
fn ret003_accepts_full_return_and_rejects_nonpositive_or_excess_requests() {
    let sale_item = ret002_sale_item(2);

    assert_eq!(sale_item.validate_return_quantity(2, &[]), Ok(()));
    assert_eq!(
        sale_item.validate_return_quantity(3, &[]),
        Err(DomainError::InvalidInput)
    );
    for quantity in [i64::MIN, -1, 0] {
        assert_eq!(
            sale_item.validate_return_quantity(quantity, &[]),
            Err(DomainError::InvalidInput)
        );
    }

    let invalid_sale_item = ret002_sale_item(0);
    assert_eq!(
        invalid_sale_item.validate_return_quantity(1, &[]),
        Err(DomainError::InvalidInput)
    );

    let fully_returned = [ret002_return(sale_item.id, 2)];
    assert_eq!(
        sale_item.validate_return_quantity(1, &fully_returned),
        Err(DomainError::InvalidInput)
    );
    let invalid_history = [ret002_return(sale_item.id, 3)];
    assert_eq!(
        sale_item.validate_return_quantity(1, &invalid_history),
        Err(DomainError::InvalidInput)
    );
}
