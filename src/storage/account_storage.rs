use crate::models::Account;
use crate::storage::Storage;
use crate::types::AccountId;
use dashmap::iter::Iter;
use dashmap::DashMap;
use std::sync::Arc;

pub struct AccountStorage {
    cache: Arc<DashMap<AccountId, Account>>
}

impl AccountStorage {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(DashMap::new())
        }
    }

    pub fn iter(&self) -> Iter<'_, AccountId, Account> {
        self.cache.iter()
    }
}

impl Storage for AccountStorage {
    fn load(&self, account_id: AccountId) -> Option<Account> {
        self.cache.remove(&account_id).map(|(_, account)| account)
    }

    fn save(&self, account_id: AccountId, account: Account) {
        self.cache.insert(account_id, account);
    }
}
