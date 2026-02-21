use super::{Account, Transaction, TransactionType};

use std::str::FromStr;

use anyhow::Result;
use rust_decimal::Decimal;

use crate::models::errors::AccountError;
use crate::types::{AccountId, TransactionId};

fn create_transaction(transaction_type: TransactionType, transaction_id: TransactionId, account_id: AccountId, amount: Option<&str>) -> Result<Transaction> {
    Ok(Transaction {
        transaction_type,
        transaction_id,
        account_id,
        amount: match amount {
            Some(s) => Some(Decimal::from_str(s)?),
            None => None
        }
    })
}

#[test]
fn test_successful_deposit_updates_balance() -> Result<()> {
    let mut account = Account::new(1);
    let deposit = create_transaction(TransactionType::Deposit, 1, 1, Some("10.0"))?;

    account.apply(&deposit)?;

    assert_eq!(account.available, Decimal::from_str("10.0")?);
    assert_eq!(account.total(), Decimal::from_str("10.0")?);

    Ok(())
}

#[test]
fn test_duplicate_deposit_fails_idempotency() -> Result<()> {
    let mut account = Account::new(1);
    let deposit = create_transaction(TransactionType::Deposit, 1, 1, Some("10.0"))?;

    account.apply(&deposit)?;

    let result = account.apply(&deposit);

    assert!(matches!(result, Err(AccountError::DuplicateTransaction { .. })));
    assert_eq!(account.available, Decimal::from_str("10.0")?);

    Ok(())
}

#[test]
fn test_withdrawal_with_exact_funds_succeeds() -> Result<()> {
    let mut account = Account::new(1);
    account.apply(&create_transaction(TransactionType::Deposit, 1, 1, Some("10.0"))?)?;

    let withdrawal = create_transaction(TransactionType::Withdrawal, 2, 1, Some("10.0"))?;
    account.apply(&withdrawal)?;

    assert!(account.available.is_zero());

    Ok(())
}

#[test]
fn test_withdrawal_with_insufficient_funds_fails() -> Result<()> {
    let mut account = Account::new(1);
    account.apply(&create_transaction(TransactionType::Deposit, 1, 1, Some("10.0"))?)?;

    let withdrawal = create_transaction(TransactionType::Withdrawal, 2, 1, Some("10.0001"))?;
    let result = account.apply(&withdrawal);

    assert!(matches!(result, Err(AccountError::InsufficientFunds { .. })));
    assert_eq!(account.available, Decimal::from_str("10.0")?);

    Ok(())
}

#[test]
fn test_dispute_and_resolve_lifecycle() -> Result<()> {
    let mut account = Account::new(1);
    account.apply(&create_transaction(TransactionType::Deposit, 1, 1, Some("100.0"))?)?;
    account.apply(&create_transaction(TransactionType::Dispute, 1, 1, None)?)?;

    assert!(account.available.is_zero());
    assert_eq!(account.held, Decimal::from_str("100.0")?);

    account.apply(&create_transaction(TransactionType::Resolve, 1, 1, None)?)?;

    assert_eq!(account.available, Decimal::from_str("100.0")?);
    assert!(account.held.is_zero());

    Ok(())
}

#[test]
fn test_dispute_and_chargeback_lifecycle() -> Result<()> {
    let mut account = Account::new(1);
    account.apply(&create_transaction(TransactionType::Deposit, 1, 1, Some("100.0"))?)?;
    account.apply(&create_transaction(TransactionType::Dispute, 1, 1, None)?)?;
    account.apply(&create_transaction(TransactionType::Chargeback, 1, 1, None)?)?;

    assert!(account.available.is_zero());
    assert!(account.held.is_zero());
    assert!(account.total().is_zero());
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
    assert!(account.available.is_zero());
    assert_eq!(account.held, Decimal::from_str("100.0")?);

    Ok(())
}

#[test]
fn test_zero_amount_deposit_is_accepted() -> Result<()> {
    let mut account = Account::new(1);
    let deposit = create_transaction(TransactionType::Deposit, 1, 1, Some("0.0000"))?;
    account.apply(&deposit)?;

    assert!(account.available.is_zero());

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
