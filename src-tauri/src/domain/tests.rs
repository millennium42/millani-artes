use super::{AccountType, DomainError, Result};

#[test]
fn successful_result_preserves_value() {
    let result: Result<&str> = Ok("SYNTHETIC_VALUE");
    assert_eq!(result, Ok("SYNTHETIC_VALUE"));
}

#[test]
fn failed_result_propagates_without_payload() {
    fn forward(input: Result<i64>) -> Result<i64> {
        let value = input?;
        Ok(value)
    }

    assert_eq!(forward(Ok(42)), Ok(42));
    assert_eq!(
        forward(Err(DomainError::InvalidInput)),
        Err(DomainError::InvalidInput)
    );
}

#[test]
fn error_has_static_display_and_no_source() {
    let error = DomainError::InvalidInput;
    assert_eq!(format!("{error}"), "DOMAIN_INVALID_INPUT");
    assert_eq!(format!("{error:?}"), "InvalidInput");
    assert!(std::error::Error::source(&error).is_none());
}

#[test]
fn account_preserves_opaque_id_and_owned_unicode_name() {
    use super::Account;

    let id = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 255];
    let name = String::from("Ateliê Millani 🧵");
    let account = Account {
        id,
        name,
        account_type: AccountType::BankPix,
        initial_balance_cents: 0,
    };

    assert_eq!(account.id, id);
    assert_eq!(account.name.as_bytes(), "Ateliê Millani 🧵".as_bytes());
}

#[test]
fn account_name_rejects_empty() {
    use super::Account;

    let account = Account {
        id: [0; 16],
        name: String::new(),
        account_type: AccountType::BankPix,
        initial_balance_cents: 0,
    };

    assert_eq!(account.validate_name(), Err(DomainError::InvalidInput));
}

#[test]
fn account_name_rejects_unicode_whitespace() {
    use super::Account;

    for name in [
        " ",
        "\t",
        "\n\r",
        "\u{00a0}",
        "\u{2003}",
        " \t\n\r\u{00a0}\u{2003}",
    ] {
        let account = Account {
            id: [0; 16],
            name: name.to_owned(),
            account_type: AccountType::BankPix,
            initial_balance_cents: 0,
        };

        assert_eq!(account.validate_name(), Err(DomainError::InvalidInput));
    }
}

#[test]
fn account_name_accepts_unicode_without_mutation() {
    use super::Account;

    let id = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 255];
    let name = String::from("  Ateliê Millani 🧵  ");
    let account = Account {
        id,
        name,
        account_type: AccountType::BankPix,
        initial_balance_cents: 0,
    };

    account.validate_name().unwrap();

    assert_eq!(account.id, id);
    assert_eq!(account.name.as_bytes(), "  Ateliê Millani 🧵  ".as_bytes());
}

#[test]
fn account_preserves_each_account_type() {
    use super::Account;

    for account_type in [
        AccountType::Cash,
        AccountType::BankPix,
        AccountType::Card,
        AccountType::Other,
    ] {
        let expected = std::mem::discriminant(&account_type);
        let account = Account {
            id: [0; 16],
            name: String::from("SYNTHETIC_ACCOUNT"),
            account_type,
            initial_balance_cents: 0,
        };

        assert_eq!(std::mem::discriminant(&account.account_type), expected);
    }
}

#[test]
fn account_preserves_initial_balance_cents() {
    use super::Account;

    for initial_balance_cents in [i64::MIN, -123456, 0, 1, 123456, i64::MAX] {
        let account = Account {
            id: [0; 16],
            name: String::from("SYNTHETIC_ACCOUNT"),
            account_type: AccountType::BankPix,
            initial_balance_cents,
        };

        assert_eq!(account.initial_balance_cents, initial_balance_cents);
    }
}

#[test]
fn category_preserves_identity_name_and_active_state() {
    use super::Category;

    let id = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 255];
    let name = "  Categoria Açúcar 🧵  ";
    for is_active in [true, false] {
        let category = Category {
            id,
            name: String::from(name),
            is_active,
        };

        assert_eq!(category.id, id);
        assert_eq!(category.name.as_bytes(), name.as_bytes());
        assert_eq!(category.is_active, is_active);
    }
}

#[test]
fn context_variants_are_distinct() {
    use super::Context;

    assert_ne!(
        std::mem::discriminant(&Context::Home),
        std::mem::discriminant(&Context::Factory)
    );
}

#[test]
fn category_name_rejects_empty() {
    use super::Category;

    let category = Category {
        id: [0; 16],
        name: String::new(),
        is_active: true,
    };

    assert_eq!(category.validate_name(), Err(DomainError::InvalidInput));
}

#[test]
fn category_name_rejects_unicode_whitespace() {
    use super::Category;

    for name in [
        " ",
        "\t",
        "\n",
        "\r",
        "\u{00a0}",
        "\u{2003}",
        " \t\n\u{00a0}\u{2003} ",
    ] {
        let category = Category {
            id: [0; 16],
            name: String::from(name),
            is_active: true,
        };

        assert_eq!(category.validate_name(), Err(DomainError::InvalidInput));
    }
}

#[test]
fn category_name_accepts_unicode_without_mutation() {
    use super::Category;

    let id = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 255];
    let name = "  Categoria Açúcar 🧵  ";
    for is_active in [true, false] {
        let category = Category {
            id,
            name: String::from(name),
            is_active,
        };

        category.validate_name().unwrap();

        assert_eq!(category.id, id);
        assert_eq!(category.name.as_bytes(), name.as_bytes());
        assert_eq!(category.is_active, is_active);
    }
}

#[test]
fn context_accepts_home_key() {
    use super::Context;

    assert!(matches!(Context::try_from("home"), Ok(Context::Home)));
}

#[test]
fn context_accepts_factory_key() {
    use super::Context;

    assert!(matches!(Context::try_from("factory"), Ok(Context::Factory)));
}

#[test]
fn context_rejects_invalid_keys() {
    use super::Context;

    for value in [
        "", " ", "\t\n", "HOME", "Factory", " home", "factory ", "fac tory", "other", "Casa",
        "Fábrica",
    ] {
        assert!(matches!(
            Context::try_from(value),
            Err(DomainError::InvalidInput)
        ));
    }
}
