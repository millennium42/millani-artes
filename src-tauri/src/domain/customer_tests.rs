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

const CUS003_VALID_DDDS: [u8; 67] = [
    11, 12, 13, 14, 15, 16, 17, 18, 19, 21, 22, 24, 27, 28, 31, 32, 33, 34, 35, 37, 38, 41, 42, 43,
    44, 45, 46, 47, 48, 49, 51, 53, 54, 55, 61, 62, 63, 64, 65, 66, 67, 68, 69, 71, 73, 74, 75, 77,
    79, 81, 82, 83, 84, 85, 86, 87, 88, 89, 91, 92, 93, 94, 95, 96, 97, 98, 99,
];

fn cus003_customer(name: &str, phone: Option<&str>) -> Customer {
    Customer {
        id: [0x63; 16],
        name: name.into(),
        phone: phone.map(String::from),
    }
}
fn cus003_assert_unchanged(customer: &Customer, name: &str, phone: Option<&str>) {
    assert_eq!(customer.id, [0x63; 16]);
    assert_eq!(customer.name, name);
    assert_eq!(customer.phone.as_deref(), phone);
}
#[test]
fn cus003_accepts_absence_and_all_67_area_codes_without_mutation_or_name_validation() {
    for name in ["", "   ", "  Cliente sintético ‘Á’\0  "] {
        let customer = cus003_customer(name, None);
        assert_eq!(customer.validate_phone(), Ok(()));
        cus003_assert_unchanged(&customer, name, None);
        for ddd in CUS003_VALID_DDDS {
            for subscriber in ["90000-0000", "91234-5678", "99999-9999"] {
                let phone = format!("({ddd:02}) {subscriber}");
                let customer = cus003_customer(name, Some(&phone));
                assert_eq!(customer.validate_phone(), Ok(()));
                cus003_assert_unchanged(&customer, name, Some(&phone));
            }
        }
    }
}
#[test]
fn cus003_rejects_all_33_unassigned_two_digit_area_codes_without_mutation() {
    for ddd in 0..=99_u8 {
        if CUS003_VALID_DDDS.contains(&ddd) {
            continue;
        }
        for name in ["", "   ", "  Cliente sintético  "] {
            let phone = format!("({ddd:02}) 91234-5678");
            let customer = cus003_customer(name, Some(&phone));
            assert_eq!(customer.validate_phone(), Err(DomainError::InvalidInput));
            cus003_assert_unchanged(&customer, name, Some(&phone));
        }
    }
}
#[test]
fn cus003_rejects_wrong_mask_lengths_prefixes_and_initial_digit_without_mutation() {
    for phone in [
        "",
        " ",
        "\t\r\n",
        "11912345678",
        "+55 (11) 91234-5678",
        "(11)91234-5678",
        "(11) 912345678",
        "11) 91234-5678",
        "(11) 91234-5678 ",
        " (11) 91234-5678",
        "(11) 81234-5678",
        "(11) 01234-5678",
        "(11) 9123-4567",
        "(11) 912345-6789",
        "(1) 91234-5678",
        "(111) 91234-5678",
        "(11) 91234.5678",
        "(11) 91234-5678\r\n",
        "(11) 91234-567",
    ] {
        for name in ["", "   ", "  Cliente sintético  "] {
            let customer = cus003_customer(name, Some(phone));
            assert_eq!(customer.validate_phone(), Err(DomainError::InvalidInput));
            cus003_assert_unchanged(&customer, name, Some(phone));
        }
    }
}
#[test]
fn cus003_rejects_unicode_nul_and_letters_at_every_position_without_panic_or_mutation() {
    let baseline = "(11) 91234-5678";
    for index in 0..baseline.len() {
        for replacement in ["é", "😀", "١", "１", "\0", "A", "–", "\u{00a0}"] {
            let mut phone = baseline.to_owned();
            phone.replace_range(index..index + 1, replacement);
            let customer = cus003_customer("  Cliente sintético  ", Some(&phone));
            assert_eq!(customer.validate_phone(), Err(DomainError::InvalidInput));
            cus003_assert_unchanged(&customer, "  Cliente sintético  ", Some(&phone));
        }
    }
    for replacement in ["é", "١", "１", "😀"] {
        for index in 0..=baseline.len() - replacement.len() {
            let mut phone = baseline.to_owned();
            phone.replace_range(index..index + replacement.len(), replacement);
            assert_eq!(phone.len(), baseline.len());
            let customer = cus003_customer("  Cliente sintético  ", Some(&phone));
            assert_eq!(customer.validate_phone(), Err(DomainError::InvalidInput));
            cus003_assert_unchanged(&customer, "  Cliente sintético  ", Some(&phone));
        }
    }
}
