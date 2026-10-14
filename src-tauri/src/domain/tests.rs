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
