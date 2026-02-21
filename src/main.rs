mod actors;
mod engine;
mod models;
mod storage;
mod types;

use std::io::{stderr, stdout, BufWriter, Write};
use std::process::exit;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, Layer};

use crate::engine::AsyncEngine;
use crate::storage::AccountStorage;

#[tokio::main]
async fn main() -> Result<()> {
    //NOTE: If I was making a much more sophisticated CLI application, I would have used the clap crate
    //      to handle the CLI parsing and execution.
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: async-transaction-engine [input].csv [log_level:optional] > [output].csv");
        eprintln!("Available log levels: error, warn, info, debug, trace (default: error)");
        exit(1);
    }

    let path = &args[1];
    let log_level = args.get(2)
        .map(|s| parse_log_level(s)).unwrap_or_else(|| LevelFilter::ERROR);

    setup_logging(log_level);

    let storage = Arc::new(AccountStorage::new());
    let engine = AsyncEngine::new(storage.clone());
    
    let timer = Instant::now();
    engine.run(path).await?;
    let duration = timer.elapsed();

    info!("Processed transactions in: {duration:?}");
    
    write_results_to_stdout(storage)?;

    Ok(())
}

fn parse_log_level(level: &str) -> LevelFilter {
    match level.to_lowercase().as_str() {
        "trace" => LevelFilter::TRACE,
        "debug" => LevelFilter::DEBUG,
        "info" => LevelFilter::INFO,
        "warn" => LevelFilter::WARN,
        "error" => LevelFilter::ERROR,
        _ => {
            eprintln!("Invalid log level '{}', defaulting to 'error'", level);
            LevelFilter::ERROR
        }
    }
}

fn setup_logging(level: LevelFilter) {
    //NOTE: Because we are doing stdout redirection, we will need to utilize stderr to display logging
    let terminal_log = fmt::layer()
        .with_target(false)
        .with_writer(stderr)
        .with_filter(level);

    tracing_subscriber::registry()
        .with(terminal_log)
        .init();
}

fn write_results_to_stdout(storage: Arc<AccountStorage>) -> Result<()> {
    let mut output = BufWriter::new(stdout().lock());

    writeln!(output, "client,available,held,total,locked")?;
    
    for account in storage.iter() {
        writeln!(
            output, 
            "{},{},{},{},{}", 
            account.account_id, 
            account.available, 
            account.held, 
            account.total(), 
            account.locked
        )?;
    }

    output.flush()?;

    Ok(())
}
