use super::AccountActor;
use crate::models::{Transaction, TransactionType};
use crate::storage::{AccountStorage, Storage};
use crate::types::{AccountId, Monetary};
use anyhow::{anyhow, Result};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Helper to create a transaction easily.
fn create_transaction(transaction_type: TransactionType, transaction_id: u32, account_id: u16, amount: &str) -> Result<Transaction> {
    Ok(Transaction {
        transaction_type,
        transaction_id,
        account_id,
        amount: Some(Monetary::from_str(amount)?)
    })
}

/// A test harness to simplify interacting with an AccountActor.
/// Encapsulates the wiring of command and guard channels.
struct ActorHarness {
    sender: Option<mpsc::UnboundedSender<Transaction>>,
    guard_sender: Option<mpsc::Sender<()>>,
    guard_receiver: Option<mpsc::Receiver<()>>
}

impl ActorHarness {
    fn new(account_id: AccountId, storage: Arc<AccountStorage>) -> Self {
        let (guard_sender, guard_receiver) = mpsc::channel(1);
        let sender = AccountActor::spawn(account_id, storage, guard_sender.clone());

        Self {
            sender: Some(sender),
            guard_sender: Some(guard_sender),
            guard_receiver: Some(guard_receiver)
        }
    }

    fn send(&self, transaction: Transaction) -> Result<()> {
        if let Some(sender) = &self.sender {
            sender.send(transaction).map_err(|_| anyhow!("Failed to send transaction"))
        } else {
            Err(anyhow!("Sender already dropped"))
        }
    }

    async fn shutdown(mut self) {
        drop(self.sender.take());
        drop(self.guard_sender.take());

        if let Some(mut reciever) = self.guard_receiver.take() {
            reciever.recv().await;
        }
    }
}

#[tokio::test]
async fn test_actor_isolation_and_storage_persistence() -> Result<()> {
    let storage = Arc::new(AccountStorage::new());
    let client1 = ActorHarness::new(1, storage.clone());
    let client2 = ActorHarness::new(2, storage.clone());

    client1.send(create_transaction(TransactionType::Deposit, 1, 1, "100.0")?)?;
    client2.send(create_transaction(TransactionType::Deposit, 2, 2, "200.0")?)?;
    client1.send(create_transaction(TransactionType::Withdrawal, 3, 1, "50.0")?)?;

    client1.shutdown().await;
    client2.shutdown().await;

    let account1 = storage.load(1).ok_or_else(|| anyhow!("Account 1 missing"))?;
    let account2 = storage.load(2).ok_or_else(|| anyhow!("Account 2 missing"))?;

    assert_eq!(account1.available.to_string(), "50.0000");
    assert_eq!(account2.available.to_string(), "200.0000");

    Ok(())
}

#[tokio::test]
async fn test_actor_maintains_resilience_to_business_logic_errors() -> Result<()> {
    let storage = Arc::new(AccountStorage::new());
    let client = ActorHarness::new(1, storage.clone());

    client.send(create_transaction(TransactionType::Deposit, 1, 1, "10.0")?)?;
    client.send(create_transaction(TransactionType::Withdrawal, 2, 1, "999.0")?)?;
    client.send(create_transaction(TransactionType::Deposit, 3, 1, "20.0")?)?;

    client.shutdown().await;

    let account = storage.load(1).ok_or_else(|| anyhow!("Account missing"))?;

    assert_eq!(account.available.to_string(), "30.0000");

    Ok(())
}