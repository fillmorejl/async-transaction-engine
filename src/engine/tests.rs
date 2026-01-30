use super::AsyncEngine;
use crate::storage::AccountStorage;
use anyhow::{anyhow, Result};
use std::fs;
use std::sync::Arc;

#[tokio::test]
async fn test_engine_processes_valid_csv_stream_successfully() -> Result<()> {
    let csv_content = "type,client,tx,amount\ndeposit,1,1,10.0\ndeposit,2,2,20.0\nwithdrawal,1,3,5.0";
    let path = "test_engine_1.csv";
    fs::write(path, csv_content)?;

    let storage = Arc::new(AccountStorage::new());
    let engine = AsyncEngine::new(storage.clone());
    engine.run(path).await?;
    let _ = fs::remove_file(path);

    let mut accounts: Vec<_> = storage.iter().map(|item| item.value().clone()).collect();
    accounts.sort_by_key(|account| account.account_id);

    assert_eq!(accounts[0].available.to_string(), "5.0000");
    assert_eq!(accounts[1].available.to_string(), "20.0000");

    Ok(())
}

#[tokio::test]
async fn test_engine_gracefully_skips_malformed_csv_input() -> Result<()> {
    let csv_content = "type,client,tx,amount\ndeposit,1,1,10.0\ninvalid,data,here,0\ndeposit,1,2,5.0";
    let path = "test_engine_2.csv";
    fs::write(path, csv_content)?;

    let storage = Arc::new(AccountStorage::new());
    let engine = AsyncEngine::new(storage.clone());
    engine.run(path).await?;
    let _ = fs::remove_file(path);

    let accounts: Vec<_> = storage.iter().map(|item| item.value().clone()).collect();

    assert_eq!(accounts[0].available.to_string(), "15.0000");

    Ok(())
}

#[tokio::test]
async fn test_engine_handles_missing_csv_file_without_error() -> Result<()> {
    let storage = Arc::new(AccountStorage::new());
    let engine = AsyncEngine::new(storage.clone());
    // Engine returns Ok but logs an error if the file cannot be opened.
    assert!(engine.run("missing.csv").await.is_ok());
    assert_eq!(storage.iter().count(), 0);
    Ok(())
}

#[tokio::test]
async fn test_engine_correctly_orchestrates_complex_dispute_sequences() -> Result<()> {
    let csv_content = "type,client,tx,amount\ndeposit,1,1,100.0\ndeposit,1,2,50.0\ndispute,1,1,\nresolve,1,1,\nchargeback,1,2,\ndeposit,1,3,10.0";
    let path = "test_engine_3.csv";
    fs::write(path, csv_content)?;

    let storage = Arc::new(AccountStorage::new());
    let engine = AsyncEngine::new(storage.clone());
    engine.run(path).await?;
    let _ = fs::remove_file(path);

    let account = storage.iter().next().ok_or_else(|| anyhow!("Account missing from storage"))?.value().clone();
    
    // Note: Chargeback 2 fails because it was never disputed.
    assert_eq!(account.available.to_string(), "160.0000");
    assert_eq!(account.held.to_string(), "0.0000");
    assert!(!account.locked);
    
    Ok(())
}
