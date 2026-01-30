use super::{Account, Transaction, TransactionType};
use crate::models::errors::AccountError;
use crate::types::{AccountId, Monetary, TransactionId};
use anyhow::Result;
use std::str::FromStr;

fn create_transaction(transaction_type: TransactionType, transaction_id: TransactionId, account_id: AccountId, amount: Option<&str>) -> Result<Transaction> {
    Ok(Transaction {
        transaction_type,
        transaction_id,
        account_id,
        amount: match amount {
            Some(s) => Some(Monetary::from_str(s)?),
            None => None
        }
    })
}

#[test]
fn test_successful_deposit_updates_balance() -> Result<()> {
    let mut account = Account::new(1);
    let deposit = create_transaction(TransactionType::Deposit, 1, 1, Some("10.0"))?;

    account.apply(&deposit)?;

    assert_eq!(account.available.to_string(), "10.0000");
    assert_eq!(account.total().to_string(), "10.0000");

    Ok(())
}

#[test]
fn test_duplicate_deposit_fails_idempotency() -> Result<()> {
    let mut account = Account::new(1);
    let deposit = create_transaction(TransactionType::Deposit, 1, 1, Some("10.0"))?;

    account.apply(&deposit)?;

    let result = account.apply(&deposit);

    assert!(matches!(result, Err(AccountError::DuplicateTransaction { .. })));
    assert_eq!(account.available.to_string(), "10.0000");

    Ok(())
}

#[test]
fn test_withdrawal_with_exact_funds_succeeds() -> Result<()> {
    let mut account = Account::new(1);
    account.apply(&create_transaction(TransactionType::Deposit, 1, 1, Some("10.0"))?)?;
    
    let withdrawal = create_transaction(TransactionType::Withdrawal, 2, 1, Some("10.0"))?;
    account.apply(&withdrawal)?;
    
    assert_eq!(account.available.to_string(), "0.0000");

    Ok(())
}

#[test]
fn test_withdrawal_with_insufficient_funds_fails() -> Result<()> {
    let mut account = Account::new(1);
    account.apply(&create_transaction(TransactionType::Deposit, 1, 1, Some("10.0"))?)?;
    
    let withdrawal = create_transaction(TransactionType::Withdrawal, 2, 1, Some("10.0001"))?;
    let result = account.apply(&withdrawal);
    
    assert!(matches!(result, Err(AccountError::InsufficientFunds { .. })));
    assert_eq!(account.available.to_string(), "10.0000");

    Ok(())
}

#[test]
fn test_dispute_and_resolve_lifecycle() -> Result<()> {
    let mut account = Account::new(1);
    account.apply(&create_transaction(TransactionType::Deposit, 1, 1, Some("100.0"))?)?;
    account.apply(&create_transaction(TransactionType::Dispute, 1, 1, None)?)?;
    
    assert_eq!(account.available.to_string(), "0.0000");
    assert_eq!(account.held.to_string(), "100.0000");

    account.apply(&create_transaction(TransactionType::Resolve, 1, 1, None)?)?;

    assert_eq!(account.available.to_string(), "100.0000");
    assert_eq!(account.held.to_string(), "0.0000");

    Ok(())
}

#[test]
fn test_dispute_and_chargeback_lifecycle() -> Result<()> {
    let mut account = Account::new(1);
    account.apply(&create_transaction(TransactionType::Deposit, 1, 1, Some("100.0"))?)?;
    account.apply(&create_transaction(TransactionType::Dispute, 1, 1, None)?)?;
    account.apply(&create_transaction(TransactionType::Chargeback, 1, 1, None)?)?;

    assert_eq!(account.available.to_string(), "0.0000");
    assert_eq!(account.held.to_string(), "0.0000");
    assert_eq!(account.total().to_string(), "0.0000");
    assert!(account.locked);

    Ok(())
}

#[test]
fn test_locked_account_rejects_subsequent_transactions() -> Result<()> {
    let mut account = Account::new(1);
    account.locked = true;
    
    let deposit = create_transaction(TransactionType::Deposit, 1, 1, Some("10.0"))?;
    let result = account.apply(&deposit);
    
    assert!(matches!(result, Err(AccountError::AccountLocked { .. })));

    Ok(())
}

#[test]
fn test_dispute_on_non_existent_transaction_fails() -> Result<()> {
    let mut account = Account::new(1);
    let dispute = create_transaction(TransactionType::Dispute, 99, 1, None)?;
    let result = account.apply(&dispute);
    
    assert!(matches!(result, Err(AccountError::TransactionNotFound { .. })));

    Ok(())
}

#[test]
fn test_resolve_on_non_disputed_transaction_fails() -> Result<()> {
    let mut account = Account::new(1);
    account.apply(&create_transaction(TransactionType::Deposit, 1, 1, Some("10.0"))?)?;
    
    let resolve = create_transaction(TransactionType::Resolve, 1, 1, None)?;
    let result = account.apply(&resolve);
    
    assert!(matches!(result, Err(AccountError::DisputeNotFound { .. })));

    Ok(())
}

#[test]
fn test_withdrawal_is_isolated_from_held_funds() -> Result<()> {
    let mut account = Account::new(1);
    account.apply(&create_transaction(TransactionType::Deposit, 1, 1, Some("100.0"))?)?;
    account.apply(&create_transaction(TransactionType::Dispute, 1, 1, None)?)?;
    account.apply(&create_transaction(TransactionType::Deposit, 2, 1, Some("50.0"))?)?;
    
    let withdrawal = create_transaction(TransactionType::Withdrawal, 3, 1, Some("50.0"))?;

    assert!(account.apply(&withdrawal).is_ok());
    assert_eq!(account.available.to_string(), "0.0000");
    assert_eq!(account.held.to_string(), "100.0000");

    Ok(())
}

#[test]
fn test_zero_amount_deposit_is_accepted() -> Result<()> {
    let mut account = Account::new(1);
    let deposit = create_transaction(TransactionType::Deposit, 1, 1, Some("0.0000"))?;
    account.apply(&deposit)?;
    
    assert_eq!(account.available.to_string(), "0.0000");

    Ok(())
}

#[test]
fn test_disputing_a_withdrawal_fails() -> Result<()> {
    let mut account = Account::new(1);
    account.apply(&create_transaction(TransactionType::Deposit, 1, 1, Some("100.0"))?)?;
    account.apply(&create_transaction(TransactionType::Withdrawal, 2, 1, Some("50.0"))?)?;
    
    let dispute = create_transaction(TransactionType::Dispute, 2, 1, None)?;
    let result = account.apply(&dispute);
    
    assert!(matches!(result, Err(AccountError::TransactionNotFound { .. })));
    
    Ok(())
}