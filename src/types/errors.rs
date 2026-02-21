use std::num::ParseIntError;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum MonetaryError {
    #[error("Monetary error: {0}")]
    InvalidFormat(String),
    #[error("Monetary error: {0}")]
    ParseInt(#[from] ParseIntError),
    #[error("Monetary error: Overflow")]
    Overflow
}
