use super::{list_accounts, AccountReadStore, ListAccountsError};
use crate::domain::{Account, AccountType};
use std::cell::{Cell, RefCell};

struct FakeStore {
    result: RefCell<Option<Result<Vec<Account>, &'static str>>>,
    call_count: Cell<usize>,
}

impl FakeStore {
    fn new(result: Result<Vec<Account>, &'static str>) -> Self {
        FakeStore {
            result: RefCell::new(Some(result)),
            call_count: Cell::new(0),
        }
    }
}

impl AccountReadStore for FakeStore {
    type Error = &'static str;
    fn read_all_accounts(&self) -> Result<Vec<Account>, Self::Error> {
        self.call_count.set(self.call_count.get() + 1);
        self.result.borrow_mut().take().expect("fake called once")
    }
}

#[test]
fn list_accounts_vazio_sucesso() {
    let store = FakeStore::new(Ok(vec![]));
    let result = list_accounts(&store);
    assert!(result.is_ok());
    let accounts = result.unwrap();
    assert_eq!(accounts.len(), 0);
}

#[test]
fn list_accounts_sucesso_com_dados_complexos() {
    let account1 = Account {
        id: [1u8; 16],
        name: "Conta Unicode Ñoño".to_string(),
        account_type: AccountType::BankPix,
        initial_balance_cents: i64::MAX,
    };
    let account2 = Account {
        id: [2u8; 16],
        name: "Outra Conta".to_string(),
        account_type: AccountType::Card,
        initial_balance_cents: i64::MIN,
    };
    let store = FakeStore::new(Ok(vec![account1, account2]));
    let result = list_accounts(&store);
    assert!(result.is_ok());
    let accounts = result.unwrap();
    assert_eq!(accounts.len(), 2);
    assert_eq!(accounts[0].id, [1u8; 16]);
    assert_eq!(accounts[0].name, "Conta Unicode Ñoño");
    assert!(matches!(accounts[0].account_type, AccountType::BankPix));
    assert_eq!(accounts[0].initial_balance_cents, i64::MAX);
    assert_eq!(accounts[1].id, [2u8; 16]);
    assert_eq!(accounts[1].name, "Outra Conta");
    assert!(matches!(accounts[1].account_type, AccountType::Card));
    assert_eq!(accounts[1].initial_balance_cents, i64::MIN);
}

#[test]
fn list_accounts_erro_leitura() {
    let store = FakeStore::new(Err("erro de leitura"));
    let result = list_accounts(&store);
    assert!(result.is_err());
    let err = match result {
        Err(e) => e,
        Ok(_) => panic!("expected error"),
    };
    assert!(matches!(err, ListAccountsError));
    assert_eq!(format!("{:?}", err), "ListAccountsError");
    assert_eq!(format!("{}", err), "accounts could not be read");
    assert!(std::error::Error::source(&err).is_none());
    let err_str = format!("{}", err);
    assert!(!err_str.contains("erro de leitura"));
}

#[test]
fn list_accounts_conta_chamadas() {
    let store = FakeStore::new(Ok(vec![]));
    let _ = list_accounts(&store);
    assert_eq!(store.call_count.get(), 1);
}
