use crate::models::{Account, Transaction};
use crate::storage::Storage;
use crate::types::AccountId;
use anyhow::Result;
use std::sync::Arc;
use tokio::spawn;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{debug, warn};

pub struct AccountActor {
    sender: Option<mpsc::UnboundedSender<Transaction>>,
    handle: Option<JoinHandle<()>>
}

impl AccountActor {
    pub fn new<S: Storage>(account_id: AccountId, storage: Arc<S>) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel::<Transaction>();
        let handle = Self::spawn(account_id, storage, receiver);

        Self {
            sender: Some(sender),
            handle: Some(handle)
        }
    }

    pub fn accept(&self, transaction: &Transaction) -> bool {
        if let Some(sender) = &self.sender {
            sender.send(transaction.clone()).is_ok()
        } else {
            false
        }
    }

    pub async fn despawn(mut self) -> Result<()> {
        if let Some(sender) = self.sender.take() {
            drop(sender);
        }

        let handle = self.handle.take()
            .ok_or_else(|| anyhow::anyhow!("Actor already despawned"))?;

        Ok(handle.await?)
    }

    fn spawn<S: Storage>(account_id: AccountId, storage: Arc<S>, mut receiver: mpsc::UnboundedReceiver<Transaction>) -> JoinHandle<()> {
        spawn(async move {
            let mut account = storage.load(account_id)
                .unwrap_or_else(|| Account::new(account_id));

            while let Some(transaction) = receiver.recv().await {
                match account.apply(&transaction) {
                    Ok(_) => {
                        //NOTE: If using Kafka in production you can consider commiting the message (transaction)
                        debug!("Transaction [{}]:[{:?}] for client [{}] completed successfully", transaction.transaction_id, transaction.transaction_type, transaction.account_id);
                    },
                    Err(error) => {
                        //NOTE: None of the current errors are critical, if using Kafka in production you can consider commiting the message (transaction)
                        warn!("{error}");
                    }
                }
            }

            storage.save(account_id, account);
        })
    }
}
