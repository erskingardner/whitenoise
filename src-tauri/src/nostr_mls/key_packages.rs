use super::NostrMls;
use crate::whitenoise::Whitenoise;
use log::debug;
use nostr_sdk::prelude::*;
use openmls::prelude::{tls_codec::*, *};
use openmls_basic_credential::SignatureKeyPair;
use tauri::State;
use tls_codec::Serialize;

/// methods needed
/// 4. fetch and validate a keypackage event for a user, return the keypackage ready to be used to invite to a group
///
pub fn generate_credential_with_key(identity: Vec<u8>) -> (CredentialWithKey, SignatureKeyPair) {
    let nostr_mls = NostrMls::default();
    let credential = BasicCredential::new(identity.clone());
    let signature_keypair = SignatureKeyPair::new(nostr_mls.ciphersuite.signature_algorithm())
        .expect("Error generating a signature keypair");

    debug!("MLS Credential keypair generated for {:?}", &credential);

    signature_keypair
        .store(nostr_mls.crypto_provider.storage())
        .expect("Error storing the signature keypair");

    (
        CredentialWithKey {
            credential: credential.into(),
            signature_key: signature_keypair.public().into(),
        },
        signature_keypair,
    )
}

/// This is for publishing a key package event (kind:443) that other users can
/// use to add you to a group.
#[tauri::command]
pub async fn generate_and_publish_key_package(
    pubkey: String,
    wn: State<'_, Whitenoise>,
) -> Result<(), String> {
    let nostr_mls = NostrMls::default();
    let pubkey_bytes = hex::decode(pubkey).unwrap();
    let (credential, signer) = generate_credential_with_key(pubkey_bytes);

    let key_package_bundle = KeyPackage::builder()
        .leaf_node_capabilities(Capabilities::new(
            None,
            None,
            Some(&nostr_mls.extensions),
            None,
            None,
        ))
        .build(
            nostr_mls.ciphersuite,
            &nostr_mls.crypto_provider,
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
                [nostr_mls.ciphersuite_value().to_string()],
            ),
            Tag::custom(
                TagKind::Custom("extensions".into()),
                [nostr_mls.extensions_value()],
            ),
            Tag::custom(TagKind::Custom("client".into()), ["whitenoise"]),
            Tag::custom(TagKind::Custom("relays".into()), ["ws://localhost:8080"]),
        ],
    );

    let event_output = wn
        .nostr
        .send_event_builder_to(vec!["ws://localhost:8080"], event)
        .await
        .unwrap();

    debug!(
        target: "nostr_mls::key_packages::generate_and_publish_key_package", "Event: {:?}",
        event_output
    );

    Ok(())
}

#[tauri::command]
pub async fn parse_key_package(key_package_hex: String) -> Result<KeyPackage, String> {
    let nostr_mls = NostrMls::default();
    let key_package_bytes = hex::decode(key_package_hex).expect("Couldn't decode key package hex");
    debug!(target: "nostr_mls::key_packages", "key_package_bytes: {:?}", key_package_bytes);
    let key_package_in = KeyPackageIn::tls_deserialize(&mut key_package_bytes.as_slice())
        .expect("Could not deserialize KeyPackage");

    let key_package = key_package_in
        .validate(nostr_mls.crypto_provider.crypto(), ProtocolVersion::Mls10)
        .expect("Invalid KeyPackage");

    debug!(target: "nostr_mls::key_packages", "key_package: {:?}", key_package);
    Ok(key_package)
}
