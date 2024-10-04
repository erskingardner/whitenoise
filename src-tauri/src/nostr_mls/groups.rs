use super::key_packages::{fetch_key_package_for_user, generate_credential_with_key};
use super::nostr_group_data::NostrGroupDataExtension;
use super::{DEFAULT_CIPHERSUITE, DEFAULT_EXTENSIONS};
use crate::database::Database;
use crate::nostr::is_valid_hex_pubkey;
use crate::whitenoise::Whitenoise;
use anyhow::anyhow;
use anyhow::Result;
use log::{debug, error};
use nostr_sdk::prelude::*;
use openmls::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::Add;
use std::str::from_utf8;
use tauri::State;
use tls_codec::Serialize as TlsSerialize;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct NostrMlsGroup {
    /// This is the MLS group ID, this will serve as the PK in the DB
    mls_group_id: Vec<u8>,
    /// Hex encoded (same value as the NostrGroupDataExtension)
    nostr_group_id: String,
    /// UTF-8 encoded (same value as the NostrGroupDataExtension)
    group_name: String,
    /// UTF-8 encoded (same value as the NostrGroupDataExtension)
    description: String,
    /// Hex encoded (same value as the NostrGroupDataExtension)
    admin_pubkeys: Vec<String>,
    /// Hex encoded Nostr event ID of the last message in the group
    last_message_id: Option<String>,
    /// Timestamp of the last message in the group
    last_message_at: Option<Timestamp>,
    /// URLs of the Nostr relays this group is using
    relay_urls: Vec<String>,
    /// Chat transscript
    transcript: Vec<UnsignedEvent>,
}

impl NostrMlsGroup {
    pub fn new(mls_group_id: Vec<u8>, nostr_group_data: NostrGroupDataExtension) -> Self {
        Self {
            mls_group_id,
            nostr_group_id: nostr_group_data.nostr_group_id(),
            group_name: nostr_group_data.name(),
            description: nostr_group_data.description(),
            admin_pubkeys: nostr_group_data.admin_identities(),
            relay_urls: nostr_group_data.relays(),
            last_message_id: None,
            last_message_at: None,
            transcript: Vec::new(),
        }
    }

    pub fn set_group_name(&mut self, name: String) {
        self.group_name = name;
    }

    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }

    pub fn set_admin_pubkeys(&mut self, admin_pubkeys: Vec<String>) {
        self.admin_pubkeys = admin_pubkeys;
    }

    pub fn set_relay_urls(&mut self, relay_urls: Vec<String>) {
        self.relay_urls = relay_urls;
    }

    pub fn set_last_message_id(&mut self, message_id: String) {
        self.last_message_id = Some(message_id);
    }

    pub fn set_last_message_at(&mut self, timestamp: Timestamp) {
        self.last_message_at = Some(timestamp);
    }

    pub fn add_to_transcript(&mut self, event: UnsignedEvent) {
        self.transcript.push(event);
    }

    pub fn save(&self, db: Database) -> Result<()> {
        let key: &str = from_utf8(&self.mls_group_id)?;
        let json = serde_json::to_string(self)?;
        db.insert_in_tree("groups", key, json.as_str())?;
        Ok(())
    }

    pub fn get_group(db: Database, mls_group_id: Vec<u8>) -> Result<Option<Self>> {
        let key: &str = from_utf8(&mls_group_id)?;
        let results = db.get_from_tree("groups", key)?;
        match results {
            Some(results) => {
                let group = from_utf8(&results)?;
                Ok(Some(serde_json::from_str(&group)?))
            }
            None => Ok(None),
        }
    }
}

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

    // Create default capabilities
    let capabilities: Capabilities = Capabilities::new(
        None,
        Some(&[DEFAULT_CIPHERSUITE]),
        Some(DEFAULT_EXTENSIONS),
        None,
        None,
    );

    // This also stores the signer secret key in the keystore
    let (credential, signer) = generate_credential_with_key(creator_pubkey.clone(), wn.clone());

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

    let mut group = MlsGroup::new(
        &wn.nostr_mls.provider,
        &signer,
        &group_config,
        credential.clone(),
    )
    .expect("Couldn't create group");

    debug!(target: "nostr_mls::groups::create_group", "Group created: ID = {:?}", group.group_id());

    // Check out group data
    let group_data = NostrGroupDataExtension::from_group(&group).expect("Failed to get group data");
    debug!(target: "nostr_mls::groups::create_group", "Nostr Group ID: {:?}", group_data.nostr_group_id());
    debug!(target: "nostr_mls::groups::create_group", "Group name: {:?}", group_data.name());
    debug!(target: "nostr_mls::groups::create_group", "Group description: {:?}", group_data.description());
    debug!(target: "nostr_mls::groups::create_group",
        "Group admin identities: {:?}",
        group_data.admin_identities()
    );

    debug!(target: "nostr_mls::groups::create_group", "Member key packages: {:?}", member_key_packages.len());
    // Add members to the group
    let (_, welcome_out, _group_info) = group
        .add_members(
            &wn.nostr_mls.provider,
            &signer,
            member_key_packages.as_slice(),
        )
        .map_err(|e| {
            error!(target: "nostr_mls::groups::create_group", "Failed to add members: {:?}", e);
            e
        })
        .expect("Failed to add members");

    debug!(target: "nostr_mls::groups::create_group", "Added members to group");

    // Merge the pending commit adding the memebers
    group
        .merge_pending_commit(&wn.nostr_mls.provider)
        .expect("Failed to merge pending commit");

    // Serialize the welcome message and send it to the members
    let serialized_welcome_message = welcome_out
        .tls_serialize_detached()
        .expect("Failed to serialize welcome message");

    // TODO: need to have a good way to get/keep relay data around.
    // TODO: need to include the ratchet tree in the welcome message so the client can bootstrap

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

        // TODO: We'll probably want to refactor this to be async eventually.
        let wrapped_event =
            EventBuilder::gift_wrap(&keys, &member_pubkey, welcome_rumor, Some(one_month_future))
                .expect("Failed to build gift wrapped welcome message");

        wn.nostr
            .send_event_to(vec!["ws://localhost:8080"], wrapped_event)
            .await
            .unwrap();

        debug!(target: "nostr_mls::groups::create_group",
            "Published welcome message to {:?}",
            &member_pubkey
        );
    }

    let nostr_group = NostrMlsGroup::new(group.group_id().to_vec(), group_data);
    nostr_group.save(wn.wdb);

    // TODO: Render a group in the UI for the saved nostr_group
    Ok(())
}

// #[tauri::command]
// pub async fn process_welcome_message(
//     welcome_message: String,
//     wn: State<'_, Whitenoise>,
// ) -> Result<(), String> {
//     Ok(())
// }
