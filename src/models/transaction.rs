use rust_decimal::Decimal;
use serde::Deserialize;

use crate::models::TransactionType;
use crate::types::{AccountId, TransactionId};

/// Represents a single row from the input CSV file.
///
/// This struct captures the raw transaction data before it is applied to an account.
/// The `amount` field is optional because `dispute`, `resolve`, and `chargeback`
/// types do not carry an amount value in the CSV.
#[derive(Debug, Clone, Deserialize)]
pub struct Transaction {
    /// The type of operation (deposit, withdrawal, dispute, etc.)
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    /// Global unique transaction ID.
    #[serde(rename = "tx")]
    pub transaction_id: TransactionId,
    /// The target client ID.
    #[serde(rename = "client")]
    pub account_id: AccountId,
    /// The amount of funds involved (if applicable).
    pub amount: Option<Decimal>
}
