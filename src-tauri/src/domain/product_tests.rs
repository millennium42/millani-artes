use super::Product;

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
