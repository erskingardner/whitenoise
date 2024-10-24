use crate::nostr::{self, subscribe_to_mls_group_messages};
use crate::nostr::{update_nostr_identity, EnrichedContact, DEFAULT_RELAYS, DEFAULT_TIMEOUT};
use crate::secrets_store;
use crate::{database::Database, whitenoise::Whitenoise};
use anyhow::Result;
use log::debug;
use nostr_sdk::{Filter, Keys, Kind, Metadata, TagKind};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::str::from_utf8;
use tauri::Emitter;
use tauri::State;

/// Key used to store and retrieve accounts data in the database
const ACCOUNTS_KEY: &str = "accounts";

/// Represents the accounts and current identity information
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Accounts {
    /// List of account identifiers
    pub accounts: Option<HashMap<String, EnrichedContact>>,
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
    pub fn save(&self, database: &Database) -> Result<()> {
        let json = serde_json::to_string(self)?;
        database.insert(ACCOUNTS_KEY, json.as_str())?;
        Ok(())
    }

    pub fn get_nostr_keys_for_current_identity(&self, app_data_dir: &Path) -> Result<Option<Keys>> {
        match &self.current_identity {
            Some(identity) => {
                let keys = secrets_store::get_nostr_keys_for_pubkey(identity, app_data_dir)?;
                Ok(Some(keys))
            }
            None => Ok(None),
        }
    }

    /// Retrieves the key package relays for the current identity
    ///
    /// This method returns a vector of relay URLs associated with the current identity's key package.
    /// If no current identity is set or if the current identity has no specific relays,
    /// it returns a default set of relays.
    ///
    /// # Returns
    ///
    /// A `Result` containing either:
    /// - A `Vec<String>` of relay URLs
    /// - An error if any operation fails
    ///
    /// # Note
    ///
    /// This method assumes the existence of a `DEFAULT_RELAYS` constant,
    /// which should be defined elsewhere in the codebase.
    pub fn get_key_package_relays_for_current_identity(&self) -> Result<Vec<String>> {
        match &self.current_identity {
            Some(identity) => {
                let relays = self
                    .accounts
                    .as_ref()
                    .and_then(|accounts| accounts.get(identity))
                    .map(|account| account.key_package_relays.clone());
                Ok(relays.unwrap_or(DEFAULT_RELAYS.iter().map(|r| r.to_string()).collect()))
            }
            None => Ok(DEFAULT_RELAYS.iter().map(|r| r.to_string()).collect()),
        }
    }

    /// Deletes all account data and resets to default
    ///
    /// This method clears all existing account data and resets the accounts
    /// to the default empty state in the database.
    ///
    /// # Arguments
    ///
    /// * `database` - A reference to the `Database` instance to save to
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an error if saving fails
    pub fn delete_data(&self, database: &Database) -> Result<()> {
        debug!(target: "accounts::delete_data", "Deleting accounts");
        let accounts = Accounts::default();
        accounts.save(database)?;
        Ok(())
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
    Ok(wn
        .accounts
        .lock()
        .expect("Couldn't lock accounts while fetching")
        .clone())
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
    source: String,
    wn: State<'_, Whitenoise>,
    app_handle: tauri::AppHandle,
) -> Result<Accounts, String> {
    debug!(target: "whitenoise::accounts::login", "Logging in with nsec or hex from {:?}", source);
    let keys = Keys::parse(nsec_or_hex).map_err(|e| e.to_string())?;

    update_nostr_identity(keys.clone(), &wn)
        .await
        .map_err(|e| e.to_string())?;

    let metadata = wn
        .nostr
        .fetch_metadata(keys.public_key(), Some(DEFAULT_TIMEOUT))
        .await
        .unwrap_or_else(|_| Metadata::default());

    let mut enriched_contact = EnrichedContact {
        metadata,
        nip17: false,
        nip104: false,
        inbox_relays: vec![],
        key_package_relays: vec![],
    };

    // Prepare filters for messaging capabilities
    let dm_relay_list_filter = Filter::new()
        .kind(Kind::Custom(10050))
        .author(keys.public_key());
    let prekey_filter = Filter::new()
        .kind(Kind::KeyPackage)
        .author(keys.public_key());
    let key_package_list_filter = Filter::new()
        .kind(Kind::Custom(10051))
        .author(keys.public_key());

    // Fetch messaging capabilities for all contacts in a single query
    let messaging_capabilities_events = wn
        .nostr
        .fetch_events(
            vec![dm_relay_list_filter, prekey_filter, key_package_list_filter],
            Some(DEFAULT_TIMEOUT),
        )
        .await
        .expect("Failed to fetch messaging capabilities");

    // Process messaging capabilities
    for event in messaging_capabilities_events {
        match event.kind {
            Kind::Replaceable(10050) => {
                enriched_contact.nip17 = true;
                enriched_contact.inbox_relays.extend(
                    event
                        .tags
                        .iter()
                        .filter(|tag| tag.kind() == TagKind::Relay)
                        .filter_map(|tag| tag.content())
                        .map(|s| s.to_string()),
                );
            }
            Kind::Replaceable(10051) => {
                enriched_contact.key_package_relays.extend(
                    event
                        .tags
                        .iter()
                        .filter(|tag| tag.kind() == TagKind::Relay)
                        .filter_map(|tag| tag.content())
                        .map(|s| s.to_string()),
                );
            }
            Kind::KeyPackage => {
                if event.tags.find(TagKind::MlsProtocolVersion).is_some() {
                    enriched_contact.nip104 = true;
                }
            }
            _ => {}
        }
    }

    // Scope the MutexGuard to release it before the .await
    {
        let mut accounts = wn
            .accounts
            .lock()
            .map_err(|e| format!("Failed to lock accounts: {}", e))?;
        let pubkey = keys.public_key().to_hex();

        // Update or create the accounts map
        accounts
            .accounts
            .get_or_insert_with(HashMap::new)
            .insert(pubkey.clone(), enriched_contact);

        // Set current identity
        accounts.current_identity = Some(pubkey);

        // Save accounts to database
        accounts
            .save(&wn.wdb)
            .map_err(|e| format!("Failed to save accounts: {}", e))?;
        debug!(target: "whitenoise::accounts::login", "Saved accounts to database: {:#?}", accounts);

        // Store private key in secrets store
        secrets_store::store_private_key(&keys, &wn.data_dir)
            .map_err(|e| format!("Failed to store private key: {}", e))?;
        debug!(target: "whitenoise::accounts::login", "Saved private key to secrets store");
    }

    subscribe_to_mls_group_messages(&wn)
        .await
        .map_err(|e| e.to_string())?;

    wn.nostr_mls
        .update_provider_for_user(Some(keys.public_key().to_hex()));

    app_handle
        .emit("identity_change", ())
        .expect("Couldn't emit event");

    // Fetch and return the updated accounts
    wn.accounts
        .lock()
        .map_err(|e| format!("Failed to lock accounts: {}", e))
        .map(|accounts| accounts.clone())
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
) -> Result<(), String> {
    debug!(target: "whitenoise::accounts::logout", "Logging out pubkey: {:?}", pubkey);
    let mut accounts = wn.accounts.lock().map_err(|e| e.to_string())?;

    debug!(target: "whitenoise::accounts::logout", "Before remove: {:?}", accounts);
    // Remove the passed pubkey from the accounts map
    if let Some(accounts_map) = &mut accounts.accounts {
        accounts_map.remove(&pubkey);
    }
    debug!(target: "whitenoise::accounts::logout", "After remove: {:?}", accounts);
    // Remove the private key from the secrets store
    secrets_store::remove_private_key_for_pubkey(&pubkey, &wn.data_dir)
        .map_err(|e| e.to_string())?;

    // Set the current identity to the next available identity (if the current identity was removed)
    if accounts.current_identity.as_ref() == Some(&pubkey) {
        accounts.current_identity = accounts
            .accounts
            .as_ref()
            .and_then(|map| map.keys().next().cloned());

        debug!(target: "whitenoise::accounts::logout", "Current identity was logged out. New current identity: {:?}", accounts.current_identity);
    } else {
        debug!(target: "whitenoise::accounts::logout", "Logged out identity was not the current identity. Current identity remains: {:?}", accounts.current_identity);
    }

    // Save the accounts
    accounts.save(&wn.wdb).map_err(|e| e.to_string())?;

    wn.nostr_mls
        .update_provider_for_user(accounts.current_identity.clone());

    // Emit an identity change event
    app_handle
        .emit("identity_change", ())
        .map_err(|e| e.to_string())?;

    Ok(())
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

    let keys = secrets_store::get_nostr_keys_for_pubkey(&pubkey, &wn.data_dir)
        .map_err(|e| e.to_string())?;
    nostr::update_nostr_identity(keys.clone(), &wn)
        .await
        .expect("Failed to update Nostr identity");

    let mut accounts = wn.accounts.lock().unwrap();
    accounts.current_identity = Some(keys.public_key().to_hex());
    accounts.save(&wn.wdb).map_err(|e| e.to_string())?;

    wn.nostr_mls
        .update_provider_for_user(accounts.current_identity.clone());

    app_handle
        .emit("identity_change", ())
        .map_err(|e| e.to_string())?;

    debug!(target: "whitenoise::accounts::set_current_identity", "Updated accounts: {:#?}", accounts.clone());
    Ok(accounts.clone())
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
    let accounts = login(
        keys.secret_key().to_secret_hex(),
        "create_identity".to_string(),
        wn,
        app_handle,
    )
    .await?;
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
        let enriched_contact = EnrichedContact {
            metadata: Metadata::new(),
            nip17: false,
            nip104: false,
            inbox_relays: vec![],
            key_package_relays: vec![],
        };
        test_accounts.insert("pubkey1".to_string(), enriched_contact);
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
        store_private_key(&test_keys, &tempdir()?.path().join("secrets.json"))?;
        accounts.current_identity = Some(test_keys.public_key().to_string());

        // Test retrieving keys
        let retrieved_keys = accounts
            .get_nostr_keys_for_current_identity(&tempdir()?.path().join("secrets.json"))?;
        assert!(retrieved_keys.is_some());
        assert_eq!(retrieved_keys.unwrap().public_key(), test_keys.public_key());

        // Test with no current identity
        accounts.current_identity = None;
        let no_keys = accounts
            .get_nostr_keys_for_current_identity(&tempdir()?.path().join("secrets.json"))?;
        assert!(no_keys.is_none());

        Ok(())
    }

    #[test]
    fn test_delete_data() -> Result<()> {
        use tempfile::tempdir;

        // Create a temporary directory for the test database
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test.db");
        let db = Database::new(&db_path)?;

        // Create and populate an Accounts instance
        let mut accounts = Accounts::default();
        let mut test_accounts = HashMap::new();
        let enriched_contact = EnrichedContact {
            metadata: Metadata::new(),
            nip17: false,
            nip104: false,
            inbox_relays: vec![],
            key_package_relays: vec![],
        };
        test_accounts.insert("pubkey1".to_string(), enriched_contact);
        accounts.accounts = Some(test_accounts);
        accounts.current_identity = Some("pubkey1".to_string());

        // Save accounts to the database
        accounts.save(&db)?;

        // Verify that the accounts data is in the database
        let retrieved_accounts = Accounts::from_database(&db)?;
        assert!(retrieved_accounts.accounts.is_some());
        assert!(retrieved_accounts.current_identity.is_some());

        // Delete the data
        accounts.delete_data(&db)?;

        // Verify that the accounts data has been deleted
        let deleted_accounts = Accounts::from_database(&db)?;
        assert_eq!(deleted_accounts, Accounts::default());

        Ok(())
    }
}
