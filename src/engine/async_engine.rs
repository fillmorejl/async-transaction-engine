use crate::actors::AccountActor;
use crate::models::Transaction;
use crate::storage::Storage;
use crate::types::AccountId;
use anyhow::Result;
use csv::{ReaderBuilder, Trim};
use moka::future::Cache;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::{spawn_blocking, JoinHandle};
use tracing::{debug, error};

/// High-performance async transaction processing engine.
pub struct AsyncEngine<S: Storage> {
    storage: Arc<S>,
    backpressure: usize,
    cache_capacity: u64,
    cache_timeout: Duration,
}

#[allow(dead_code)]
impl<S: Storage> AsyncEngine<S> {
    pub fn new(storage: Arc<S>) -> Self {
        Self {
            storage,
            backpressure: 256,
            cache_capacity: 5000,
            cache_timeout: Duration::from_mins(5)
        }
    }

    pub fn with_cache_capacity(mut self, capacity: u64) -> Self {
        self.cache_capacity = capacity;
        self
    }

    pub fn with_cache_timeout(mut self, timeout: Duration) -> Self {
        self.cache_timeout = timeout;
        self
    }

    /// Orchestrates the end-to-end transaction processing pipeline for a CSV file.
    pub async fn run(&self, path: &str) -> Result<()> {
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

    async fn process_transactions(&self, mut receiver: mpsc::Receiver<Transaction>) -> Result<()> {
        let (guard_sender, mut guard_receiver) = mpsc::channel::<()>(1);

        let cache: Cache<AccountId, UnboundedSender<Transaction>> = Cache::builder()
            .time_to_idle(self.cache_timeout)
            .max_capacity(self.cache_capacity)
            .eviction_listener(|key, _value, cause| {
                debug!("Actor for client [{key:?}] was despawned for reason: [{cause:?}]");
            })
            .build();

        //NOTE: In a production system, this loop mimics a Kafka consumer stream. Partitioning by an agreed upon ID ensures strict ordering per client.
        while let Some(transaction) = receiver.recv().await {

            let sender = cache.get_with(transaction.account_id, async {
                AccountActor::spawn(transaction.account_id, self.storage.clone(), guard_sender.clone())
            }).await;

            if sender.send(transaction.clone()).is_err() {
                error!("Account actor for client [{}] could not accept transaction [{}]", transaction.account_id, transaction.transaction_id);
            }
        }

        // Invalidate and process pending tasks to trigger cache evictions
        cache.invalidate_all();
        cache.run_pending_tasks().await;
        
        // Explicitly drop the cache to cause all actors to drop their guard sender
        drop(cache);

        drop(guard_sender);
        guard_receiver.recv().await;

        Ok(())
    }
}
