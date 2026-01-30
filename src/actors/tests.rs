use super::AccountActor;
use crate::models::{Transaction, TransactionType};
use crate::storage::{AccountStorage, Storage};
use crate::types::Monetary;
use anyhow::{anyhow, Result};
use std::str::FromStr;
use std::sync::Arc;

fn create_transaction(transaction_type: TransactionType, transaction_id: u32, account_id: u16, amount: &str) -> Result<Transaction> {
    Ok(Transaction {
        transaction_type,
        transaction_id,
        account_id,
        amount: Some(Monetary::from_str(amount)?)
    })
}

#[tokio::test]
async fn test_actor_isolation_and_storage_persistence() -> Result<()> {
    let storage = Arc::new(AccountStorage::new());
    
    let actor_client_1 = AccountActor::new(1, storage.clone());
    let actor_client_2 = AccountActor::new(2, storage.clone());

    actor_client_1.accept(&create_transaction(TransactionType::Deposit, 1, 1, "100.0")?);
    actor_client_2.accept(&create_transaction(TransactionType::Deposit, 2, 2, "200.0")?);
    actor_client_1.accept(&create_transaction(TransactionType::Withdrawal, 3, 1, "50.0")?);

    actor_client_1.despawn().await?;
    actor_client_2.despawn().await?;

    let account_client_1 = storage.load(1).ok_or_else(|| anyhow!("Account 1 missing from storage"))?;
    let account_client_2 = storage.load(2).ok_or_else(|| anyhow!("Account 2 missing from storage"))?;

    assert_eq!(account_client_1.available.to_string(), "50.0000");
    assert_eq!(account_client_2.available.to_string(), "200.0000");

    Ok(())
}

#[tokio::test]
async fn test_actor_maintains_resilience_to_business_logic_errors() -> Result<()> {
    let storage = Arc::new(AccountStorage::new());
    let actor = AccountActor::new(1, storage.clone());

    // Valid -> Invalid (Insufficient Funds) -> Valid
    actor.accept(&create_transaction(TransactionType::Deposit, 1, 1, "10.0")?);
    actor.accept(&create_transaction(TransactionType::Withdrawal, 2, 1, "999.0")?); 
    actor.accept(&create_transaction(TransactionType::Deposit, 3, 1, "20.0")?);

    actor.despawn().await?;

    let account = storage.load(1).ok_or_else(|| anyhow!("Account missing from storage"))?;

    assert_eq!(account.available.to_string(), "30.0000");
    
    Ok(())
}