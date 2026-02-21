use std::sync::Arc;

use tokio::spawn;
use tokio::sync::mpsc;
use tracing::{debug, warn};

use crate::models::{Account, Transaction};
use crate::storage::Storage;
use crate::types::AccountId;

pub struct AccountActor;

impl AccountActor {
    /// Spawns a new actor and returns its input channel.
    pub fn spawn<S: Storage>(account_id: AccountId, storage: Arc<S>, guard_sender: mpsc::Sender<()>) -> mpsc::UnboundedSender<Transaction> {
        let (sender, mut receiver) = mpsc::unbounded_channel();
        
        spawn(async move {
            let mut account = storage.load(account_id)
                .unwrap_or_else(|| Account::new(account_id));

            while let Some(transaction) = receiver.recv().await {
                match account.apply(&transaction) {
                    Ok(_) => {
                        //NOTE: If using Kafka in production you can consider commiting the message (transaction)
                        debug!("Transaction [{}]:[{:?}] for client [{}] processed", transaction.transaction_id, transaction.transaction_type, transaction.account_id);
                    },
                    Err(error) => {
                        //NOTE: None of the current errors are critical, if using Kafka in production you can consider commiting the message (transaction)
                        warn!("{error}");
                    }
                }
            }

            storage.save(account_id, account);

            drop(guard_sender);
        });
        
        sender
    }
}