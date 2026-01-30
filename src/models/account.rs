use crate::models::errors::AccountError;
use crate::models::{DisputeStatus, Transaction, TransactionType};
use crate::types::{AccountId, Monetary, TransactionId};
use std::collections::HashMap;

/// Represents the state of a single client account.
///
/// This struct manages the balance (available and held), lock status, and
/// history of the ledger and disputes required for correct transaction processing.
#[derive(Debug, Clone)]
pub struct Account {
    /// The unique identifier for the client.
    pub account_id: AccountId,
    /// Funds available for withdrawal or trading.
    pub available: Monetary,
    /// Funds held due to active disputes.
    pub held: Monetary,
    /// Whether the account is frozen (due to a chargeback).
    pub locked: bool,
    /// History of all successful deposits, mapped by transaction ID.
    /// Used to reference the amount during disputes.
    ledger: HashMap<TransactionId, Monetary>,
    /// Status of active or past disputes, mapped by transaction ID.
    disputes: HashMap<TransactionId, DisputeStatus>
}

impl Account {
    /// Creates a new, empty account for the given client ID.
    pub fn new(account_id: AccountId) -> Self {
        Self {
            account_id,
            available: Monetary::new(),
            held: Monetary::new(),
            locked: false,
            ledger: HashMap::new(),
            disputes: HashMap::new()
        }
    }

    /// Applies a single transaction to the account state.
    ///
    /// This is the main entry point for business logic. It dispatches to specific
    /// handlers based on the transaction type and enforces the global "locked" check.
    ///
    /// # Errors
    /// Returns `AccountError` if:
    /// - The account is locked.
    /// - Insufficient funds for withdrawal.
    /// - Duplicate transaction ID.
    /// - Referenced transaction for dispute/resolve/chargeback is missing.
    pub fn apply(&mut self, transaction: &Transaction) -> Result<(), AccountError> {
        if self.locked {
            return Err(AccountError::account_locked(transaction))
        }

        match transaction.transaction_type {
            TransactionType::Deposit => self.deposit(transaction),
            TransactionType::Withdrawal => self.withdrawal(transaction),
            TransactionType::Dispute => self.dispute(transaction),
            TransactionType::Resolve => self.resolve(transaction),
            TransactionType::Chargeback => self.chargeback(transaction)
        }
    }

    /// Calculates the total funds (available + held).
    pub fn total(&self) -> Monetary {
        let mut total = self.available;
        total += self.held;
        total
    }

    fn deposit(&mut self, transaction: &Transaction) -> Result<(), AccountError> {
        if self.ledger.contains_key(&transaction.transaction_id) {
            return Err(AccountError::duplicate_transaction(transaction))
        }

        let Some(amount) = transaction.amount else {
            return Err(AccountError::amount_required(transaction))
        };

        if amount.is_negative() {
            return Err(AccountError::negative_amount(transaction))
        }

        self.available = self.available.checked_add(amount)
            .ok_or_else(|| AccountError::overflow(transaction))?;
            
        self.ledger.insert(transaction.transaction_id, amount);

        Ok(())
    }

    fn withdrawal(&mut self, transaction: &Transaction) -> Result<(), AccountError> {
        let Some(amount) = transaction.amount else {
            return Err(AccountError::amount_required(transaction))
        };

        if amount.is_negative() {
            return Err(AccountError::negative_amount(transaction))
        }

        if self.available < amount {
            return Err(AccountError::insufficient_funds(transaction))
        }

        self.available = self.available.checked_sub(amount)
            .ok_or_else(|| AccountError::overflow(transaction))?;

        Ok(())
    }

    fn dispute(&mut self, transaction: &Transaction) -> Result<(), AccountError> {
        if self.disputes.contains_key(&transaction.transaction_id) {
            return Err(AccountError::duplicate_dispute(transaction))
        }

        let amount = self.get_deposit(transaction)?;

        self.available = self.available.checked_sub(amount)
            .ok_or_else(|| AccountError::overflow(transaction))?;
            
        self.held = self.held.checked_add(amount)
            .ok_or_else(|| AccountError::overflow(transaction))?;
            
        self.disputes.insert(transaction.transaction_id, DisputeStatus::InProgress);

        Ok(())
    }

    fn resolve(&mut self, transaction: &Transaction) -> Result<(), AccountError> {
        self.check_dispute_in_progress(transaction)?;
        let amount = self.get_deposit(transaction)?;

        self.available = self.available.checked_add(amount)
            .ok_or_else(|| AccountError::overflow(transaction))?;
            
        self.held = self.held.checked_sub(amount)
            .ok_or_else(|| AccountError::overflow(transaction))?;
            
        self.disputes.insert(transaction.transaction_id, DisputeStatus::Resolved);

        Ok(())
    }

    fn chargeback(&mut self, transaction: &Transaction) -> Result<(), AccountError> {
        self.check_dispute_in_progress(transaction)?;
        let amount = self.get_deposit(transaction)?;

        self.held = self.held.checked_sub(amount)
            .ok_or_else(|| AccountError::overflow(transaction))?;
            
        self.locked = true;
        self.disputes.insert(transaction.transaction_id, DisputeStatus::Chargeback);

        Ok(())
    }

    fn get_deposit(&self, transaction: &Transaction) -> Result<Monetary, AccountError> {
        self.ledger.get(&transaction.transaction_id).copied()
            .ok_or_else(|| AccountError::transaction_not_found(transaction))
    }

    fn check_dispute_in_progress(&self, transaction: &Transaction) -> Result<(), AccountError> {
        let status = self.disputes.get(&transaction.transaction_id)
            .ok_or_else(|| AccountError::dispute_not_found(transaction))?;

        if *status != DisputeStatus::InProgress {
            return Err(AccountError::dispute_not_in_progress(transaction));
        }

        Ok(())
    }
}
