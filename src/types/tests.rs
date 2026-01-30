use super::Monetary;
use anyhow::Result;
use std::str::FromStr;

#[test]
fn test_monetary_successfully_parses_valid_strings() -> Result<()> {
    let test_cases = vec![
        ("1.0", "1.0000"),
        ("1.1234", "1.1234"),
        ("0.0001", "0.0001"),
        ("-1.5", "-1.5000"),
        ("  1.0  ", "1.0000"),
        ("-0.01", "-0.0100"),
        ("-0.0001", "-0.0001"),
        ("+1.0", "1.0000"),
        ("100", "100.0000"),
        ("1.", "1.0000"),
    ];

    for (input_string, expected_output) in test_cases {
        assert_eq!(Monetary::from_str(input_string)?.to_string(), expected_output);
    }

    Ok(())
}

#[test]
fn test_monetary_fails_to_parse_invalid_strings() {
    assert!(Monetary::from_str("1.12345").is_err()); 
    assert!(Monetary::from_str("abc").is_err());
    assert!(Monetary::from_str("1.2.3").is_err());
    assert!(Monetary::from_str("").is_err());
    assert!(Monetary::from_str(".5").is_err()); 
}

#[test]
fn test_monetary_supports_basic_addition_and_subtraction() -> Result<()> {
    let mut monetary_value_1 = Monetary::from_str("1.5")?;
    let monetary_value_2 = Monetary::from_str("2.5")?;
    monetary_value_1 += monetary_value_2;

    assert_eq!(monetary_value_1.to_string(), "4.0000");

    monetary_value_1 -= Monetary::from_str("5.0")?;

    assert_eq!(monetary_value_1.to_string(), "-1.0000");

    Ok(())
}

#[test]
fn test_monetary_provides_overflow_protection_for_large_values() -> Result<()> {
    let large_value = "922337203685477.0000";
    let mut monetary_value = Monetary::from_str(large_value)?;
    
    monetary_value += Monetary::from_str("0.5")?;

    assert_eq!(monetary_value.to_string(), "922337203685477.5000");

    let previous_value = monetary_value.to_string();
    monetary_value += Monetary::from_str("1.0")?;

    assert_eq!(monetary_value.to_string(), previous_value);
    
    Ok(())
}
