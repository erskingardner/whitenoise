use crate::account_manager::AccountError;
use crate::nostr_manager;
use crate::whitenoise::Whitenoise;
use nostr_sdk::prelude::*;
use openmls_nostr::key_packages::KeyPackage;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeyPackageError {
    #[error("No valid key package found: {0}")]
    NoValidKeyPackage(String),
    #[error("Error fetching key package: {0}")]
    FetchingKeyPackage(String),
    #[error("Account Error: {0}")]
    AccountError(#[from] AccountError),
    #[error("Nostr Error: {0}")]
    NostrError(#[from] nostr_manager::NostrManagerError),
    #[error("Nostr Client Error: {0}")]
    NostrClientError(#[from] nostr_sdk::client::Error),
    #[error("Nostr Signer Error: {0}")]
    NostrSignerError(#[from] nostr_sdk::SignerError),
    #[error("Nostr MLS Error: {0}")]
    NostrMlsError(#[from] openmls_nostr::key_packages::KeyPackageError),
}

pub type Result<T> = std::result::Result<T, KeyPackageError>;

pub async fn fetch_key_packages_for_members(
    member_pubkeys: &[String],
    wn: &tauri::State<'_, Whitenoise>,
) -> Result<Vec<KeyPackage>> {
    let mut member_key_packages: Vec<KeyPackage> = Vec::new();

    tracing::debug!(
        target: "whitenoise::key_packages::fetch_key_packages_for_members",
        "Member pubkeys: {:?}",
        member_pubkeys
    );

    // Check that members are valid pubkeys & fetch prekeys
    for pubkey in member_pubkeys.iter() {
        // Fetch prekeys from the members
        let key_package = match fetch_key_package_for_pubkey(pubkey.clone(), wn).await {
            Ok(kp) => match kp {
                Some(kp) => kp,
                None => {
                    return Err(KeyPackageError::NoValidKeyPackage(format!(
                        "No valid prekey found for member: {}",
                        pubkey
                    )));
                }
            },
            Err(_) => {
                return Err(KeyPackageError::FetchingKeyPackage(format!(
                    "Error fetching valid prekey for member: {}",
                    pubkey
                )));
            }
        };
        member_key_packages.push(key_package);
    }
    Ok(member_key_packages)
}

pub async fn fetch_key_package_for_pubkey(
    pubkey: String,
    wn: &tauri::State<'_, Whitenoise>,
) -> Result<Option<KeyPackage>> {
    tracing::debug!(target: "whitenoise::key_packages::fetch_key_package_for_pubkey", "Fetching key package for pubkey: {:?}", pubkey);
    let public_key = PublicKey::from_hex(pubkey.clone()).expect("Invalid pubkey");
    let prekey_filter = Filter::new().kind(Kind::MlsKeyPackage).author(public_key);
    let prekey_events = wn
        .nostr
        .client
        .fetch_events(vec![prekey_filter], Some(wn.nostr.timeout()?))
        .await
        .expect("Error fetching prekey events");

    let key_packages: Vec<KeyPackage> = prekey_events
        .iter()
        .filter_map(|event| {
            let nostr_mls = wn.nostr_mls.lock().expect("Failed to lock nostr_mls");
            openmls_nostr::key_packages::parse_key_package(event.content.to_string(), &nostr_mls)
                .ok()
        })
        .collect();

    // Get the first valid key package
    let valid_key_package = key_packages.iter().find(|&kp| {
        // Check that the ciphersuite and extensions are the same
        let nostr_mls = wn.nostr_mls.lock().expect("Failed to lock nostr_mls");
        kp.ciphersuite() == nostr_mls.ciphersuite
            && kp.last_resort()
            && kp.leaf_node().capabilities().extensions().len() == nostr_mls.extensions.len()
            && nostr_mls.extensions.iter().all(|&ext_type| {
                kp.leaf_node()
                    .capabilities()
                    .extensions()
                    .iter()
                    .any(|ext| ext == &ext_type)
            })
    });

    match valid_key_package {
        Some(kp) => {
            tracing::debug!(
                target: "whitenoise::key_packages::fetch_key_package_for_pubkey",
                "Found valid key package for user {:?}",
                pubkey.clone()
            );
            Ok(Some(kp.clone()))
        }
        None => {
            tracing::debug!(
                target: "whitenoise::key_packages::fetch_key_package_for_pubkey",
                "No valid key package found for user {:?}",
                pubkey
            );
            Ok(None)
        }
    }
}

/// Deletes a specific key package event from Nostr relays.
///
/// This function performs the following steps:
/// 1. Retrieves the relays associated with key packages for the current identity.
/// 2. Fetches the specific key package event from the Nostr network.
/// 3. Verifies that the event is a valid key package event and is authored by the current user.
/// 4. Creates and sends a delete event for the specified key package event.
///
/// # Arguments
///
/// * `event_id` - The `EventId` of the key package event to be deleted.
/// * `wn` - A Tauri State containing a Whitenoise instance, which provides access to Nostr functionality.
///
/// # Returns
///
/// * `Result<()>` - A Result that is Ok(()) if the key package was successfully deleted,
///   or an Err with a descriptive error message if any step of the process failed.
///
/// # Errors
///
/// This function may return an error if:
/// - There's an error retrieving the key package relays for the current identity.
/// - There's an error fetching the specified event from the Nostr network.
/// - The specified event is not a key package event (Kind::KeyPackage).
/// - The specified event is not authored by the current user.
/// - There's an error creating or sending the delete event.
pub async fn delete_key_package_from_relays(
    event_id: &EventId,
    key_package_relays: &[String],
    delete_mls_stored_keys: bool,
    wn: &tauri::State<'_, Whitenoise>,
) -> Result<()> {
    let current_pubkey = wn
        .nostr
        .client
        .signer()
        .await
        .unwrap()
        .get_public_key()
        .await
        .unwrap();
    let key_package_events = wn
        .nostr
        .client
        .fetch_events(
            vec![Filter::new()
                .id(*event_id)
                .kind(Kind::MlsKeyPackage)
                .author(current_pubkey)],
            Some(wn.nostr.timeout()?),
        )
        .await?;

    if let Some(event) = key_package_events.first() {
        // Make sure we delete the private key material from MLS storage if requested
        if delete_mls_stored_keys {
            let nostr_mls = wn.nostr_mls.lock().map_err(|_| {
                KeyPackageError::NoValidKeyPackage("Failed to lock nostr_mls".to_string())
            })?;

            let key_package = openmls_nostr::key_packages::parse_key_package(
                event.content.to_string(),
                &nostr_mls,
            )
            .map_err(KeyPackageError::NostrMlsError)?;

            openmls_nostr::key_packages::delete_key_package_from_storage(key_package, &nostr_mls)
                .map_err(KeyPackageError::NostrMlsError)?;
        }
        let builder = EventBuilder::delete(vec![event.id]);
        wn.nostr
            .client
            .send_event_builder_to(key_package_relays, builder)
            .await?;
    }
    Ok(())
}
