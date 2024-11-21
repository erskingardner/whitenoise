use nostr_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;

use crate::nostr_manager;
use crate::types::EnrichedContact;

const ACCOUNTS_DB_TREE_NAME: &str = "accounts";

#[derive(Error, Debug)]
pub enum AccountError {
    #[error("Account already exists: {0}")]
    AccountAlreadyExists(String),

    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Invalid account state: {0}")]
    InvalidAccountState(String),

    #[error("No accounts exist")]
    NoAccountsExist,

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Failed to parse public key: {0}")]
    PublicKeyError(#[from] nostr_sdk::key::Error),

    #[error("Nostr Manager error: {0}")]
    NostrManagerError(#[from] nostr_manager::NostrManagerError),

    #[error("Failed to serialize account: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Failed to acquire lock: {0}")]
    LockError(String),
}

pub type Result<T> = std::result::Result<T, AccountError>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountSettings {
    pub dark_theme: bool,
    pub dev_mode: bool,
    pub lockdown_mode: bool,
}

impl Default for AccountSettings {
    fn default() -> Self {
        Self {
            dark_theme: true,
            dev_mode: false,
            lockdown_mode: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AccountOnboarding {
    pub inbox_relays: bool,
    pub key_package_relays: bool,
    pub publish_key_package: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Account {
    pub pubkey: String,
    pub metadata: Metadata,
    pub nostr_relays: Vec<String>,
    pub inbox_relays: Vec<String>,
    pub key_package_relays: Vec<String>,
    pub mls_group_ids: Vec<Vec<u8>>,
    pub nostr_group_ids: Vec<String>,
    pub settings: AccountSettings,
    pub onboarding: AccountOnboarding,
    pub last_used: Timestamp,
    pub last_synced: Timestamp,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountManagerState {
    accounts: HashMap<String, Account>,
    active_account: Option<String>,
}

#[derive(Debug)]
pub struct AccountManager {
    state: Mutex<AccountManagerState>,
    db: Arc<sled::Db>,
}

impl AccountManager {
    pub fn new(database: Arc<sled::Db>) -> Result<Self> {
        // Load accounts from database
        let mut accounts = HashMap::new();
        let accounts_tree = database
            .open_tree(ACCOUNTS_DB_TREE_NAME)
            .map_err(|e| AccountError::DatabaseError(e.to_string()))?;

        for result in accounts_tree.iter() {
            let (key, value) = result.map_err(|e| AccountError::DatabaseError(e.to_string()))?;
            let account: Account =
                serde_json::from_slice(&value).map_err(AccountError::SerializationError)?;
            accounts.insert(String::from_utf8_lossy(&key).to_string(), account);
        }

        // Load active account pubkey
        let active_account_pubkey = database
            .get("active_account")
            .map_err(|e| AccountError::DatabaseError(e.to_string()))?
            .map(|bytes| String::from_utf8_lossy(&bytes).to_string());

        let account_state = Self {
            state: Mutex::new(AccountManagerState {
                accounts,
                active_account: active_account_pubkey,
            }),
            db: database,
        };

        tracing::debug!(
            target: "whitenoise::account_manager::new",
            "Loaded accounts state",
        );
        Ok(account_state)
    }

    fn persist_state(&self) -> Result<()> {
        tracing::debug!(
            target: "whitenoise::account_manager::persist_state",
            "Persisting accounts state"
        );

        let state = self
            .state
            .lock()
            .map_err(|e| AccountError::LockError(e.to_string()))?;

        // Persist accounts
        let accounts_tree = self
            .db
            .open_tree(ACCOUNTS_DB_TREE_NAME)
            .map_err(|e| AccountError::DatabaseError(e.to_string()))?;
        for (pubkey, account) in state.accounts.iter() {
            let account_bytes =
                serde_json::to_vec(account).map_err(AccountError::SerializationError)?;
            accounts_tree
                .insert(pubkey, account_bytes)
                .map_err(|e| AccountError::DatabaseError(e.to_string()))?;
        }

        // Persist active account
        if let Some(active_account_pubkey) = state.active_account.clone() {
            self.db
                .insert("active_account", active_account_pubkey.as_bytes())
                .map_err(|e| AccountError::DatabaseError(e.to_string()))?;
        } else {
            self.db
                .insert("active_account", &[])
                .map_err(|e| AccountError::DatabaseError(e.to_string()))?;
        }

        // Flush changes to database to be sure they are written
        self.db
            .flush()
            .map_err(|e| AccountError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub fn account_exists(&self, hex_pubkey: &str) -> Result<bool> {
        let state = self
            .state
            .lock()
            .map_err(|e| AccountError::LockError(e.to_string()))?;
        Ok(state.accounts.contains_key(hex_pubkey))
    }

    pub async fn add_account(
        &self,
        keys: Keys,
        set_active: bool,
        nostr_manager: &nostr_manager::NostrManager,
    ) -> Result<Account> {
        let pubkey = keys.public_key();
        {
            let state = self
                .state
                .lock()
                .map_err(|e| AccountError::LockError(e.to_string()))?;

            if state.accounts.contains_key(&pubkey.to_hex()) {
                return Err(AccountError::AccountAlreadyExists(pubkey.to_hex()));
            }
        }

        // Fetch metadata & relays from Nostr
        let metadata = nostr_manager
            .fetch_user_metadata(pubkey)
            .await
            .map_err(AccountError::NostrManagerError);
        let nostr_relays = nostr_manager
            .fetch_user_relays(pubkey)
            .await
            .map_err(AccountError::NostrManagerError);
        let inbox_relays = nostr_manager
            .fetch_user_inbox_relays(pubkey)
            .await
            .map_err(AccountError::NostrManagerError);
        let key_package_relays = nostr_manager
            .fetch_user_key_package_relays(pubkey)
            .await
            .map_err(AccountError::NostrManagerError);
        let key_packages = nostr_manager
            .fetch_user_key_packages(pubkey)
            .await
            .map_err(AccountError::NostrManagerError)?;

        let mut onboarding = AccountOnboarding::default();

        let unwrapped_metadata = match metadata {
            Ok(Some(metadata)) => metadata,
            _ => Metadata::default(),
        };

        let inbox_relays_unwrapped = inbox_relays.unwrap_or_default();
        let key_package_relays_unwrapped = key_package_relays.unwrap_or_default();

        if !inbox_relays_unwrapped.is_empty() {
            onboarding.inbox_relays = true;
        }
        if !key_package_relays_unwrapped.is_empty() {
            onboarding.key_package_relays = true;
        }
        if !key_packages.is_empty() {
            onboarding.publish_key_package = true;
        }

        let account = Account {
            pubkey: pubkey.to_hex(),
            metadata: unwrapped_metadata,
            nostr_relays: nostr_relays.unwrap_or_default(),
            inbox_relays: inbox_relays_unwrapped,
            key_package_relays: key_package_relays_unwrapped,
            mls_group_ids: vec![],
            nostr_group_ids: vec![],
            settings: AccountSettings::default(),
            onboarding,
            last_used: Timestamp::now(),
            last_synced: Timestamp::zero(),
        };

        {
            let mut state = self
                .state
                .lock()
                .map_err(|e| AccountError::LockError(e.to_string()))?;

            if state.active_account.is_none() || set_active {
                state.active_account = Some(pubkey.to_hex());
            }

            // Add account to state
            state.accounts.insert(pubkey.to_hex(), account.clone());
        }

        self.persist_state()
            .map_err(|e| AccountError::DatabaseError(e.to_string()))?;

        Ok(account)
    }

    pub fn set_active_account(&self, hex_pubkey: Option<String>) -> Result<()> {
        match hex_pubkey {
            Some(hex_pubkey) => {
                {
                    let mut state = self
                        .state
                        .lock()
                        .map_err(|e| AccountError::LockError(e.to_string()))?;

                    if !state.accounts.contains_key(&hex_pubkey) {
                        return Err(AccountError::AccountNotFound(hex_pubkey));
                    }

                    // Update last accessed time
                    if let Some(fetched_account) = state.accounts.get_mut(&hex_pubkey) {
                        fetched_account.last_used = Timestamp::now();
                    }

                    state.active_account = Some(hex_pubkey.clone());
                }

                self.persist_state()
                    .map_err(|e| AccountError::DatabaseError(e.to_string()))?;

                Ok(())
            }
            None => {
                {
                    let mut state = self
                        .state
                        .lock()
                        .map_err(|e| AccountError::LockError(e.to_string()))?;

                    if state.accounts.is_empty() {
                        state.active_account = None;
                    } else {
                        return Err(AccountError::InvalidAccountState(
                            "No active account but accounts exist".to_string(),
                        ));
                    }
                }

                self.persist_state()
                    .map_err(|e| AccountError::DatabaseError(e.to_string()))?;

                Ok(())
            }
        }
    }

    pub fn get_active_account(&self) -> Result<Account> {
        let state = self
            .state
            .lock()
            .map_err(|e| AccountError::LockError(e.to_string()))?;

        let active_pubkey = &state.active_account;

        active_pubkey
            .as_ref()
            .and_then(|pubkey| state.accounts.get(pubkey))
            .cloned()
            .ok_or(AccountError::NoAccountsExist)
    }

    pub fn update_account_onboarding(
        &self,
        pubkey: String,
        inbox_relays: bool,
        key_package_relays: bool,
        publish_key_package: bool,
    ) -> Result<()> {
        {
            let mut state = self
                .state
                .lock()
                .map_err(|e| AccountError::LockError(e.to_string()))?;

            if let Some(account) = state.accounts.get_mut(&pubkey) {
                account.onboarding.inbox_relays = inbox_relays;
                account.onboarding.key_package_relays = key_package_relays;
                account.onboarding.publish_key_package = publish_key_package;
            }
        }

        self.persist_state()
            .map_err(|e| AccountError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub fn add_group_ids(
        &self,
        pubkey: String,
        mls_group_id: Vec<u8>,
        nostr_group_id: String,
    ) -> Result<()> {
        {
            let mut state = self
                .state
                .lock()
                .map_err(|e| AccountError::LockError(e.to_string()))?;

            if let Some(account) = state.accounts.get_mut(&pubkey) {
                if !account.mls_group_ids.contains(&mls_group_id) {
                    account.mls_group_ids.push(mls_group_id);
                }
                if !account.nostr_group_ids.contains(&nostr_group_id) {
                    account.nostr_group_ids.push(nostr_group_id);
                }
            }
        }

        self.persist_state()
            .map_err(|e| AccountError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn remove_group_ids(
        &self,
        pubkey: String,
        mls_group_id: Vec<u8>,
        nostr_group_id: String,
    ) -> Result<()> {
        {
            let mut state = self
                .state
                .lock()
                .map_err(|e| AccountError::LockError(e.to_string()))?;

            if let Some(account) = state.accounts.get_mut(&pubkey) {
                account.mls_group_ids.retain(|id| *id != mls_group_id);
                account.nostr_group_ids.retain(|id| *id != nostr_group_id);
            }
        }

        self.persist_state()
            .map_err(|e| AccountError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub fn update_account(&self, pubkey: String, enriched_contact: EnrichedContact) -> Result<()> {
        {
            let mut state = self
                .state
                .lock()
                .map_err(|e| AccountError::LockError(e.to_string()))?;

            if let Some(account) = state.accounts.get_mut(&pubkey) {
                account.metadata = enriched_contact.metadata;
                account.inbox_relays = enriched_contact.inbox_relays;
                account.key_package_relays = enriched_contact.key_package_relays;
            }
        }

        self.persist_state()
            .map_err(|e| AccountError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub fn get_accounts_state(&self) -> Result<AccountManagerState> {
        let state = self
            .state
            .lock()
            .map_err(|e| AccountError::LockError(e.to_string()))?;

        // If we have somehow gotten into a strange state where there are
        // accounts but no active account, we should try and fix it
        // if !state.accounts.is_empty()
        //     && (state.active_account.is_none()
        //         || state
        //             .active_account
        //             .as_ref()
        //             .is_some_and(|pubkey| pubkey.is_empty()))
        // {
        //     state.active_account = Some(state.accounts.keys().next().unwrap().clone());
        //     self.persist_state()
        //         .map_err(|e| AccountError::DatabaseError(e.to_string()))?;
        // }

        Ok(state.clone())
    }

    pub fn remove_account(&self, hex_pubkey: String) -> Result<()> {
        let current_active_pubkey: Option<String>;
        let new_active_pubkey: Option<String>;

        {
            let mut state = self
                .state
                .lock()
                .map_err(|e| AccountError::LockError(e.to_string()))?;

            if !state.accounts.contains_key(&hex_pubkey) {
                return Err(AccountError::AccountNotFound(hex_pubkey.clone()));
            }

            let removed = state.accounts.remove(&hex_pubkey);
            tracing::debug!(
                target: "whitenoise::account_manager::remove_account",
                "Removed account: {:?}. Accounts now: {:?}",
                removed,
                state.accounts
            );
            current_active_pubkey = state.active_account.clone();
            new_active_pubkey = state.accounts.keys().next().cloned();
        }

        let accounts_tree = self
            .db
            .open_tree(ACCOUNTS_DB_TREE_NAME)
            .map_err(|e| AccountError::DatabaseError(e.to_string()))?;

        accounts_tree
            .remove(&hex_pubkey)
            .map_err(|e| AccountError::DatabaseError(e.to_string()))?;

        if current_active_pubkey == Some(hex_pubkey.clone()) {
            self.set_active_account(new_active_pubkey)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    #[ignore]
    async fn test_add_account() {
        todo!("Implement this test, needs mocks for nostr_manager")
    }

    #[test]
    fn test_set_active_account() {
        let temp_dir = tempdir().unwrap().path().to_path_buf();
        let database = Arc::new(
            sled::open(temp_dir.join("whitenoise.sled")).expect("Failed to open database"),
        );
        let account_manager = AccountManager::new(database).unwrap();

        let result = account_manager.set_active_account(Some(
            "10587d333f9ccaeab8c856a5d16fdc7fe5dd8eac3dd4167c48862e3460462e92".to_string(),
        ));
        assert!(result.is_err()); // Should fail because account doesn't exist

        // Add an account first
        let account = Account {
            pubkey: "10587d333f9ccaeab8c856a5d16fdc7fe5dd8eac3dd4167c48862e3460462e92".to_string(),
            metadata: Metadata::default(),
            nostr_relays: vec![],
            inbox_relays: vec![],
            key_package_relays: vec![],
            mls_group_ids: vec![],
            nostr_group_ids: vec![],
            settings: AccountSettings::default(),
            onboarding: AccountOnboarding::default(),
            last_used: Timestamp::now(),
            last_synced: Timestamp::zero(),
        };

        {
            let mut state = account_manager.state.lock().unwrap();
            state.accounts.insert(
                "10587d333f9ccaeab8c856a5d16fdc7fe5dd8eac3dd4167c48862e3460462e92".to_string(),
                account,
            );
        }

        let result = account_manager.set_active_account(Some(
            "10587d333f9ccaeab8c856a5d16fdc7fe5dd8eac3dd4167c48862e3460462e92".to_string(),
        ));
        assert!(result.is_ok());
    }

    #[test]
    fn test_remove_account() {
        let temp_dir = tempdir().unwrap().path().to_path_buf();
        let database = Arc::new(
            sled::open(temp_dir.join("whitenoise.sled")).expect("Failed to open database"),
        );
        let account_manager = AccountManager::new(database).unwrap();

        // Add an account first
        let account = Account {
            pubkey: "10587d333f9ccaeab8c856a5d16fdc7fe5dd8eac3dd4167c48862e3460462e92".to_string(),
            metadata: Metadata::default(),
            nostr_relays: vec![],
            inbox_relays: vec![],
            key_package_relays: vec![],
            mls_group_ids: vec![],
            nostr_group_ids: vec![],
            settings: AccountSettings::default(),
            onboarding: AccountOnboarding::default(),
            last_used: Timestamp::now(),
            last_synced: Timestamp::zero(),
        };

        {
            let mut state = account_manager.state.lock().unwrap();
            state.accounts.insert(
                "10587d333f9ccaeab8c856a5d16fdc7fe5dd8eac3dd4167c48862e3460462e92".to_string(),
                account,
            );
        }

        let result = account_manager.remove_account(
            "10587d333f9ccaeab8c856a5d16fdc7fe5dd8eac3dd4167c48862e3460462e92".to_string(),
        );
        assert!(result.is_ok());

        let state = account_manager.state.lock().unwrap();
        assert!(!state
            .accounts
            .contains_key("10587d333f9ccaeab8c856a5d16fdc7fe5dd8eac3dd4167c48862e3460462e92"));
    }
}
