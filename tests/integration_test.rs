use std::process::Command;
use std::path::Path;
use std::collections::HashMap;
use anyhow::{anyhow, Result};

#[test]
fn test_cli_correctly_processes_sample() -> Result<()> {
    let binary_path = env!("CARGO_BIN_EXE_async-transaction-engine");
    let sample_path = Path::new("samples").join("sample.csv");
    
    let output = Command::new(binary_path)
        .arg(sample_path)
        .output()?;
    
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout)?;
    let mut lines = stdout.lines();

    assert_eq!(lines.next(), Some("client,available,held,total,locked"));

    for line in lines {
        let fields: Vec<&str> = line.split(',').collect();

        assert_eq!(fields.len(), 5);

        let _: u16 = fields[0].parse()?;
        let _: f64 = fields[1].parse()?;
        let _: f64 = fields[2].parse()?;
        let _: f64 = fields[3].parse()?;
        let _: bool = fields[4].parse()?;
    }

    Ok(())
}

#[test]
fn test_cli_outputs_correct_final_balances() -> Result<()> {
    let binary_path = env!("CARGO_BIN_EXE_async-transaction-engine");
    let fixture_path = Path::new("samples").join("fixed.csv");
    
    let output = Command::new(binary_path)
        .arg(fixture_path)
        .output()?;
    
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout)?;
    let lines = stdout.lines();
    let mut results = HashMap::new();

    for line in lines.skip(1) {
        let fields: Vec<&str> = line.split(',').collect();
        results.insert(fields[0].to_string(), (fields[1].to_string(), fields[2].to_string(), fields[3].to_string(), fields[4].to_string()));
    }

    let client_1_results = results.get("1").ok_or_else(|| anyhow!("client 1 missing from output"))?;

    assert_eq!(client_1_results.0, "25.0000");
    assert_eq!(client_1_results.1, "0.0000");
    assert_eq!(client_1_results.2, "25.0000");
    assert_eq!(client_1_results.3, "false");

    let client_2_results = results.get("2").ok_or_else(|| anyhow!("client 2 missing from output"))?;

    assert_eq!(client_2_results.0, "100.0000");
    assert_eq!(client_2_results.1, "0.0000");
    assert_eq!(client_2_results.2, "100.0000");
    assert_eq!(client_2_results.3, "false");

    Ok(())
}
