use crate::nostr::DEFAULT_TIMEOUT;
use crate::nostr_mls::{DEFAULT_CIPHERSUITE, DEFAULT_EXTENSIONS};
use crate::whitenoise::Whitenoise;
use anyhow::Result;
use log::debug;
use nostr_sdk::prelude::*;
use openmls::prelude::{tls_codec::*, *};
use openmls_basic_credential::SignatureKeyPair;
use tauri::State;

/// Generates a credential with a key for MLS (Messaging Layer Security) operations.
///
/// This function creates a new credential and associated signature key pair for use in MLS.
/// It uses the default NostrMls configuration and stores the generated key pair in the
/// crypto provider's storage.
///
/// # Arguments
///
/// * `identity` - A vector of bytes representing the identity for which the credential is being created.
///
/// # Returns
///
/// A tuple containing:
/// * `CredentialWithKey` - The generated credential along with its public key.
/// * `SignatureKeyPair` - The generated signature key pair.
///
/// # Panics
///
/// This function will panic if:
/// * It fails to generate a signature key pair.
/// * It fails to store the signature key pair in the crypto provider's storage.
pub fn generate_credential_with_key(
    identity: String,
    wn: State<'_, Whitenoise>,
) -> (CredentialWithKey, SignatureKeyPair) {
    let credential = BasicCredential::new(identity.clone().into());
    let signature_keypair = SignatureKeyPair::new(wn.nostr_mls.ciphersuite.signature_algorithm())
        .expect("Error generating a signature keypair");

    debug!("MLS Credential keypair generated for {:?}", &identity);

    signature_keypair
        .store(wn.nostr_mls.provider.storage())
        .expect("Error storing the signature keypair");

    (
        CredentialWithKey {
            credential: credential.into(),
            signature_key: signature_keypair.public().into(),
        },
        signature_keypair,
    )
}

/// Generates and publishes a key package for a given public key.
///
/// This function creates a new key package for the specified public key and publishes it
/// as a Nostr event.
///
/// # Arguments
///
/// * `pubkey` - A String containing the hexadecimal representation of the public key.
/// * `wn` - A Tauri State containing a Whitenoise instance, which provides access to Nostr functionality.
///
/// # Returns
///
/// * `Result<(), String>` - A Result that is Ok(()) if the key package was successfully generated and published,
///   or an Err with a descriptive error message if any step of the process failed.
///
/// # Errors
///
/// This function may return an error if:
/// - The public key cannot be decoded from hexadecimal.
/// - There's an error generating the credential or key package.
/// - The key package cannot be serialized.
/// - There's an error publishing the Nostr event.
///
#[tauri::command]
pub async fn generate_and_publish_key_package(
    pubkey: String,
    wn: State<'_, Whitenoise>,
) -> Result<(), String> {
    let (credential, signer) = generate_credential_with_key(pubkey, wn.clone());

    let capabilities: Capabilities = Capabilities::new(
        None,
        Some(&[DEFAULT_CIPHERSUITE]),
        Some(DEFAULT_EXTENSIONS),
        None,
        None,
    );

    let key_package_bundle = KeyPackage::builder()
        .leaf_node_capabilities(capabilities)
        .build(
            wn.nostr_mls.ciphersuite,
            &wn.nostr_mls.provider,
            &signer,
            credential,
        )
        .expect("Error generating key package");

    // serialize the key package, then encode it to hex and put it in the content field
    let key_package_serialized = key_package_bundle
        .key_package()
        .tls_serialize_detached()
        .unwrap();

    let key_package_hex = hex::encode(key_package_serialized);

    let event = EventBuilder::new(
        Kind::Custom(443),
        key_package_hex,
        [
            Tag::custom(TagKind::Custom("mls_protocol_version".into()), ["1.0"]),
            Tag::custom(
                TagKind::Custom("ciphersuite".into()),
                [wn.nostr_mls.ciphersuite_value().to_string()],
            ),
            Tag::custom(
                TagKind::Custom("extensions".into()),
                [wn.nostr_mls.extensions_value()],
            ),
            Tag::custom(TagKind::Custom("client".into()), ["whitenoise"]),
            Tag::custom(TagKind::Custom("relays".into()), ["ws://localhost:8080"]),
        ],
    );

    wn.nostr
        .send_event_builder_to(vec!["ws://localhost:8080"], event)
        .await
        .unwrap();

    debug!(
        target: "nostr_mls::key_packages::generate_and_publish_key_package", "Key package event published");

    Ok(())
}

/// Parses a hexadecimal representation of a KeyPackage and returns the validated KeyPackage.
///
/// This function takes a hexadecimal string representing a serialized KeyPackage,
/// deserializes it, and validates it according to the MLS protocol version 1.0.
///
/// # Arguments
///
/// * `key_package_hex` - A String containing the hexadecimal representation of the KeyPackage.
///
/// # Returns
///
/// * `Result<KeyPackage, String>` - A Result containing the validated KeyPackage if successful,
///   or an error message as a String if parsing or validation fails.
///
/// # Errors
///
/// This function will return an error if:
/// - The hexadecimal string cannot be decoded
/// - The KeyPackage cannot be deserialized
/// - The KeyPackage fails validation (it's not an MLS 1.0 KeyPackage)
///
#[tauri::command]
pub fn parse_key_package(
    key_package_hex: String,
    wn: State<'_, Whitenoise>,
) -> Result<KeyPackage, String> {
    let key_package_bytes = hex::decode(key_package_hex).map_err(|e| e.to_string())?;

    let key_package_in = KeyPackageIn::tls_deserialize(&mut key_package_bytes.as_slice())
        .map_err(|e| format!("Could not deserialize KeyPackage: {}", e))?;

    let key_package = key_package_in
        .validate(wn.nostr_mls.provider.crypto(), ProtocolVersion::Mls10)
        .map_err(|e| format!("Invalid KeyPackage: {}", e))?;

    Ok(key_package)
}

/// Fetches a valid key package for a given user from the Nostr network.
///
/// This function retrieves key package events for a specified user, parses them,
/// and returns the first valid key package that matches the default ciphersuite
/// and extensions.
///
/// # Arguments
///
/// * `pubkey` - A string slice containing the public key of the user.
/// * `wn` - A Tauri state containing the Whitenoise instance.
///
/// # Returns
///
/// * `Result<Option<KeyPackage>, String>` - A Result containing:
///   - `Some(KeyPackage)` if a valid key package is found
///   - `None` if no valid key package is found
///   - An error message as a String if an error occurs during the process
///
/// # Errors
///
/// This function will return an error if:
/// - The public key is invalid
/// - There's an error fetching prekey events from the Nostr network
///
pub async fn fetch_key_package_for_user(
    pubkey: &str,
    wn: State<'_, Whitenoise>,
) -> Result<Option<KeyPackage>, String> {
    let public_key = PublicKey::from_hex(pubkey).expect("Invalid pubkey");
    let prekey_filter = Filter::new().kind(Kind::Custom(443)).author(public_key);
    let prekey_events = wn
        .nostr
        .get_events_of(
            vec![prekey_filter],
            EventSource::Both {
                timeout: Some(DEFAULT_TIMEOUT),
                specific_relays: None,
            },
        )
        .await
        .expect("Error fetching prekey events");

    let key_packages: Vec<KeyPackage> = prekey_events
        .iter()
        .filter_map(|event| parse_key_package(event.content().to_string(), wn.clone()).ok())
        .collect();

    // Get the first valid key package
    let valid_key_package = key_packages.iter().find(|&kp| {
        // Check that the ciphersuite and extensions are the same
        kp.ciphersuite() == DEFAULT_CIPHERSUITE
            && kp.leaf_node().capabilities().extensions().iter().count() == DEFAULT_EXTENSIONS.len()
            && DEFAULT_EXTENSIONS.iter().all(|&ext_type| {
                kp.leaf_node()
                    .capabilities()
                    .extensions()
                    .iter()
                    .any(|ext| ext == &ext_type)
            })
    });

    match valid_key_package {
        Some(kp) => {
            debug!(
                target: "nostr_mls::key_packages::fetch_key_package_for_user",
                "Found valid key package for user {:?}",
                pubkey
            );
            Ok(Some(kp.clone()))
        }
        None => {
            debug!(
                target: "nostr_mls::key_packages::fetch_key_package_for_user",
                "No valid key package found for user {:?}",
                pubkey
            );
            Ok(None)
        }
    }
}
