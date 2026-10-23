use super::Customer;

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
