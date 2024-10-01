use super::key_packages::{fetch_key_package_for_user, generate_credential_with_key};
use super::nostr_group_data::NostrGroupDataExtension;
use super::{DEFAULT_CIPHERSUITE, DEFAULT_EXTENSIONS};
use crate::nostr::is_valid_hex_pubkey;
use crate::whitenoise::Whitenoise;
use anyhow::anyhow;
use anyhow::Result;
use log::debug;
use nostr_sdk::prelude::*;
use openmls::prelude::*;
use std::ops::Add;
use tauri::State;
use tls_codec::Serialize;

#[tauri::command]
pub async fn create_group(
    creator_pubkey: String,
    member_pubkeys: Vec<String>,
    admin_pubkeys: Vec<String>,
    group_name: String,
    description: String,
    wn: State<'_, Whitenoise>,
) -> Result<(), String> {
    // Check pubkey args to make sure they're correct

    // Creator must be an admin
    if !admin_pubkeys.contains(&creator_pubkey) {
        return Err(anyhow!("Creator must be an admin").to_string());
    }

    // Creator must not be included as a member
    if member_pubkeys.contains(&creator_pubkey) {
        return Err(anyhow!("Creator must not be included as a member").to_string());
    }

    // Creator must be valid pubkey
    if !is_valid_hex_pubkey(&creator_pubkey) {
        return Err(anyhow!("Invalid creator pubkey: {}", creator_pubkey).to_string());
    }

    // Check that admins are valid pubkeys and are members
    for pubkey in admin_pubkeys.iter() {
        if !is_valid_hex_pubkey(pubkey) {
            return Err(anyhow!("Invalid admin pubkey: {}", pubkey).to_string());
        }
        if !member_pubkeys.contains(pubkey) && &creator_pubkey != pubkey {
            return Err(anyhow!("Admin must be a member").to_string());
        }
    }

    let mut member_key_packages: Vec<KeyPackage> = Vec::new();

    // Check that members are valid pubkeys & fetch prekeys
    for pubkey in member_pubkeys.iter() {
        if !is_valid_hex_pubkey(pubkey) {
            return Err(anyhow!("Invalid member pubkey: {}", pubkey).to_string());
        }

        // Fetch prekeys from the members
        let key_package = match fetch_key_package_for_user(pubkey, wn.clone()).await {
            Ok(kp) => match kp {
                Some(kp) => kp,
                None => {
                    return Err(anyhow!("No valid prekey found for member: {}", pubkey).to_string())
                }
            },
            Err(_) => {
                return Err(
                    anyhow!("Error fetching valid prekey for member: {}", pubkey).to_string(),
                );
            }
        };

        member_key_packages.push(key_package);
    }

    let provider = &openmls_libcrux_crypto::Provider::default();

    // Create default capabilities
    let capabilities: Capabilities = Capabilities::new(
        None,
        Some(&[DEFAULT_CIPHERSUITE]),
        Some(DEFAULT_EXTENSIONS),
        None,
        None,
    );

    // This also stores the signer secret key in the keystore
    let (credential, signer) = generate_credential_with_key(creator_pubkey.clone());

    // Create the nostr group data extension and serialize it
    let group_data = NostrGroupDataExtension::new(group_name, description, admin_pubkeys);

    let serialized_group_data = group_data
        .tls_serialize_detached()
        .expect("Failed to serialize group data");

    let extensions = vec![Extension::Unknown(
        group_data.extension_type(),
        UnknownExtension(serialized_group_data),
    )];

    // Build the group config
    let group_config = MlsGroupCreateConfig::builder()
        .ciphersuite(DEFAULT_CIPHERSUITE)
        .capabilities(capabilities)
        .with_group_context_extensions(
            Extensions::from_vec(extensions).expect("Couldn't convert extensions vec to Object"),
        )
        .expect("Couldn't set group context extensions")
        .build();

    let mut group = MlsGroup::new(provider, &signer, &group_config, credential.clone())
        .expect("Couldn't create group");

    // Check out group data
    let group_data = NostrGroupDataExtension::from_group(&group).expect("Failed to get group data");
    debug!(target: "nostr_mls::groups::create_group", "Group ID: {:?}", group_data.get_id());
    debug!(target: "nostr_mls::groups::create_group", "Group name: {:?}", group_data.get_name());
    debug!(target: "nostr_mls::groups::create_group", "Group description: {:?}", group_data.get_description());
    debug!(target: "nostr_mls::groups::create_group",
        "Group admin identities: {:?}",
        group_data.get_admin_identities()
    );

    debug!(target: "nostr_mls::groups::create_group", "Member key packages: {:?}", member_key_packages.len());
    // Add members to the group
    let (_, welcome_out, _group_info) = group
        .add_members(provider, &signer, &member_key_packages)
        .unwrap();

    // Merge the pending commit adding the memebers
    group
        .merge_pending_commit(provider)
        .expect("Failed to merge pending commit");

    // Serialize the welcome message and send it to the members
    let serialized_welcome_message = welcome_out
        .tls_serialize_detached()
        .expect("Failed to serialize welcome message");

    // TODO: need to have a good way to get/keep relay data around.

    let signer = wn
        .nostr
        .clone()
        .signer()
        .await
        .expect("Failed to get nostr signer");

    let keys: Keys = wn
        .accounts
        .lock()
        .unwrap()
        .get_nostr_keys_for_current_identity()
        .expect("Failed to get nostr keys")
        .unwrap();

    for member in member_pubkeys {
        let member_pubkey = PublicKey::from_hex(member).expect("Invalid pubkey");
        let welcome_rumor = EventBuilder::new(
            Kind::Custom(444),
            hex::encode(&serialized_welcome_message),
            vec![Tag::from_standardized_without_cell(TagStandard::Relays(
                vec!["ws://localhost:8080".into()],
            ))],
        )
        .to_unsigned_event(signer.public_key().await.unwrap());

        debug!(target: "nostr_mls::groups::create_group", "Welcome rumor: {:?}", welcome_rumor);

        // Create a timestamp 1 month in the future
        let one_month_future = Timestamp::now().add(30 * 24 * 60 * 60);

        let wrapped_event =
            EventBuilder::gift_wrap(&keys, &member_pubkey, welcome_rumor, Some(one_month_future));

        debug!(target: "nostr_mls::groups::create_group",
            "Sending welcome message to {:?}: {:?}",
            &member_pubkey, &wrapped_event
        );
    }
    // TODO: save group to database

    Ok(())
}

// #[tauri::command]
// pub async fn process_welcome_message(
//     welcome_message: String,
//     wn: State<'_, Whitenoise>,
// ) -> Result<(), String> {
//     Ok(())
// }
