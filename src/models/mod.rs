mod account;
mod errors;
#[cfg(test)]
mod tests;
mod transaction;

use serde::Deserialize;

pub use account::Account;
pub use transaction::Transaction;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback
}

#[derive(Debug, Eq, PartialEq, Clone)]
enum DisputeStatus {
    InProgress,
    Resolved,
    Chargeback
}
