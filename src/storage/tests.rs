use super::{AccountStorage, Storage};
use crate::models::Account;
use crate::types::Monetary;
use anyhow::{anyhow, Result};
use std::str::FromStr;

#[test]
fn test_storage_basic_load_and_save_operations() -> Result<()> {
    let storage = AccountStorage::new();
    
    assert!(storage.load(99).is_none());

    let mut account = Account::new(1);
    account.available = Monetary::from_str("100.0")?;
    storage.save(1, account);
    
    let retrieved_account = storage.load(1).ok_or_else(|| anyhow!("Account not found in storage"))?;
    
    assert_eq!(retrieved_account.account_id, 1);
    assert_eq!(retrieved_account.available.to_string(), "100.0000");

    Ok(())
}

#[test]
fn test_storage_iterator_collects_all_accounts() {
    let storage = AccountStorage::new();
    storage.save(1, Account::new(1));
    storage.save(2, Account::new(2));
    storage.save(3, Account::new(3));

    assert_eq!(storage.iter().count(), 3);
}

#[test]
fn test_storage_enforces_correct_overwrite_semantics() -> Result<()> {
    let storage = AccountStorage::new();
    
    let mut account_version_1 = Account::new(1);
    account_version_1.available = Monetary::from_str("10.0")?;
    storage.save(1, account_version_1);
    
    let mut account_version_2 = storage.load(1).ok_or_else(|| anyhow!("Account v1 missing"))?;
    account_version_2.available = Monetary::from_str("20.0")?;
    storage.save(1, account_version_2);
    
    let final_account = storage.load(1).ok_or_else(|| anyhow!("Final account version missing"))?;

    assert_eq!(final_account.available.to_string(), "20.0000");

    Ok(())
}