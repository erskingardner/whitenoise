use super::key_packages::{fetch_key_package_for_user, generate_credential_with_key};
use super::nostr_group_data::NostrGroupDataExtension;
use super::{DEFAULT_CIPHERSUITE, DEFAULT_EXTENSIONS};
use crate::nostr::{get_contact, is_valid_hex_pubkey};
use crate::whitenoise::Whitenoise;
use anyhow::anyhow;
use anyhow::Result;
use log::{debug, error};
use nostr_sdk::prelude::*;
use openmls::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::Add;
use std::time::Instant;
use tauri::State;
use tls_codec::Serialize as TlsSerialize;

/// Key used to store and retrieve groups data in the database
const GROUPS_KEY: &str = "groups";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NostrMlsGroup {
    /// This is the MLS group ID, this will serve as the PK in the DB and doesn't change
    pub mls_group_id: Vec<u8>,
    /// Hex encoded (same value as the NostrGroupDataExtension) this is the group_id used in Nostr events
    pub nostr_group_id: String,
    /// UTF-8 encoded (same value as the NostrGroupDataExtension)
    pub group_name: String,
    /// UTF-8 encoded (same value as the NostrGroupDataExtension)
    pub description: String,
    /// Hex encoded (same value as the NostrGroupDataExtension)
    pub admin_pubkeys: Vec<String>,
    /// Hex encoded Nostr event ID of the last message in the group
    pub last_message_id: Option<String>,
    /// Timestamp of the last message in the group
    pub last_message_at: Option<Timestamp>,
    /// URLs of the Nostr relays this group is using
    pub relay_urls: Vec<String>,
    /// Type of Nostr MLS group
    pub group_type: NostrMlsGroupType,
    /// Chat transscript
    pub transcript: Vec<UnsignedEvent>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NostrMlsGroupType {
    DirectMessage,
    Group,
}

impl NostrMlsGroup {
    /// Returns the tree key for the Nostr MLS groups
    /// This scopes the database tree to the current user.
    fn group_tree_key(wn: State<'_, Whitenoise>) -> Result<String> {
        let accounts = wn.accounts.lock().expect("Failed to lock accounts");
        let current_identity = match &accounts.current_identity {
            Some(identity) => identity,
            None => return Err(anyhow!("No current identity found")),
        };
        Ok(format!("{}{}", GROUPS_KEY, current_identity))
    }

    pub fn new(mls_group_id: Vec<u8>, nostr_group_data: NostrGroupDataExtension) -> Self {
        Self {
            mls_group_id,
            nostr_group_id: nostr_group_data.nostr_group_id(),
            group_name: nostr_group_data.name(),
            description: nostr_group_data.description(),
            admin_pubkeys: nostr_group_data.admin_identities(),
            last_message_id: None,
            last_message_at: Some(Timestamp::now()),
            relay_urls: nostr_group_data.relays(),
            group_type: NostrMlsGroupType::DirectMessage,
            transcript: Vec::new(),
        }
    }

    pub fn save(&self, wn: State<'_, Whitenoise>) -> Result<()> {
        let key = hex::encode(&self.mls_group_id);
        let json = serde_json::to_string(self)?;
        debug!(target: "nostr_mls::groups::save", "Saving group {:?}: {:?}", key, json);
        wn.wdb.insert_in_tree(
            NostrMlsGroup::group_tree_key(wn.clone())?.as_str(),
            key.as_str(),
            json.as_str(),
        )?;
        Ok(())
    }

    pub fn get_groups(wn: State<'_, Whitenoise>) -> Result<Vec<NostrMlsGroup>> {
        let group_tree = wn
            .wdb
            .db
            .open_tree(NostrMlsGroup::group_tree_key(wn.clone())?)?;
        let groups = group_tree
            .iter()
            .filter_map(|result| {
                result.ok().and_then(|(_, value)| {
                    serde_json::from_str::<NostrMlsGroup>(std::str::from_utf8(&value).ok()?).ok()
                })
            })
            .collect::<Vec<NostrMlsGroup>>();
        debug!(target: "nostr_mls::groups::get_groups", "Loaded {:?} groups", groups.len());
        debug!(target: "nostr_mls::groups::get_groups", "Groups: {:?}", groups.iter().map(|g| GroupId::from_slice(&g.mls_group_id)).collect::<Vec<GroupId>>());
        Ok(groups)
    }
}

#[tauri::command]
pub async fn get_groups(wn: State<'_, Whitenoise>) -> Result<Vec<NostrMlsGroup>, String> {
    let start = Instant::now();
    let groups = NostrMlsGroup::get_groups(wn).expect("Failed to get groups");
    debug!(target: "nostr_mls::groups::get_groups", "{:?} groups found in {:?}", groups.len(), start.elapsed());
    Ok(groups)
}

#[tauri::command]
pub async fn get_group_member_pubkeys(
    mls_group_id: Vec<u8>,
    wn: State<'_, Whitenoise>,
) -> Result<Vec<String>, String> {
    let group_id = GroupId::from_slice(&mls_group_id);
    debug!(target: "nostr_mls::groups::get_group_member_pubkeys", "Group ID: {:?}", group_id);
    let mls_group = MlsGroup::load(wn.nostr_mls.provider.storage(), &group_id)
        .expect("Failed to load MLS group");
    debug!(target: "nostr_mls::groups::get_group_member_pubkeys", "MLS group loaded: {:?}", mls_group);
    let member_pubkeys: Vec<String> = match mls_group {
        Some(group) => group
            .members()
            .map(|m| {
                let credential = BasicCredential::try_from(m.credential)
                    .expect("Failed to deserialize credential");
                let identity = credential.identity();
                let pubkey = String::from_utf8_lossy(identity).to_string();
                debug!(target: "nostr_mls::groups::get_group_member_pubkeys", "Member identity: {:?}", pubkey);
                pubkey
            })
            .collect::<Vec<String>>(),
        None => Vec::new(),
    };
    debug!(target: "nostr_mls::groups::get_group_member_pubkeys", "Member pubkeys: {:?}", member_pubkeys);
    Ok(member_pubkeys)
}

#[tauri::command]
pub async fn create_group(
    creator_pubkey: String,
    member_pubkeys: Vec<String>,
    admin_pubkeys: Vec<String>,
    group_name: String,
    description: String,
    wn: State<'_, Whitenoise>,
) -> Result<NostrMlsGroup, String> {
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
    debug!(target: "nostr_mls::groups::create_group", "Member key packages: {:?}", member_key_packages.len());
    let identities = member_key_packages
        .iter()
        .map(|kp| {
            let cred = BasicCredential::try_from(kp.leaf_node().credential().clone());
            match cred {
                Ok(cred) => {
                    let identity = cred.identity();
                    String::from_utf8_lossy(identity).to_string()
                }
                Err(e) => {
                    error!(target: "nostr_mls::groups::create_group", "Failed to deserialize credential: {:?}", e);
                    "".to_string()
                }
            }
        })
        .collect::<Vec<String>>();
    debug!(target: "nostr_mls::groups::create_group", "Member identities: {:?}", identities);

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
    let group_data = NostrGroupDataExtension::new(
        group_name,
        description,
        admin_pubkeys,
        vec!["ws://localhost:8080".into()], // TODO: need to do a better job wth relays
    );

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
        .use_ratchet_tree_extension(true)
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
        let member_pubkey = PublicKey::from_hex(&member).expect("Invalid pubkey");
        let contact = get_contact(member, wn.clone()).await.unwrap();

        // If we're in dev mode, use the local relay, otherwise use the relays from the contact
        let relay_urls = if tauri::is_dev() {
            vec!["ws://localhost:8080".to_string()]
        } else {
            contact.inbox_relays
        };

        let welcome_rumor = EventBuilder::new(
            Kind::Custom(444),
            hex::encode(&serialized_welcome_message),
            vec![Tag::from_standardized_without_cell(TagStandard::Relays(
                relay_urls.iter().map(|r| r.to_string().into()).collect(),
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

        let max_retries = 5;
        let mut retry_count = 0;
        let mut last_error = None;

        while retry_count < max_retries {
            match wn
                .nostr
                .send_event_to(relay_urls.clone(), wrapped_event.clone())
                .await
            {
                Ok(_) => {
                    // Successfully sent, break the loop
                    break;
                }
                Err(e) => {
                    last_error = Some(e);
                    retry_count += 1;
                    if retry_count < max_retries {
                        // Wait for a short time before retrying
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                }
            }
        }

        if retry_count == max_retries {
            return Err(format!(
                "Failed to send event after {} attempts. Last error: {:?}",
                max_retries, last_error
            ));
        }

        debug!(target: "nostr_mls::groups::create_group",
            "Published welcome message to {:?}",
            &member_pubkey
        );
    }
    debug!(target: "nostr_mls::groups::create_group", "Group ID: {:?}", group.group_id());
    debug!(target: "nostr_mls::groups::create_group", "Group ID Vec: {:?}", group.group_id().to_vec());
    let nostr_group = NostrMlsGroup::new(group.group_id().to_vec(), group_data);
    debug!(target: "nostr_mls::groups::create_group", "Saving group to database: {:?}", nostr_group);
    nostr_group
        .save(wn)
        .expect("Failed to save nostr group state to database");

    // TODO: Render a group in the UI for the saved nostr_group
    Ok(nostr_group)
}

// #[tauri::command]
// pub async fn process_welcome_message(
//     welcome_message: String,
//     wn: State<'_, Whitenoise>,
// ) -> Result<(), String> {
//     Ok(())
// }
