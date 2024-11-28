use crate::{key_packages::fetch_key_package_for_pubkey, Whitenoise};
use nostr_sdk::event::{EventBuilder, Kind, Tag, TagKind};
use nostr_openmls::key_packages::create_key_package_for_event;

/// Checks if a valid MLS key package exists for a given user
///
/// # Arguments
/// * `pubkey` - Hex encoded Nostr public key of the user to check
/// * `wn` - Whitenoise state containing Nostr client
///
/// # Returns
/// * `Ok(bool)` - True if valid key package exists, false otherwise
/// * `Err(String)` - Error message if check fails
///
/// # Errors
/// Returns error if:
/// - Public key is invalid
/// - Network error occurs fetching key package
/// - Key package parsing fails
#[tauri::command]
pub async fn valid_key_package_exists_for_user(
    pubkey: String,
    wn: tauri::State<'_, Whitenoise>,
) -> Result<bool, String> {
    let key_package = fetch_key_package_for_pubkey(pubkey, &wn)
        .await
        .map_err(|e| e.to_string())?;
    Ok(key_package.is_some())
}

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
pub async fn publish_key_package(wn: tauri::State<'_, Whitenoise>) -> Result<(), String> {
    let pubkey = wn
        .nostr
        .client
        .signer()
        .await
        .map_err(|e| e.to_string())?
        .get_public_key()
        .await
        .map_err(|e| e.to_string())?;

    let event: EventBuilder;
    let key_package_relays: Vec<String>;
    {
        let nostr_mls = wn.nostr_mls.lock().expect("Failed to lock nostr_mls");
        let ciphersuite = nostr_mls.ciphersuite_value().to_string();
        let extensions = nostr_mls.extensions_value();

        let serialized_key_package =
            create_key_package_for_event(pubkey.to_hex(), &nostr_mls).map_err(|e| e.to_string())?;

        if cfg!(dev) {
            key_package_relays = vec!["ws://localhost:8080".to_string()];
        } else {
            key_package_relays = match wn
                .account_manager
            .get_active_account()
            .map_err(|e| e.to_string())?
            .key_package_relays
            .is_empty()
        {
            true => return Err("Key package relays not set".to_string()),
            false => wn
                .account_manager
                .get_active_account()
                .map_err(|e| e.to_string())?
                .key_package_relays
                .clone(),
            };
        }
        event = EventBuilder::new(
            Kind::MlsKeyPackage,
            serialized_key_package).tags(
            [
                Tag::custom(TagKind::MlsProtocolVersion, ["1.0"]),
                Tag::custom(TagKind::MlsCiphersuite, [ciphersuite]),
                Tag::custom(TagKind::MlsExtensions, [extensions]),
                Tag::custom(TagKind::Client, ["whitenoise"]),
                Tag::custom(TagKind::Relays, key_package_relays.clone()),
            ]
        );
    }
    wn.nostr
        .client
        .send_event_builder_to(key_package_relays.clone(), event)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn delete_all_key_packages(wn: tauri::State<'_, Whitenoise>) -> Result<(), String> {
    let pubkey = wn
        .nostr
        .client
        .signer()
        .await
        .map_err(|e| e.to_string())?
        .get_public_key()
        .await
        .map_err(|e| e.to_string())?;

    let active_account = wn.account_manager.get_active_account().map_err(|e| e.to_string())?;

    let key_package_relays: Vec<String> = if cfg!(dev) {
        vec!["ws://localhost:8080".to_string()]
    } else {
        active_account.key_package_relays.clone()
    };

    let key_package_events = wn.nostr.query_user_key_packages(pubkey).await.map_err(|e| e.to_string())?;

    if !key_package_events.is_empty() {
        let delete_event = EventBuilder::delete_with_reason(key_package_events.iter().map(|e| e.id), "Delete own key package");
        tracing::debug!(target: "whitenoise::commands::key_packages::delete_all_key_packages", "Deleting key packages: {:?}", delete_event);
        wn.nostr.client.send_event_builder_to(key_package_relays, delete_event).await.map_err(|e| e.to_string())?;
    } else {
        tracing::debug!(target: "whitenoise::commands::key_packages::delete_all_key_packages", "No key packages to delete");
    }
    Ok(())
}
