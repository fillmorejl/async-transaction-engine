mod errors;
mod monetary;
#[cfg(test)]
mod tests;

pub use monetary::Monetary;

pub type AccountId = u16;
pub type TransactionId = u32;
