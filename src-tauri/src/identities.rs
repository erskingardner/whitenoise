use crate::database::Database;
use crate::nostr;
use crate::secrets_store;
use crate::AppSettings;
use crate::AppState;
use anyhow::Result;
use nostr_sdk::nips::nip04;
use nostr_sdk::Keys;
use nostr_sdk::PublicKey;
use serde::{Deserialize, Serialize};
use std::str::from_utf8;
use tauri::Emitter;
use tauri::State;

/// Identities
/// The various identities that the user has created or signed in with) are
/// json encoded arrays of hexpubkey values
#[derive(Debug, Serialize, Deserialize)]
pub struct Identities(Vec<String>);

const IDENTITIES_KEY: &str = "identities";

#[derive(Debug, Serialize, Deserialize)]
pub struct CurrentIdentity(String);

impl Identities {
    pub fn add(&mut self, pubkey: &String) {
        self.0.push(pubkey.to_owned());
    }
    pub fn remove(&mut self, pubkey: &String) {
        self.0.retain(|x| x != pubkey);
    }

    pub fn from_database(database: &Database) -> Result<Option<Self>> {
        let identities = database.get(IDENTITIES_KEY)?;
        match identities {
            Some(identities) => {
                let identities_str = from_utf8(&identities)?;
                let filtered_identities: Vec<String> = identities_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                if filtered_identities.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(Identities(filtered_identities)))
                }
            }
            None => Ok(None),
        }
    }
    pub fn save(&self, database: &Database) -> Result<()> {
        let filtered_identities: Vec<String> = self
            .0
            .iter()
            .filter(|id| !id.trim().is_empty())
            .cloned()
            .collect();

        if !filtered_identities.is_empty() {
            let identities = filtered_identities.join(",");
            database.insert(IDENTITIES_KEY, &identities)?;
        };
        Ok(())
    }
}

impl std::iter::FromIterator<String> for Identities {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

// --- Commands ---

/// Fetch the current identities that are logged in for a given user
/// These are the Nostr hexpubkeys for each identity
/// The secret store should have a matching hex private key for each pubkey
#[tauri::command]
pub fn get_identities(state: State<'_, AppState>) -> Option<Identities> {
    let db = state.db.clone();
    Identities::from_database(&db).expect("Couldn't read from database")
}

/// Fetch the currently active identity from settings
#[tauri::command]
pub fn get_current_identity(state: State<'_, AppState>) -> Option<String> {
    let db = state.db.clone();
    let settings = AppSettings::from_database(&db).expect("Couldn't read settings from database");
    settings.current_identity
}

/// Change the currently active identity
#[tauri::command]
pub async fn set_current_identity(
    pubkey: String,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let db = state.db.clone();

    let mut settings =
        AppSettings::from_database(&db).expect("Couldn't read settings from database");
    settings.current_identity = Some(pubkey.clone());
    settings.save(&db).expect("Couldn't save settings");

    // Update the nostr client with the new signer
    let keys =
        secrets_store::get_nostr_keys_for_pubkey(pubkey.as_str()).map_err(|e| e.to_string())?;
    nostr::update_signer_with_keys(keys).await;
    app_handle
        .emit("identity_change", ())
        .expect("Couldn't emit event");
    Ok(pubkey)
}

/// Create a new identity keypair
/// For new users or for creating ephemeral identities
#[tauri::command]
pub async fn create_identity(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let keys = Keys::generate();
    add_identity(keys.clone(), state).await;
    app_handle
        .emit("identity_change", ())
        .expect("Couldn't emit event");
    Ok(keys.public_key().to_string())
}

/// Log in with an nostr private key (nsec or hex)
#[tauri::command]
pub async fn login(
    nsec_or_hex: String,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let keys = Keys::parse(nsec_or_hex).expect("Couldn't parse keys from provided secret key");
    let pubkey = keys.public_key().to_string();

    // Check if the identity already exists
    if let Some(identities) = get_identities(state.clone()) {
        if identities.0.contains(&pubkey) {
            return Ok(pubkey);
        }
    }

    add_identity(keys.clone(), state.clone()).await;
    app_handle
        .emit("identity_change", ())
        .expect("Couldn't emit event");
    Ok(pubkey)
}

/// Log out of the current identity, removing it from the secrets store
#[tauri::command]
pub fn logout(pubkey: String, state: State<'_, AppState>, app_handle: tauri::AppHandle) {
    let db = state.db.clone();
    let mut identities = get_identities(state.clone()).unwrap_or(Identities(vec![]));
    let mut settings = state.settings.lock().expect("Couldn't lock settings");

    identities.remove(&pubkey);
    identities.save(&db).expect("Couldn't save identities");

    // Only attempt to remove the key if it exists
    if let Err(e) = secrets_store::remove_private_key_for_pubkey(pubkey.as_str()) {
        eprintln!("Error removing private key: {}", e);
        // Continue execution even if key removal fails
    }

    // Update current identity
    if identities.0.is_empty() {
        settings.current_identity = None;
    } else if settings.current_identity.as_ref() == Some(&pubkey) {
        settings.current_identity = identities.0.first().cloned();
    }

    app_handle
        .emit("identity_change", ())
        .expect("Couldn't emit identity change");

    settings.save(&db).expect("Couldn't save settings");
}

/// Decrypt a nip04 message
#[tauri::command]
pub fn nip04_decrypt(counterparty: String, message: String, state: State<'_, AppState>) -> String {
    let identity = get_current_identity(state).expect("No identity selected");
    let keys =
        secrets_store::get_nostr_keys_for_pubkey(&identity).expect("Unable to get keys for pubkey");
    let secret = keys.secret_key().expect("Failed to get secret key");
    let pubkey = PublicKey::from_hex(counterparty).expect("Invalid recipient key");

    nip04::decrypt(&secret, &pubkey, message).expect("Failed to decrypt")
}

// --- Private functions ---

/// Adds a nostr keypair the identities vector, sets the current identity,
/// and saves the private key to the secrets store.
async fn add_identity(keys: Keys, state: State<'_, AppState>) {
    let db = state.db.clone();
    let pubkey = keys.public_key().to_string();
    let identities = match get_identities(state.clone()) {
        Some(mut identities) => {
            identities.add(&pubkey);
            identities
        }
        None => Identities(vec![pubkey.clone()]),
    };
    identities.save(&db).expect("Couldn't save identities");

    {
        let mut settings = state.settings.lock().expect("Couldn't lock settings");
        // Update the current identity & save settings
        settings.current_identity = Some(pubkey.clone());
        settings.save(&db).expect("Couldn't update settings");
    } // Release the lock here

    // Update the nostr client with the new signer
    nostr::update_signer_with_keys(keys.clone()).await;

    // Save the private key to the secrets store
    let _ = secrets_store::store_private_key(keys);
}
