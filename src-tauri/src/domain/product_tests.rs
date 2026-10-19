use super::{DomainError, Product};

#[test]
fn prd001_template_preserves_fields_without_normalization() {
    for (is_active, suggested_price_cents) in [(true, 12_345_i64), (false, i64::MAX)] {
        let product = Product {
            id: [0x51; 16],
            name: "  Peça artesanal ‘azul’  ".into(),
            description: "\tDescrição \"livre\"\r\nLinha dois  ".into(),
            suggested_price_cents,
            is_active,
        };
        let id: [u8; 16] = product.id;
        let name: &String = &product.name;
        let description: &String = &product.description;
        let cents: i64 = product.suggested_price_cents;
        let active: bool = product.is_active;
        assert_eq!(id, [0x51; 16]);
        assert_eq!(name, "  Peça artesanal ‘azul’  ");
        assert_eq!(description, "\tDescrição \"livre\"\r\nLinha dois  ");
        assert_eq!(cents, suggested_price_cents);
        assert_eq!(active, is_active);
    }
}

fn prd002_product(name: &str, price: i64, active: bool) -> Product {
    Product {
        id: [0x52; 16],
        name: name.into(),
        description: "  Descrição\r\n‘livre’  ".into(),
        suggested_price_cents: price,
        is_active: active,
    }
}

fn prd002_assert_unchanged(product: &Product, name: &str, price: i64, active: bool) {
    assert_eq!(product.id, [0x52; 16]);
    assert_eq!(product.name, name);
    assert_eq!(product.description, "  Descrição\r\n‘livre’  ");
    assert_eq!(product.suggested_price_cents, price);
    assert_eq!(product.is_active, active);
}

#[test]
fn prd002_rejects_blank_names_without_mutation() {
    for name in ["", " ", "\t\r\n", "\u{00a0}\u{2003}", "\u{3000}"] {
        for price in [i64::MIN, 0, i64::MAX] {
            for active in [true, false] {
                let product = prd002_product(name, price, active);
                assert_eq!(product.validate_name(), Err(DomainError::InvalidInput));
                prd002_assert_unchanged(&product, name, price, active);
            }
        }
    }
}

#[test]
fn prd002_accepts_content_without_normalization_or_price_state_coupling() {
    for name in [
        "P",
        "  Árvore azul  ",
        "\u{00a0}Peça\u{2003}",
        "O'Brien \"livre\"",
        "0",
        "e\u{0301}",
    ] {
        for price in [i64::MIN, 0, i64::MAX] {
            for active in [true, false] {
                let product = prd002_product(name, price, active);
                assert_eq!(product.validate_name(), Ok(()));
                prd002_assert_unchanged(&product, name, price, active);
            }
        }
    }
}

#[test]
fn prd003_accepts_zero_and_positive_prices_without_mutation_or_name_state_coupling() {
    for name in ["", "   ", "  Peça ‘azul’  "] {
        for price in [0, 1, i64::MAX] {
            for active in [true, false] {
                let product = prd002_product(name, price, active);
                assert_eq!(product.validate_suggested_price(), Ok(()));
                prd002_assert_unchanged(&product, name, price, active);
            }
        }
    }
}

#[test]
fn prd003_rejects_negative_prices_without_mutation_or_name_state_coupling() {
    for name in ["", "   ", "  Peça ‘azul’  "] {
        for price in [-1, i64::MIN] {
            for active in [true, false] {
                let product = prd002_product(name, price, active);
                assert_eq!(
                    product.validate_suggested_price(),
                    Err(DomainError::InvalidInput)
                );
                prd002_assert_unchanged(&product, name, price, active);
            }
        }
    }
}
