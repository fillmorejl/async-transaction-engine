use crate::actors::AccountActor;
use crate::models::Transaction;
use crate::storage::AccountStorage;
use crate::types::AccountId;
use csv::{ReaderBuilder, Trim};
use futures::future::join_all;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::{spawn_blocking, JoinHandle};
use tracing::error;

/// High-performance async transaction processing engine.
pub struct AsyncEngine {
    storage: Arc<AccountStorage>,
    backpressure: usize
}

impl AsyncEngine {
    /// Creates a new engine instance with the provided storage.
    pub fn new(storage: Arc<AccountStorage>) -> Self {
        Self {
            storage,
            backpressure: 256
        }
    }

    /// Orchestrates the end-to-end transaction processing pipeline for a CSV file.
    pub async fn run(&self, path: &str) -> anyhow::Result<()> {
        let (sender, receiver) = mpsc::channel::<Transaction>(self.backpressure);
        let csv_handle = self.spawn_csv_reader(path.to_string(), sender);
        let processing_result = self.process_transactions(receiver).await;

        if let Err(error) = csv_handle.await {
            error!("CSV ingestion failed: {error}");
        }

        processing_result
    }

    fn spawn_csv_reader(&self, path: String, sender: mpsc::Sender<Transaction>) -> JoinHandle<()> {
        spawn_blocking(move || {
            let file = match File::open(&path) {
                Ok(file) => file,
                Err(error) => {
                    error!("Error opening CSV at path: {path} | {error}");
                    return;
                }
            };

            let mut reader = ReaderBuilder::new()
                .trim(Trim::All)
                .flexible(true)
                .from_reader(BufReader::new(file));

            for result in reader.deserialize::<Transaction>() {
                match result {
                    Ok(transaction) => {
                        if sender.blocking_send(transaction).is_err() {
                            break;
                        }
                    }
                    Err(error) => {
                        error!("CSV deserialization error: {error}");
                    }
                }
            }
        })
    }

    async fn process_transactions(&self, mut receiver: mpsc::Receiver<Transaction>) -> anyhow::Result<()> {
        let mut actors = HashMap::<AccountId, AccountActor>::new();

        // NOTE: In a production system, this loop mimics a Kafka consumer stream. Partitioning by an agreed upon ID ensures strict ordering per client.
        while let Some(transaction) = receiver.recv().await {
            let actor = actors.entry(transaction.account_id).or_insert_with(|| {
                AccountActor::new(transaction.account_id, self.storage.clone())
            });

            if !actor.accept(&transaction) {
                error!("Account actor for client [{}] could not accept transaction [{}]", transaction.account_id, transaction.transaction_id);
            }
        }

        //NOTE: Provide a graceful shutdown and wait for all actors to finish processing their individual queues
        let despawns = actors.into_values().map(|actor| actor.despawn());

        for result in join_all(despawns).await {
            if let Err(error) = result {
                error!("An account actor did not despawn gracefully: {error:?}");
            }
        }

        Ok(())
    }
}
