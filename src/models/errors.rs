use thiserror::Error;

use crate::models::{Transaction, TransactionType};
use crate::types::{AccountId, TransactionId};

#[derive(Debug, Error)]
pub enum AccountError {
    #[error("Account is locked for client [{account_id}]")]
    AccountLocked {
        account_id: AccountId
    },
    #[error("Duplicate transaction [{transaction_id}]:[{transaction_type:?}] for client [{account_id}]")]
    DuplicateTransaction {
        account_id: AccountId,
        transaction_id: TransactionId,
        transaction_type: TransactionType
    },
    #[error("Duplicate dispute for transaction [{transaction_id}]:[{transaction_type:?}] for client [{account_id}]")]
    DuplicateDispute {
        account_id: AccountId,
        transaction_id: TransactionId,
        transaction_type: TransactionType
    },
    #[error("Transaction [{transaction_id}]:[{transaction_type:?}] was not found for client [{account_id}]")]
    TransactionNotFound {
        account_id: AccountId,
        transaction_id: TransactionId,
        transaction_type: TransactionType
    },
    #[error("Dispute for transaction [{transaction_id}]:[{transaction_type:?}] was not found for client [{account_id}]")]
    DisputeNotFound {
        account_id: AccountId,
        transaction_id: TransactionId,
        transaction_type: TransactionType
    },
    #[error("Amount is required for transaction [{transaction_id}]:[{transaction_type:?}] for client [{account_id}]")]
    AmountRequired {
        account_id: AccountId,
        transaction_id: TransactionId,
        transaction_type: TransactionType
    },
    #[error("Insufficient funds for transaction [{transaction_id}]:[{transaction_type:?}] for client [{account_id}]")]
    InsufficientFunds {
        account_id: AccountId,
        transaction_id: TransactionId,
        transaction_type: TransactionType
    },
    #[error("Dispute for transaction [{transaction_id}]:[{transaction_type:?}] is not in progress for client [{account_id}]")]
    DisputeNotInProgress {
        account_id: AccountId,
        transaction_id: TransactionId,
        transaction_type: TransactionType
    },
    #[error("Amount must be positive for transaction [{transaction_id}]:[{transaction_type:?}] for client [{account_id}]")]
    NegativeAmount {
        account_id: AccountId,
        transaction_id: TransactionId,
        transaction_type: TransactionType
    },
    #[error("Numeric overflow occurred for transaction [{transaction_id}]:[{transaction_type:?}] for client [{account_id}]")]
    Overflow {
        account_id: AccountId,
        transaction_id: TransactionId,
        transaction_type: TransactionType
    }
}

impl AccountError {
    //NOTE: I know this is not seen in Rust as often but I did not like seeing every error type take the same parameters making
    //      the code highly verbose and more difficult to read.  In my past life using C# I came across many patterns, the factory
    //      patten made sense to make these specific errors easier to instantiate and use.

    pub fn account_locked(transaction: &Transaction) -> Self {
        Self::AccountLocked { account_id: transaction.account_id }
    }

    pub fn duplicate_transaction(transaction: &Transaction) -> Self {
        Self::DuplicateTransaction {
            account_id: transaction.account_id,
            transaction_id: transaction.transaction_id,
            transaction_type: transaction.transaction_type,
        }
    }

    pub fn duplicate_dispute(transaction: &Transaction) -> Self {
        Self::DuplicateDispute {
            account_id: transaction.account_id,
            transaction_id: transaction.transaction_id,
            transaction_type: transaction.transaction_type,
        }
    }

    pub fn transaction_not_found(transaction: &Transaction) -> Self {
        Self::TransactionNotFound {
            account_id: transaction.account_id,
            transaction_id: transaction.transaction_id,
            transaction_type: transaction.transaction_type,
        }
    }

    pub fn dispute_not_found(transaction: &Transaction) -> Self {
        Self::DisputeNotFound {
            account_id: transaction.account_id,
            transaction_id: transaction.transaction_id,
            transaction_type: transaction.transaction_type,
        }
    }

    pub fn amount_required(transaction: &Transaction) -> Self {
        Self::AmountRequired {
            account_id: transaction.account_id,
            transaction_id: transaction.transaction_id,
            transaction_type: transaction.transaction_type,
        }
    }

    pub fn insufficient_funds(transaction: &Transaction) -> Self {
        Self::InsufficientFunds {
            account_id: transaction.account_id,
            transaction_id: transaction.transaction_id,
            transaction_type: transaction.transaction_type,
        }
    }

    pub fn dispute_not_in_progress(transaction: &Transaction) -> Self {
        Self::DisputeNotInProgress {
            account_id: transaction.account_id,
            transaction_id: transaction.transaction_id,
            transaction_type: transaction.transaction_type,
        }
    }

    pub fn negative_amount(transaction: &Transaction) -> Self {
        Self::NegativeAmount {
            account_id: transaction.account_id,
            transaction_id: transaction.transaction_id,
            transaction_type: transaction.transaction_type,
        }
    }

    pub fn overflow(transaction: &Transaction) -> Self {
        Self::Overflow {
            account_id: transaction.account_id,
            transaction_id: transaction.transaction_id,
            transaction_type: transaction.transaction_type,
        }
    }
}
