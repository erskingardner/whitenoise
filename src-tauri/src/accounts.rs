use crate::nostr;
use crate::secrets_store;
use crate::{database::Database, whitenoise::Whitenoise};
use anyhow::Result;
use log::debug;
use nostr_sdk::{Keys, Metadata};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::from_utf8;
use tauri::Emitter;
use tauri::State;

/// Key used to store and retrieve accounts data in the database
const ACCOUNTS_KEY: &str = "accounts";

/// Represents the accounts and current identity information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Accounts {
    /// List of account identifiers
    pub accounts: Option<HashMap<String, Metadata>>,
    /// The currently active identity
    pub current_identity: Option<String>,
}

impl Accounts {
    /// Creates a new `Accounts` instance with default values
    ///
    /// # Returns
    ///
    /// A new `Accounts` struct with `None` values for both fields
    pub fn default() -> Self {
        Self {
            accounts: None,
            current_identity: None,
        }
    }

    /// Retrieves the accounts data from the database
    ///
    /// # Arguments
    ///
    /// * `database` - A reference to the `Database` instance
    ///
    /// # Returns
    ///
    /// A `Result` containing either the deserialized `Accounts` struct or an error
    pub fn from_database(database: &Database) -> Result<Self> {
        let results = database.get(ACCOUNTS_KEY)?;
        match results {
            Some(results) => {
                let accounts_str = from_utf8(&results)?;
                Ok(serde_json::from_str(accounts_str)?)
            }
            None => Ok(Accounts::default()),
        }
    }

    /// Saves the current accounts data to the database
    ///
    /// # Arguments
    ///
    /// * `database` - A reference to the `Database` instance
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of the save operation
    ///
    /// # TODO
    ///
    /// Improve error handling and propagate errors to the UI layer
    pub fn save(&self, database: &Database) -> Result<()> {
        let json = serde_json::to_string(self)?;
        database.insert(ACCOUNTS_KEY, json.as_str())?;
        Ok(())
    }

    pub fn get_nostr_keys_for_current_identity(&self) -> Result<Option<Keys>> {
        match &self.current_identity {
            Some(identity) => {
                let keys = secrets_store::get_nostr_keys_for_pubkey(identity)?;
                Ok(Some(keys))
            }
            None => Ok(None),
        }
    }
}

/// Retrieves the current accounts data
///
/// # Arguments
///
/// * `wn` - A `State` containing the `Whitenoise` instance
///
/// # Returns
///
/// A `Result` containing either the `Accounts` struct or an error string
#[tauri::command]
pub fn get_accounts(wn: State<'_, Whitenoise>) -> Result<Accounts, String> {
    debug!(target: "whitenoise::accounts::get_accounts", "Getting accounts");
    Ok(wn.accounts.lock().unwrap().clone())
}

/// Adds a new identity to the accounts using the provided secret key
/// Log in of sorts
///
/// # Arguments
///
/// * `wn` - A `State` containing the `Whitenoise` instance
/// * `secret_key` - A string slice containing the secret key for the new identity
///
/// # Returns
///
/// A `Result` indicating success (`Ok(())`) or an error message (`Err(String)`)
///
/// # Errors
///
/// This function will return an error if:
/// - The secret key cannot be parsed
/// - The metadata for the public key cannot be retrieved
/// - The accounts mutex cannot be locked
/// - The accounts cannot be saved to the database
#[tauri::command]
pub async fn login(
    nsec_or_hex: String,
    wn: State<'_, Whitenoise>,
    app_handle: tauri::AppHandle,
) -> Result<Accounts, String> {
    debug!(target: "whitenoise::accounts::login", "Logging in with nsec or hex");
    let keys = Keys::parse(nsec_or_hex).map_err(|e| e.to_string())?;
    let metadata = wn
        .nostr
        .metadata(keys.public_key())
        .await
        .unwrap_or_else(|_| Metadata::default());

    // Scope the MutexGuard to release it before the .await
    {
        let mut accounts = wn.accounts.lock().map_err(|e| e.to_string())?;
        match accounts.accounts.as_mut() {
            Some(accounts) => {
                accounts.insert(keys.public_key().to_string(), metadata);
            }
            None => {
                let mut new_accounts = HashMap::new();
                new_accounts.insert(keys.public_key().to_string(), metadata);
                accounts.accounts = Some(new_accounts);
            }
        }
        accounts.current_identity = Some(keys.public_key().to_string());
        accounts.save(&wn.wdb).map_err(|e| e.to_string())?;
        debug!(target: "whitenoise::accounts::login", "Saved accounts to database: {:?}", accounts);
        secrets_store::store_private_key(&keys).map_err(|e| e.to_string())?;
        debug!(target: "whitenoise::accounts::login", "Saved private key to secrets store");
    }

    nostr::update_nostr_identity(keys, &wn).await?;

    app_handle
        .emit("identity_change", ())
        .expect("Couldn't emit event");

    // Fetch and return the updated accounts
    Ok(wn.accounts.lock().map_err(|e| e.to_string())?.clone())
}

/// Logs out a user by removing their account and associated private key
///
/// This function removes the specified account from the accounts map,
/// deletes the corresponding private key from the secrets store,
/// updates the current identity if necessary, and saves the changes.
///
/// # Arguments
///
/// * `pubkey` - A `String` representing the public key of the account to be removed
/// * `wn` - A `State` containing the `Whitenoise` instance
/// * `app_handle` - The `tauri::AppHandle` for emitting events
///
/// # Returns
///
/// A `Result` containing the updated `Accounts` if successful, or an error string
///
/// # Errors
///
/// This function will return an error if:
/// - The accounts mutex cannot be locked
/// - The private key cannot be removed from the secrets store
/// - The accounts cannot be saved to the database
/// - The identity change event cannot be emitted
#[tauri::command]
pub fn logout(
    pubkey: String,
    wn: State<'_, Whitenoise>,
    app_handle: tauri::AppHandle,
) -> Result<Accounts, String> {
    debug!(target: "whitenoise::accounts::logout", "Logging out pubkey: {:?}", pubkey);
    let mut accounts = wn.accounts.lock().map_err(|e| e.to_string())?;

    // Remove the passed pubkey from the accounts map
    if let Some(accounts_map) = &mut accounts.accounts {
        accounts_map.remove(&pubkey);
    }

    // Remove the private key from the secrets store
    secrets_store::remove_private_key_for_pubkey(&pubkey).map_err(|e| e.to_string())?;

    // Set the current identity to the next available identity (if the current identity was removed)
    if accounts.current_identity.as_ref() == Some(&pubkey) {
        accounts.current_identity = accounts
            .accounts
            .as_ref()
            .and_then(|map| map.keys().next().cloned());
    }

    // Save the accounts
    accounts.save(&wn.wdb).map_err(|e| e.to_string())?;

    // Emit an identity change event
    app_handle
        .emit("identity_change", ())
        .map_err(|e| e.to_string())?;

    Ok(accounts.clone())
}

/// Sets the current active identity
///
/// # Arguments
///
/// * `wn` - A `State` containing the `Whitenoise` instance
/// * `identity` - A `String` representing the pubkey to set as current
///
/// # Returns
///
/// A `Result` indicating success or an error string
#[tauri::command]
pub async fn set_current_identity(
    pubkey: String,
    wn: State<'_, Whitenoise>,
    app_handle: tauri::AppHandle,
) -> Result<Accounts, String> {
    debug!(target: "whitenoise::accounts::set_current_identity", "Setting current identity to: {:?}", pubkey);

    let keys = secrets_store::get_nostr_keys_for_pubkey(&pubkey).map_err(|e| e.to_string())?;
    nostr::update_nostr_identity(keys, &wn).await?;

    let mut accounts = wn.accounts.lock().map_err(|e| e.to_string())?;
    accounts.current_identity = Some(pubkey);
    accounts.save(&wn.wdb).map_err(|e| e.to_string())?;
    let accounts_clone = accounts.clone();
    drop(accounts);

    app_handle
        .emit("identity_change", ())
        .map_err(|e| e.to_string())?;

    Ok(accounts_clone)
}

/// Creates a new identity and adds it to the accounts
///
/// This function generates a new key pair, adds the corresponding identity
/// to the accounts, and sets it as the current identity.
///
/// # Arguments
///
/// * `wn` - A `State` containing the `Whitenoise` instance
/// * `app_handle` - The `tauri::AppHandle` for emitting events
///
/// # Returns
///
/// A `Result` containing the updated `Accounts` if successful, or an error string
#[tauri::command]
pub async fn create_identity(
    wn: State<'_, Whitenoise>,
    app_handle: tauri::AppHandle,
) -> Result<Accounts, String> {
    let keys = Keys::generate();
    let accounts = login(keys.secret_key().unwrap().to_string(), wn, app_handle).await?;
    Ok(accounts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_accounts_default() {
        let accounts = Accounts::default();
        assert!(accounts.accounts.is_none());
        assert!(accounts.current_identity.is_none());
    }

    #[test]
    fn test_accounts_from_database_empty() -> Result<()> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test.db");
        let db = Database::new(&db_path).unwrap();
        let accounts = Accounts::from_database(&db).unwrap();
        assert!(accounts.accounts.is_none());
        assert!(accounts.current_identity.is_none());
        Ok(())
    }

    #[test]
    fn test_accounts_save_and_retrieve() -> Result<()> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test.db");
        let db = Database::new(&db_path).unwrap();

        let mut accounts = Accounts::default();
        let mut test_accounts = HashMap::new();
        test_accounts.insert("pubkey1".to_string(), Metadata::new());
        accounts.accounts = Some(test_accounts);
        accounts.current_identity = Some("pubkey1".to_string());

        // Save accounts
        accounts.save(&db)?;

        // Retrieve accounts
        let retrieved_accounts = Accounts::from_database(&db)?;

        assert_eq!(accounts.accounts, retrieved_accounts.accounts);
        assert_eq!(
            accounts.current_identity,
            retrieved_accounts.current_identity
        );

        Ok(())
    }

    #[test]
    fn test_get_nostr_keys_for_current_identity() -> Result<()> {
        use crate::secrets_store::store_private_key;

        let mut accounts = Accounts::default();

        // Set up a test key in the secrets store
        let test_keys = Keys::generate();
        store_private_key(&test_keys)?;
        accounts.current_identity = Some(test_keys.public_key().to_string());

        // Test retrieving keys
        let retrieved_keys = accounts.get_nostr_keys_for_current_identity()?;
        assert!(retrieved_keys.is_some());
        assert_eq!(retrieved_keys.unwrap().public_key(), test_keys.public_key());

        // Test with no current identity
        accounts.current_identity = None;
        let no_keys = accounts.get_nostr_keys_for_current_identity()?;
        assert!(no_keys.is_none());

        Ok(())
    }
}
