use super::{DomainError, Result};

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
    let account = Account { id, name };

    assert_eq!(account.id, id);
    assert_eq!(account.name.as_bytes(), "Ateliê Millani 🧵".as_bytes());
}
