use crate::database::DatabaseError;
use crate::groups::Group;
use crate::invites::Invite;
use crate::nostr_manager;
use crate::secrets_store;
use crate::Whitenoise;
use nostr_openmls::NostrMls;
use nostr_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use tauri::Emitter;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AccountError {
    #[error("Missing required pubkey")]
    MissingPubkey,

    #[error("Account not found")]
    AccountNotFound,

    #[error("Database error: {0}")]
    DatabaseError(#[from] DatabaseError),

    #[error("Failed to parse public key: {0}")]
    PublicKeyError(#[from] nostr_sdk::key::Error),

    #[error("Nostr Manager error: {0}")]
    NostrManagerError(#[from] nostr_manager::NostrManagerError),

    #[error("Error with secrets store: {0}")]
    SecretsStoreError(#[from] secrets_store::SecretsStoreError),

    #[error("Tauri error: {0}")]
    TauriError(#[from] tauri::Error),

    #[error("Failed to acquire lock")]
    LockError,

    #[error("No active account found")]
    NoActiveAccount,
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

impl Account {
    /// Generates a new keypair and saves the mostly blank account to the database
    pub fn new(wn: &tauri::State<'_, Whitenoise>) -> Result<Account> {
        let keys = Keys::generate();
        let account = Account {
            pubkey: keys.public_key().to_hex(),
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
        let account = account.save(wn)?;

        // If the record saves, add the keys to the secret store
        secrets_store::store_private_key(&keys, &wn.data_dir)?;

        Ok(account)
    }
    /// Adds an account from an existing keypair
    pub async fn add_from_keys(
        keys: Keys,
        set_active: bool,
        wn: &Whitenoise,
        app_handle: &tauri::AppHandle,
    ) -> Result<Account> {
        let pubkey = keys.public_key();

        tracing::debug!(target: "whitenoise::accounts", "Adding account for pubkey: {}", pubkey.to_hex());

        // Fetch metadata & relays from Nostr
        // We only fetch the data at this point to store on the account
        // We'll add and connect to the relays in another step
        let metadata = wn
            .nostr
            .fetch_user_metadata(pubkey)
            .await
            .map_err(AccountError::NostrManagerError);
        tracing::debug!(target: "whitenoise::accounts", "Fetched metadata for pubkey: {}", pubkey.to_hex());
        let nostr_relays = wn
            .nostr
            .fetch_user_relays(pubkey)
            .await
            .map_err(AccountError::NostrManagerError);
        tracing::debug!(target: "whitenoise::accounts", "Fetched relays for pubkey: {}", pubkey.to_hex());
        let inbox_relays = wn
            .nostr
            .fetch_user_inbox_relays(pubkey)
            .await
            .map_err(AccountError::NostrManagerError);
        tracing::debug!(target: "whitenoise::accounts", "Fetched inbox relays for pubkey: {}", pubkey.to_hex());
        let key_package_relays = wn
            .nostr
            .fetch_user_key_package_relays(pubkey)
            .await
            .map_err(AccountError::NostrManagerError);
        tracing::debug!(target: "whitenoise::accounts", "Fetched key package relays for pubkey: {}", pubkey.to_hex());
        // let key_packages = wn
        //     .nostr
        //     .fetch_user_key_packages(pubkey)
        //     .await
        //     .map_err(AccountError::NostrManagerError)?;
        // tracing::debug!(target: "whitenoise::accounts", "Fetched key packages for pubkey: {}", pubkey.to_hex());

        let mut onboarding = AccountOnboarding::default();

        let unwrapped_metadata = match metadata {
            Ok(Some(metadata)) => metadata.to_owned(),
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
        // if !key_packages.is_empty() {
        //     onboarding.publish_key_package = true;
        // }

        tracing::debug!(target: "whitenoise::accounts", "Creating account with metadata: {:?}", unwrapped_metadata);

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

        account.save(wn)?;

        // If the record saves, add the keys to the secret store
        secrets_store::store_private_key(&keys, &wn.data_dir)?;

        tracing::debug!(target: "whitenoise::accounts", "Account added from keys and secret saved");

        if set_active {
            account.set_active(wn, app_handle).await?;
        }

        Ok(account)
    }

    /// Finds an account by its public key
    pub fn find_by_pubkey(pubkey: &str, wn: &Whitenoise) -> Result<Account> {
        if pubkey.is_empty() {
            return Err(AccountError::MissingPubkey);
        }

        let rtxn = wn.database.read_txn()?;
        wn.database
            .accounts_db()
            .get(&rtxn, pubkey)
            .map_err(DatabaseError::LmdbError)?
            .ok_or_else(|| AccountError::AccountNotFound)
    }

    /// Returns all accounts
    pub fn all(wn: &Whitenoise) -> Result<Vec<Account>> {
        let rtxn = wn.database.read_txn()?;
        let iter = wn
            .database
            .accounts_db()
            .iter(&rtxn)
            .map_err(DatabaseError::LmdbError)?;

        iter.map(|result| {
            result
                .map(|(_, account)| account)
                .map_err(|e| AccountError::from(DatabaseError::DeserializationError(e.to_string())))
        })
        .collect()
    }
    /// Returns the currently active account
    pub fn get_active(wn: &Whitenoise) -> Result<Account> {
        let rtxn = wn.database.read_txn()?;
        if let Some(active_pubkey) = wn
            .database
            .active_account_db()
            .get(&rtxn, "active")
            .map_err(DatabaseError::LmdbError)?
        {
            match wn
                .database
                .accounts_db()
                .get(&rtxn, active_pubkey)
                .map_err(DatabaseError::LmdbError)?
            {
                Some(account) => Ok(account),
                None => Err(AccountError::NoActiveAccount),
            }
        } else {
            Err(AccountError::NoActiveAccount)
        }
    }

    /// Sets the active account in the database and updates nostr for the active identity
    pub async fn set_active(
        &self,
        wn: &Whitenoise,
        app_handle: &tauri::AppHandle,
    ) -> Result<Account> {
        let mut wtxn = wn.database.write_txn()?;
        wn.database
            .active_account_db()
            .put(&mut wtxn, "active", &self.pubkey)
            .map_err(|e| DatabaseError::SerializationError(e.to_string()))?;
        wtxn.commit()
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

        // If the database operation is successful, update Nostr client
        wn.nostr.set_nostr_identity(self, wn, app_handle).await?;

        app_handle.emit("nostr_ready", ())?;

        // Then update Nostr MLS instance
        {
            let mut nostr_mls = wn.nostr_mls.lock().map_err(|_| AccountError::LockError)?;
            *nostr_mls = NostrMls::new(wn.data_dir.clone(), Some(self.pubkey.clone()));
        }

        app_handle.emit("account_changed", ())?;
        Ok(self.clone())
    }

    /// Returns the groups the account is a member of
    pub fn groups(&self, wn: &Whitenoise) -> Result<Vec<Group>> {
        let rtxn = wn.database.read_txn()?;
        let iter = wn
            .database
            .groups_db()
            .prefix_iter(&rtxn, self.pubkey.as_bytes())
            .map_err(DatabaseError::LmdbError)?;
        iter.map(|result| {
            result
                .map(|(_, group)| group)
                .map_err(|e| AccountError::from(DatabaseError::DeserializationError(e.to_string())))
        })
        .collect()
    }

    /// Returns the invites the account has received
    pub fn invites(&self, wn: &Whitenoise) -> Result<Vec<Invite>> {
        let rtxn = wn.database.read_txn()?;
        let iter = wn
            .database
            .invites_db()
            .prefix_iter(&rtxn, self.pubkey.as_str())
            .map_err(DatabaseError::LmdbError)?;
        iter.map(|result| {
            result
                .map(|(_, invite)| invite)
                .map_err(|e| AccountError::from(DatabaseError::DeserializationError(e.to_string())))
        })
        .collect()
    }

    pub fn keys(&self, wn: &Whitenoise) -> Result<Keys> {
        Ok(secrets_store::get_nostr_keys_for_pubkey(
            self.pubkey.as_str(),
            &wn.data_dir,
        )?)
    }

    /// Saves the account to the database
    pub fn save(&self, wn: &Whitenoise) -> Result<Account> {
        if self.pubkey.is_empty() {
            return Err(AccountError::MissingPubkey);
        }

        let mut wtxn = wn.database.write_txn()?;
        wn.database
            .accounts_db()
            .put(&mut wtxn, &self.pubkey, self)
            .map_err(|e| DatabaseError::SerializationError(e.to_string()))?;

        wtxn.commit()
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

        Ok(self.clone())
    }

    /// Removes the account from the database
    pub async fn remove(&self, wn: &Whitenoise, app_handle: tauri::AppHandle) -> Result<()> {
        let hex_pubkey = self.pubkey.clone();

        let rtxn = wn.database.read_txn()?;
        let mut wtxn = wn.database.write_txn()?;
        // First remove the account from the database
        wn.database
            .accounts_db()
            .delete(&mut wtxn, &self.pubkey)
            .map_err(DatabaseError::LmdbError)?;

        // Get first remaining account's pubkey (if any)
        let first_account_pubkey = wn
            .database
            .accounts_db()
            .iter(&rtxn)
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?
            .next()
            .map(|r| r.map(|(k, _)| k.to_string()))
            .transpose()
            .map_err(|e| DatabaseError::DeserializationError(e.to_string()))?;

        // Then set the next account as the active one
        if let Some(pubkey) = first_account_pubkey.clone() {
            wn.database
                .active_account_db()
                .put(&mut wtxn, "active", &pubkey)
                .map_err(DatabaseError::LmdbError)?;
        } else {
            // Or if no accounts remain, clear the active account
            wn.database
                .active_account_db()
                .delete(&mut wtxn, "active")
                .map_err(DatabaseError::LmdbError)?;
        }

        wtxn.commit()
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

        // If the database update succeeded, then we continue with other steps

        // Remove the old account's private key from the secrets store
        secrets_store::remove_private_key_for_pubkey(&hex_pubkey, &wn.data_dir)?;

        // Update Nostr client & Nostr MLS
        let account = Account::get_active(wn)?;
        wn.nostr
            .set_nostr_identity(&account, wn, &app_handle)
            .await?;

        app_handle.emit("nostr_ready", ())?;

        // Then update Nostr MLS instance
        {
            let mut nostr_mls = wn.nostr_mls.lock().map_err(|_| AccountError::LockError)?;
            *nostr_mls = NostrMls::new(wn.data_dir.clone(), Some(account.pubkey.clone()));
        }

        app_handle.emit("account_changed", ())?;
        Ok(())
    }
}
