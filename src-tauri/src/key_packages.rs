use crate::account_manager::AccountError;
use crate::whitenoise::Whitenoise;
use nostr_sdk::prelude::*;
use openmls_nostr::key_packages::KeyPackage;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeyPackageError {
    #[error("Key package relays not set")]
    RelaysNotSet,
    #[error("No valid key package found: {0}")]
    NoValidKeyPackage(String),
    #[error("Error fetching key package: {0}")]
    FetchingKeyPackage(String),
    #[error("Account Error: {0}")]
    AccountError(#[from] AccountError),
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
    Ok(vec![])
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
        .fetch_events(vec![prekey_filter], Some(wn.nostr.timeout()))
        .await
        .expect("Error fetching prekey events");

    let key_packages: Vec<KeyPackage> = prekey_events
        .iter()
        .filter_map(|event| {
            openmls_nostr::key_packages::parse_key_package(event.content.to_string(), &wn.nostr_mls)
                .ok()
        })
        .collect();

    // Get the first valid key package
    let valid_key_package = key_packages.iter().find(|&kp| {
        // Check that the ciphersuite and extensions are the same
        // TODO: Do we need to check that the credential is the same?
        kp.ciphersuite() == wn.nostr_mls.ciphersuite
            && kp.last_resort()
            && kp.leaf_node().capabilities().extensions().len() == wn.nostr_mls.extensions.len()
            && wn.nostr_mls.extensions.iter().all(|&ext_type| {
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

// pub async fn delete_key_package();
