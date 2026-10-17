use super::fin008_account_creation_tests::{migrate_accounts, snapshot, FixedClock, FixedIds};
use super::*;
use millani_artes::application::create_account::create_account;
use millani_artes::application::list_accounts::list_accounts;
use millani_artes::domain::AccountType;
use millani_artes::infrastructure::accounts::AdapterSqliteAccountStore;
use std::time::UNIX_EPOCH;

fn id_from_u64(n: u64) -> [u8; 16] {
    let mut arr = [0u8; 16];
    arr[8..].copy_from_slice(&n.to_be_bytes());
    arr
}

fn type_str(t: &AccountType) -> &'static str {
    match t {
        AccountType::Cash => "Cash",
        AccountType::BankPix => "BankPix",
        AccountType::Card => "Card",
        AccountType::Other => "Other",
    }
}

pub(super) fn schema(connection: &Connection) -> Vec<(String, String)> {
    let mut statement = connection
        .prepare("SELECT name, coalesce(sql, '') FROM main.sqlite_schema ORDER BY name")
        .unwrap();
    let rows = statement
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap();
    rows.collect::<rusqlite::Result<_>>().unwrap()
}

#[test]
fn fin009_list_accounts_empty_physical() {
    let tmp = common::TempDirectory::new();
    let path = tmp.path().join("db.sqlite");
    let conn = migrate_accounts(&path);
    let observer_conn = open_database(&path).expect("open observer");
    let baseline_snapshot = snapshot(&observer_conn);
    let baseline_schema = schema(&observer_conn);
    let store = AdapterSqliteAccountStore::new(conn);
    let accounts1 = list_accounts(&store).expect("list");
    assert!(accounts1.is_empty());
    let snapshot1 = snapshot(&observer_conn);
    assert_eq!(baseline_snapshot, snapshot1);
    assert_eq!(baseline_schema, schema(&observer_conn));
    let accounts2 = list_accounts(&store).expect("list");
    assert!(accounts2.is_empty());
    let snapshot2 = snapshot(&observer_conn);
    assert_eq!(baseline_snapshot, snapshot2);
    assert_eq!(baseline_schema, schema(&observer_conn));
}

#[test]
fn fin009_list_accounts_success_physical() {
    let tmp = common::TempDirectory::new();
    let path = tmp.path().join("db.sqlite");
    let conn = migrate_accounts(&path);
    let observer_conn = open_database(&path).expect("open observer");
    let mut store = AdapterSqliteAccountStore::new(conn);

    let mut ids_vec = Vec::new();
    for i in (1..=48).rev() {
        ids_vec.push(id_from_u64(i as u64));
    }
    let mut ids = FixedIds::new(ids_vec.clone(), None);
    let clock = FixedClock::new(UNIX_EPOCH);

    let keys = ["cash", "bank_pix", "card", "other"];
    let balances = [i64::MIN, -123456, 0, 1, 123456, i64::MAX];
    let name_const = "Conta çãõ";

    let mut expected_entries = Vec::new();
    let mut idx = 0usize;
    for key in keys.iter() {
        for &b in balances.iter() {
            let account_type = match *key {
                "cash" => AccountType::Cash,
                "bank_pix" => AccountType::BankPix,
                "card" => AccountType::Card,
                "other" => AccountType::Other,
                _ => unreachable!(),
            };
            let type_name = type_str(&account_type);
            let account_id = ids_vec[2 * idx];
            expected_entries.push((account_id, name_const.to_string(), type_name.to_string(), b));
            let acc = create_account(
                name_const.to_string(),
                account_type,
                b,
                &mut ids,
                &clock,
                &mut store,
            )
            .expect("create");
            assert_eq!(acc.id, account_id);
            idx += 1;
        }
    }

    let snapshot_after_create = snapshot(&observer_conn);
    let schema_after_create = schema(&observer_conn);

    let mut expected_sorted = expected_entries.clone();
    expected_sorted.sort_by_key(|a| a.0);

    let accounts = list_accounts(&store).expect("list");
    assert_eq!(accounts.len(), 24);
    for (acc, (exp_id, exp_name, exp_type_name, exp_balance)) in
        accounts.iter().zip(expected_sorted.iter())
    {
        assert_eq!(acc.id, *exp_id);
        assert_eq!(acc.name.as_str(), exp_name.as_str());
        assert_eq!(type_str(&acc.account_type), exp_type_name.as_str());
        assert_eq!(acc.initial_balance_cents, *exp_balance);
    }
    let snapshot_after_list = snapshot(&observer_conn);
    assert_eq!(snapshot_after_create, snapshot_after_list);
    assert_eq!(schema_after_create, schema(&observer_conn));

    let accounts2 = list_accounts(&store).expect("list");
    assert_eq!(accounts2.len(), 24);
    for (acc, (exp_id, exp_name, exp_type_name, exp_balance)) in
        accounts2.iter().zip(expected_sorted.iter())
    {
        assert_eq!(acc.id, *exp_id);
        assert_eq!(acc.name.as_str(), exp_name.as_str());
        assert_eq!(type_str(&acc.account_type), exp_type_name.as_str());
        assert_eq!(acc.initial_balance_cents, *exp_balance);
    }

    drop(store);
    drop(observer_conn);

    let conn2 = open_database(&path).expect("open");
    let observer_conn2 = open_database(&path).expect("open observer2");
    let store2 = AdapterSqliteAccountStore::new(conn2);
    let accounts3 = list_accounts(&store2).expect("list");
    assert_eq!(accounts3.len(), 24);
    for (acc, (exp_id, exp_name, exp_type_name, exp_balance)) in
        accounts3.iter().zip(expected_sorted.iter())
    {
        assert_eq!(acc.id, *exp_id);
        assert_eq!(acc.name.as_str(), exp_name.as_str());
        assert_eq!(type_str(&acc.account_type), exp_type_name.as_str());
        assert_eq!(acc.initial_balance_cents, *exp_balance);
    }
    let snapshot_reopen = snapshot(&observer_conn2);
    assert_eq!(snapshot_after_create, snapshot_reopen);
    assert_eq!(schema_after_create, schema(&observer_conn2));
}
