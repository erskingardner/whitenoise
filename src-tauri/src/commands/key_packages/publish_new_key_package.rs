use crate::key_packages::publish_key_package;
use crate::Whitenoise;

/// Publishes a new MLS key package for the active account to Nostr
///
/// Creates and publishes a new MLS key package event to the user's configured key package relays.
/// The key package contains the necessary cryptographic material for adding the user to MLS groups.
///
/// # Arguments
/// * `wn` - Whitenoise state containing account and Nostr clients
///
/// # Returns
/// * `Ok(())` - Key package was successfully published
/// * `Err(String)` - Error message if publishing fails
///
/// # Flow
/// 1. Gets active account's public key
/// 2. Creates new MLS key package
/// 3. Gets configured key package relays
/// 4. Builds Nostr event with key package and metadata
/// 5. Publishes event to relays
///
/// # Errors
/// Returns error if:
/// - No active account found
/// - Key package relays not configured
/// - Key package creation fails
/// - Event publishing fails
#[tauri::command]
pub async fn publish_new_key_package(wn: tauri::State<'_, Whitenoise>) -> Result<(), String> {
    publish_key_package(wn.clone())
        .await
        .map_err(|e| e.to_string())
}
