use super::{Customer, DomainError};

#[test]
fn cus001_minimal_customer_preserves_fields_without_normalization() {
    for phone in [
        None,
        Some(""),
        Some("   "),
        Some("  Sintético ‘α’ \"livre\"\r\n\0  "),
    ] {
        let customer = Customer {
            id: [0x61; 16],
            name: "  Cliente sintético ‘Á’ \"livre\"\r\n\0  ".into(),
            phone: phone.map(String::from),
        };
        let id: [u8; 16] = customer.id;
        let name: &String = &customer.name;
        let stored_phone: &Option<String> = &customer.phone;
        assert_eq!(id, [0x61; 16]);
        assert_eq!(name, "  Cliente sintético ‘Á’ \"livre\"\r\n\0  ");
        assert_eq!(stored_phone.as_deref(), phone);
    }
}

fn cus002_customer(name: &str, phone: Option<&str>) -> Customer {
    Customer {
        id: [0x62; 16],
        name: name.into(),
        phone: phone.map(String::from),
    }
}

fn cus002_assert_unchanged(customer: &Customer, name: &str, phone: Option<&str>) {
    assert_eq!(customer.id, [0x62; 16]);
    assert_eq!(customer.name, name);
    assert_eq!(customer.phone.as_deref(), phone);
}

#[test]
fn cus002_rejects_blank_names_without_mutation_or_phone_validation() {
    for name in ["", " ", "\t\r\n", "\u{00a0}\u{2003}", "\u{3000}"] {
        for phone in [None, Some(""), Some("  Sintético ‘α’\r\n\0  ")] {
            let customer = cus002_customer(name, phone);
            assert_eq!(customer.validate_name(), Err(DomainError::InvalidInput));
            cus002_assert_unchanged(&customer, name, phone);
        }
    }
}

#[test]
fn cus002_accepts_content_without_normalization_or_phone_validation() {
    for name in [
        "C",
        "  Cliente ‘Á’  ",
        "\u{00a0}Cliente\u{2003}",
        "O'Brien \"livre\"",
        "0",
        "e\u{0301}",
    ] {
        for phone in [None, Some(""), Some("  Sintético ‘α’\r\n\0  ")] {
            let customer = cus002_customer(name, phone);
            assert_eq!(customer.validate_name(), Ok(()));
            cus002_assert_unchanged(&customer, name, phone);
        }
    }
}
