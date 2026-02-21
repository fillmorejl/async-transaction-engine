use super::AsyncEngine;

use anyhow::{anyhow, Result};
use std::fs;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;

use tempfile::NamedTempFile;
use tokio::time::sleep;

use crate::storage::{AccountStorage, Storage};

fn create_temporary_csv(transactions: &[(&str, u16, u32, &str)]) -> Result<NamedTempFile> {
    let mut file = NamedTempFile::new()?;

    writeln!(file, "type,client,tx,amount")?;

    for (kind, client, tx, amount) in transactions {
        if amount.is_empty() {
            writeln!(file, "{},{},{},", kind, client, tx)?;
        } else {
            writeln!(file, "{},{},{},{}", kind, client, tx, amount)?;
        }
    }

    Ok(file)
}

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

    assert_eq!(account.available.to_string(), "160.0000");
    assert_eq!(account.held.to_string(), "0.0000");
    assert!(!account.locked);
    
    Ok(())
}

#[tokio::test]
async fn test_cache_capacity_eviction() -> Result<()> {
    // Scenario: Capacity 2. Send 3 clients. One must be evicted.
    // Confirms: That capacity based auto-eviction works after capacity is met.

    let storage = Arc::new(AccountStorage::new());
    let engine = AsyncEngine::new(storage.clone())
        .with_cache_capacity(2);

    let file = create_temporary_csv(&[
        ("deposit", 1, 1, "10.0"),
        ("deposit", 2, 2, "10.0"),
        ("deposit", 3, 3, "10.0"), // Should trigger eviction of 1 or 2
        ("deposit", 1, 4, "10.0")  // Should hydrate 1 again
    ])?;

    engine.run(file.path().to_str().unwrap()).await?;

    assert_eq!(storage.load(1).unwrap().available.to_string(), "20.0000");
    assert_eq!(storage.load(2).unwrap().available.to_string(), "10.0000");
    assert_eq!(storage.load(3).unwrap().available.to_string(), "10.0000");

    Ok(())
}

#[tokio::test]
async fn test_cache_time_eviction() -> Result<()> {
    // Scenario: Timeout 100ms (auto-eviction on next access). Send 1. Sleep 200ms. Send 1 again.
    // Confirms: Re-hydration works after time-based auto-eviction.

    let storage = Arc::new(AccountStorage::new());
    let engine = AsyncEngine::new(storage.clone())
        .with_cache_timeout(Duration::from_millis(100));

    let file1 = create_temporary_csv(&[
        ("deposit", 1, 1, "10.0")
    ])?;

    engine.run(file1.path().to_str().unwrap()).await?;

    sleep(Duration::from_millis(200)).await;

    let file2 = create_temporary_csv(&[
        ("deposit", 1, 2, "20.0")
    ])?;

    engine.run(file2.path().to_str().unwrap()).await?;

    assert_eq!(storage.load(1).unwrap().available.to_string(), "30.0000");

    Ok(())
}
