use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::{create_dir_all, File};
use std::io::{self, stdout, Write};
use std::path::Path;

use rand::seq::{IteratorRandom, SliceRandom};
use rand::Rng;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;

const PROBABILITY_DEPOSIT: f64 = 0.49;
const PROBABILITY_WITHDRAWAL: f64 = 0.49;
const PROBABILITY_DISPUTE: f64 = 0.005;
const PROBABILITY_RESOLVE: f64 = 0.004;
const PROBABILITY_CHARGEBACK: f64 = 0.001;

const INVALID_TX_ID_DISPUTE: u32 = 99_999_999;
const INVALID_TX_ID_RESOLVE: u32 = 88_888_888;
const INVALID_TX_ID_CHARGEBACK: u32 = 77_777_777;

struct GeneratorConfig {
    num_records: usize,
    num_clients: usize,
    output_path: String,
}

impl GeneratorConfig {
    fn from_args() -> Self {
        let args: Vec<String> = env::args().collect();
        let num_records = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1_000_000);
        let num_clients = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(u16::MAX as usize);
        
        Self {
            num_records,
            num_clients,
            output_path: "samples/stress_test.csv".to_string(),
        }
    }
}

fn main() -> io::Result<()> {
    let config = GeneratorConfig::from_args();

    println!(
        "Generating {} transactions for {} clients in {}...",
        config.num_records, config.num_clients, config.output_path
    );

    if let Some(parent) = Path::new(&config.output_path).parent() {
        create_dir_all(parent)?;
    }
    
    let file = File::create(&config.output_path)?;
    let mut writer = io::BufWriter::new(file);

    writeln!(writer, "type,client,tx,amount")?;

    let mut rng = rand::thread_rng();
    let mut client_deposit_history: HashMap<u16, Vec<u32>> = HashMap::with_capacity(config.num_clients);
    let mut client_active_disputes: HashMap<u16, HashSet<u32>> = HashMap::with_capacity(config.num_clients);

    for tx_id in 1..=config.num_records as u32 {
        let client_id = rng.gen_range(1..=config.num_clients as u16);
        let roll: f64 = rng.r#gen();

        if roll < PROBABILITY_DEPOSIT {
            generate_deposit(&mut writer, &mut rng, client_id, tx_id, &mut client_deposit_history)?;
        } else if roll < PROBABILITY_DEPOSIT + PROBABILITY_WITHDRAWAL {
            generate_withdrawal(&mut writer, &mut rng, client_id, tx_id)?;
        } else if roll < PROBABILITY_DEPOSIT + PROBABILITY_WITHDRAWAL + PROBABILITY_DISPUTE {
            generate_dispute(&mut writer, &mut rng, client_id, &client_deposit_history, &mut client_active_disputes)?;
        } else if roll < PROBABILITY_DEPOSIT + PROBABILITY_WITHDRAWAL + PROBABILITY_DISPUTE + PROBABILITY_RESOLVE {
            generate_resolve(&mut writer, &mut rng, client_id, &mut client_active_disputes)?;
        } else if roll < PROBABILITY_DEPOSIT + PROBABILITY_WITHDRAWAL + PROBABILITY_DISPUTE + PROBABILITY_RESOLVE + PROBABILITY_CHARGEBACK {
            generate_chargeback(&mut writer, &mut rng, client_id, &mut client_active_disputes)?;
        } else {
            generate_invalid_record(&mut writer, &mut rng, client_id, tx_id)?;
        }

        if tx_id % 100_000 == 0 {
            print!(".");
            stdout().flush()?;
        }
    }

    println!("\nGeneration complete.");

    Ok(())
}

fn generate_random_amount<R: Rng>(rng: &mut R, max: f64) -> Decimal {
    let amount_val = if rng.gen_bool(0.05) {
        rng.gen_range(-1000.0..-0.0001)
    } else {
        rng.gen_range(0.0001..max)
    };

    Decimal::from_f64(amount_val).unwrap().round_dp(4)
}

fn generate_deposit<W: Write, R: Rng>(writer: &mut W, rng: &mut R, client_id: u16, tx_id: u32, history: &mut HashMap<u16, Vec<u32>>) -> io::Result<()> {
    let amount = generate_random_amount(rng, 10000.0);
    writeln!(writer, "deposit,{},{},{}", client_id, tx_id, amount)?;

    if amount > Decimal::ZERO {
        history.entry(client_id).or_default().push(tx_id);
    }

    Ok(())
}

fn generate_withdrawal<W: Write, R: Rng>(writer: &mut W, rng: &mut R, client_id: u16, tx_id: u32) -> io::Result<()> {
    let amount = generate_random_amount(rng, 5000.0);
    writeln!(writer, "withdrawal,{},{},{}", client_id, tx_id, amount)?;

    Ok(())
}

fn generate_dispute<W: Write, R: Rng>(writer: &mut W, rng: &mut R, client_id: u16, history: &HashMap<u16, Vec<u32>>, active_disputes: &mut HashMap<u16, HashSet<u32>>) -> io::Result<()> {
    let target_tx = history.get(&client_id)
        .and_then(|h| h.choose(rng).copied())
        .unwrap_or(INVALID_TX_ID_DISPUTE);

    writeln!(writer, "dispute,{},{},", client_id, target_tx)?;
    
    if target_tx != INVALID_TX_ID_DISPUTE {
        active_disputes.entry(client_id).or_default().insert(target_tx);
    }

    Ok(())
}

fn pick_and_remove_dispute<R: Rng>(rng: &mut R, client_id: u16, active_disputes: &mut HashMap<u16, HashSet<u32>>) -> Option<u32> {
    let disputes = active_disputes.get_mut(&client_id)?;
    let chosen = *disputes.iter().choose(rng)?;
    disputes.remove(&chosen);

    Some(chosen)
}

fn generate_resolve<W: Write, R: Rng>(writer: &mut W, rng: &mut R, client_id: u16, active_disputes: &mut HashMap<u16, HashSet<u32>>) -> io::Result<()> {
    let target_tx = pick_and_remove_dispute(rng, client_id, active_disputes).unwrap_or(INVALID_TX_ID_RESOLVE);
    writeln!(writer, "resolve,{},{},", client_id, target_tx)?;

    Ok(())
}

fn generate_chargeback<W: Write, R: Rng>(writer: &mut W, rng: &mut R, client_id: u16, active_disputes: &mut HashMap<u16, HashSet<u32>>) -> io::Result<()> {
    let target_tx = pick_and_remove_dispute(rng, client_id, active_disputes).unwrap_or(INVALID_TX_ID_CHARGEBACK);
    writeln!(writer, "chargeback,{},{},", client_id, target_tx)?;

    Ok(())
}

fn generate_invalid_record<W: Write, R: Rng>(writer: &mut W, rng: &mut R, client_id: u16, tx_id: u32) -> io::Result<()> {
    let invalid_types = [
        format!("deposit,bad_id,{}", tx_id),
        format!("withdrawal,{},bad_tx", client_id),
        format!("junk,{},{},10.0", client_id, tx_id),
        format!("withdrawal,{},{},", client_id, tx_id),
        format!("deposit,{},{}", client_id, tx_id),
        format!("deposit,{},{},10.0,extra", client_id, tx_id),
        format!(" ,{},{}, ", client_id, tx_id),
        "junk,bad_id,bad_tx,junk".to_string()
    ];

    let record = invalid_types.choose(rng).unwrap();
    writeln!(writer, "{}", record)?;

    Ok(())
}
