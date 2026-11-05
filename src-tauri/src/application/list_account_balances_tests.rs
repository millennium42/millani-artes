use super::{list_account_balances, AccountBalance, ListAccountBalancesError};
use crate::application::list_accounts::AccountReadStore;
use crate::application::list_movements::MovementReadStore;
use crate::domain::{Account, AccountType, Context, Movement, MovementKind};
use std::cell::{Cell, RefCell};

struct FakeAccounts {
    result: RefCell<Option<Result<Vec<Account>, &'static str>>>,
    calls: Cell<usize>,
}

impl FakeAccounts {
    fn new(result: Result<Vec<Account>, &'static str>) -> Self {
        Self {
            result: RefCell::new(Some(result)),
            calls: Cell::new(0),
        }
    }
}

impl AccountReadStore for FakeAccounts {
    type Error = &'static str;

    fn read_all_accounts(&self) -> Result<Vec<Account>, Self::Error> {
        self.calls.set(self.calls.get() + 1);
        self.result.borrow_mut().take().expect("one account read")
    }
}

struct FakeMovements {
    result: RefCell<Option<Result<Vec<Movement>, &'static str>>>,
    calls: Cell<usize>,
}

impl FakeMovements {
    fn new(result: Result<Vec<Movement>, &'static str>) -> Self {
        Self {
            result: RefCell::new(Some(result)),
            calls: Cell::new(0),
        }
    }
}

impl MovementReadStore for FakeMovements {
    type Error = &'static str;

    fn read_all_movements(&self) -> Result<Vec<Movement>, Self::Error> {
        self.calls.set(self.calls.get() + 1);
        self.result.borrow_mut().take().expect("one movement read")
    }
}

fn account(id: u8, name: &str, account_type: AccountType, opening: i64) -> Account {
    Account {
        id: [id; 16],
        name: name.into(),
        account_type,
        initial_balance_cents: opening,
    }
}

fn movement(id: u8, amount_cents: i64, context: Context, kind: MovementKind) -> Movement {
    Movement {
        id: [id; 16],
        amount_cents,
        context,
        kind,
    }
}

fn assert_error(result: Result<Vec<AccountBalance>, ListAccountBalancesError>) {
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("expected opaque account balance error"),
    };
    assert_eq!(format!("{error:?}"), "ListAccountBalancesError");
    assert_eq!(format!("{error}"), "account balances could not be listed");
    assert!(std::error::Error::source(&error).is_none());
}

#[test]
fn balances_preserve_account_order_and_metadata_and_read_each_port_once() {
    let accounts = FakeAccounts::new(Ok(vec![
        account(1, "Conta A", AccountType::Cash, -1234),
        account(2, "Conta B", AccountType::BankPix, 7654),
    ]));
    let movements = FakeMovements::new(Ok(vec![
        movement(
            1,
            100,
            Context::Home,
            MovementKind::Entry {
                account_id: [1; 16],
            },
        ),
        movement(
            2,
            30,
            Context::Factory,
            MovementKind::Exit {
                account_id: [1; 16],
            },
        ),
        movement(
            3,
            20,
            Context::Home,
            MovementKind::Transfer {
                source_account_id: [1; 16],
                destination_account_id: [2; 16],
            },
        ),
        movement(
            4,
            5,
            Context::Factory,
            MovementKind::Transfer {
                source_account_id: [2; 16],
                destination_account_id: [1; 16],
            },
        ),
    ]));

    let balances = list_account_balances(&accounts, &movements).unwrap();

    assert_eq!(balances.len(), 2);
    assert_eq!(balances[0].account.id, [1; 16]);
    assert_eq!(balances[0].account.name, "Conta A");
    assert!(matches!(
        balances[0].account.account_type,
        AccountType::Cash
    ));
    assert_eq!(balances[0].account.initial_balance_cents, -1234);
    assert_eq!(balances[0].balance_cents, -1179);
    assert_eq!(balances[1].account.id, [2; 16]);
    assert_eq!(balances[1].account.name, "Conta B");
    assert!(matches!(
        balances[1].account.account_type,
        AccountType::BankPix
    ));
    assert_eq!(balances[1].account.initial_balance_cents, 7654);
    assert_eq!(balances[1].balance_cents, 7669);
    assert_eq!(accounts.calls.get(), 1);
    assert_eq!(movements.calls.get(), 1);
}

#[test]
fn empty_accounts_return_an_empty_projection() {
    let accounts = FakeAccounts::new(Ok(vec![]));
    let movements = FakeMovements::new(Ok(vec![]));

    assert!(list_account_balances(&accounts, &movements)
        .unwrap()
        .is_empty());
    assert_eq!(accounts.calls.get(), 1);
    assert_eq!(movements.calls.get(), 1);
}

#[test]
fn account_read_failure_is_opaque_and_does_not_read_movements() {
    let accounts = FakeAccounts::new(Err("private account SQL path"));
    let movements = FakeMovements::new(Ok(vec![]));

    assert_error(list_account_balances(&accounts, &movements));
    assert_eq!(accounts.calls.get(), 1);
    assert_eq!(movements.calls.get(), 0);
}

#[test]
fn movement_read_failure_is_opaque() {
    let accounts = FakeAccounts::new(Ok(vec![account(1, "Conta A", AccountType::Cash, 0)]));
    let movements = FakeMovements::new(Err("private movement SQL path"));

    assert_error(list_account_balances(&accounts, &movements));
    assert_eq!(accounts.calls.get(), 1);
    assert_eq!(movements.calls.get(), 1);
}

#[test]
fn invalid_ledger_or_unrepresentable_balance_returns_no_partial_projection() {
    for (target_account, movement) in [
        (
            account(1, "Conta A", AccountType::Cash, 0),
            movement(
                1,
                0,
                Context::Home,
                MovementKind::Entry {
                    account_id: [2; 16],
                },
            ),
        ),
        (
            account(1, "Conta A", AccountType::Cash, i64::MAX),
            movement(
                1,
                1,
                Context::Home,
                MovementKind::Entry {
                    account_id: [1; 16],
                },
            ),
        ),
    ] {
        let accounts = FakeAccounts::new(Ok(vec![
            account(3, "Conta anterior", AccountType::Other, 50),
            target_account,
        ]));
        let movements = FakeMovements::new(Ok(vec![movement]));

        assert_error(list_account_balances(&accounts, &movements));
        assert_eq!(accounts.calls.get(), 1);
        assert_eq!(movements.calls.get(), 1);
    }
}
