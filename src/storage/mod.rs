mod account_storage;
#[cfg(test)]
mod tests;

use crate::models::Account;
use crate::types::AccountId;

pub use account_storage::AccountStorage;

pub trait Storage: Send + Sync + 'static {
    fn load(&self, account_id: AccountId) -> Option<Account>;
    fn save(&self, account_id: AccountId, account: Account);
}
