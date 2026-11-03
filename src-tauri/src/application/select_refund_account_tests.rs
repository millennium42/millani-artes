use super::{select_refund_account, SelectRefundAccountError};
use crate::domain::{Account, AccountType};

fn account(id: [u8; 16], name: &str, account_type: AccountType) -> Account {
    Account {
        id,
        name: name.to_string(),
        account_type,
        initial_balance_cents: 0,
    }
}

#[test]
fn seleciona_exatamente_a_conta_escolhida() {
    let accounts = [
        account([1; 16], "Caixa", AccountType::Cash),
        account([2; 16], "Cartão", AccountType::Card),
    ];

    let selected =
        select_refund_account([2; 16], &accounts).expect("listed account must be selectable");

    assert_eq!(selected.id, [2; 16]);
    assert_eq!(selected.name, "Cartão");
    assert!(matches!(selected.account_type, AccountType::Card));
}

#[test]
fn rejeita_conta_ausente_ou_opcoes_vazias() {
    let accounts = [account([1; 16], "Caixa", AccountType::Cash)];

    let error = match select_refund_account([2; 16], &accounts) {
        Err(error) => error,
        Ok(_) => panic!("unlisted account must be rejected"),
    };
    assert_eq!(error, SelectRefundAccountError::AccountUnavailable);
    assert_eq!(error.to_string(), "selected refund account is unavailable");
    assert!(std::error::Error::source(&error).is_none());
    assert!(matches!(
        select_refund_account([2; 16], &[]),
        Err(SelectRefundAccountError::AccountUnavailable)
    ));
}
