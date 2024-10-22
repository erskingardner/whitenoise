use super::key_packages::{fetch_key_package_for_user, generate_credential_with_key};
use super::nostr_group_data::NostrGroupDataExtension;
use super::{DEFAULT_CIPHERSUITE, DEFAULT_EXTENSIONS};
use crate::nostr::{get_contact, is_valid_hex_pubkey, DEFAULT_TIMEOUT};
use crate::secrets_store::{get_export_secret_keys_for_group, store_mls_export_secret};
use crate::whitenoise::Whitenoise;
use anyhow::anyhow;
use anyhow::Result;
use log::{debug, error};
use nostr_sdk::prelude::*;
use openmls::prelude::*;
use openmls_basic_credential::SignatureKeyPair;
use serde::{Deserialize, Serialize};
use std::ops::Add;
use std::time::Instant;
use tauri::State;
use tls_codec::{Deserialize as TlsDeserialize, Serialize as TlsSerialize};
use url::Url;

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

    pub fn export_secret_as_keys(
        &self,
        mls_group: &MlsGroup,
        wn: &State<'_, Whitenoise>,
    ) -> Result<Keys> {
        let export_secret = mls_group.export_secret(
            &*wn.nostr_mls.provider.lock().unwrap(),
            "gw-key",
            b"gw-key",
            32,
        )?;
        let export_nostr_keys = Keys::parse(hex::encode(&export_secret))?;
        Ok(export_nostr_keys)
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

    let mls_group = MlsGroup::load(wn.nostr_mls.provider.lock().unwrap().storage(), &group_id)
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
        &*wn.nostr_mls.provider.lock().unwrap(),
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
            &*wn.nostr_mls.provider.lock().unwrap(),
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
        .merge_pending_commit(&*wn.nostr_mls.provider.lock().unwrap())
        .expect("Failed to merge pending commit");

    // Serialize the welcome message and send it to the members
    let serialized_welcome_message = welcome_out
        .tls_serialize_detached()
        .expect("Failed to serialize welcome message");

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
                relay_urls
                    .iter()
                    .filter_map(|r| Url::parse(r).ok())
                    .collect(),
            ))],
        )
        .to_unsigned_event(keys.public_key());

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
        .save(wn.clone())
        .expect("Failed to save nostr group state to database");

    // TODO: Render a group in the UI for the saved nostr_group
    Ok(nostr_group)
}

#[tauri::command]
pub async fn send_mls_message(
    mut group: NostrMlsGroup,
    message: String,
    wn: State<'_, Whitenoise>,
) -> Result<UnsignedEvent, String> {
    let serialized_message: Vec<u8>;
    let mut mls_group: MlsGroup;

    let nostr_keys = wn
        .accounts
        .lock()
        .unwrap()
        .get_nostr_keys_for_current_identity()
        .unwrap()
        .unwrap();

    // Create an unsigned nostr event with the message
    let mut event = UnsignedEvent::new(
        nostr_keys.public_key(),
        Timestamp::now(),
        Kind::TextNote,
        Vec::new(),
        message,
    );

    {
        let provider = wn.nostr_mls.provider.lock().unwrap();

        mls_group = MlsGroup::load(
            provider.storage(),
            &GroupId::from_slice(&group.mls_group_id),
        )
        .expect("Failed to load MLS group")
        .unwrap();

        // Ensure the event has an ID
        event.ensure_id();

        let json_event = serde_json::to_string(&event).map_err(|e| e.to_string())?;

        let signer = SignatureKeyPair::read(
            provider.storage(),
            mls_group
                .own_leaf()
                .unwrap()
                .signature_key()
                .clone()
                .as_slice(),
            mls_group.ciphersuite().signature_algorithm(),
        )
        .unwrap();

        let message_out = mls_group
            .create_message(&*provider, &signer, json_event.as_bytes())
            .map_err(|e| e.to_string())?;
        // TODO: Handle the errors that can come from create_message. use after evict and pending proposal/commit

        serialized_message = message_out
            .tls_serialize_detached()
            .map_err(|e| e.to_string())?;
    }

    let message_rumor = EventBuilder::new(
        Kind::Custom(445),
        hex::encode(serialized_message),
        Vec::new(),
    )
    .to_unsigned_event(nostr_keys.public_key());

    debug!(target: "nostr_mls::groups::send_message", "Message rumor: {:?}", message_rumor);

    let export_nostr_keys = group
        .export_secret_as_keys(&mls_group, &wn)
        .expect("Failed to get export_secret as keys");

    let wrapped_event = EventBuilder::gift_wrap_with_tags(
        &nostr_keys,
        &export_nostr_keys.public_key(),
        message_rumor,
        vec![Tag::custom(
            TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::H)),
            vec![group.nostr_group_id.clone()],
        )],
        None,
    )
    .expect("Failed to build gift wrapped message");

    debug!(target: "nostr_mls::groups::send_message", "Publishing gift wrapped event to group relays");

    let relays = if tauri::is_dev() {
        vec!["ws://localhost:8080".to_string()]
    } else {
        group.relay_urls.clone()
    };

    wn.nostr
        .send_event_to(relays, wrapped_event)
        .await
        .map_err(|e| e.to_string())?;

    group.transcript.push(event.clone());
    group.last_message_id = Some(event.id.unwrap().to_string());
    group.last_message_at = Some(Timestamp::now());
    group.save(wn.clone()).map_err(|e| e.to_string())?;

    debug!(target: "nostr_mls::groups::send_message", "Event saved: {:?}", event);
    debug!(target: "nostr_mls::groups::send_message", "Group transcript: {:?}", group.transcript);
    Ok(event)
}

#[tauri::command]
pub async fn fetch_and_process_mls_messages(
    wn: State<'_, Whitenoise>,
) -> Result<Vec<UnsignedEvent>, String> {
    debug!(target: "nostr_mls::groups::fetch_and_process_mls_messages", "Fetching and processing MLS messages");
    let nostr_groups = NostrMlsGroup::get_groups(wn.clone()).expect("Failed to get groups");

    let group_ids = &nostr_groups
        .iter()
        .map(|g| g.nostr_group_id.clone())
        .collect::<Vec<String>>();

    let group_messages_filter = Filter::new()
        .kind(Kind::GiftWrap)
        .custom_tag(SingleLetterTag::lowercase(Alphabet::H), group_ids);

    // TODO: The created at fields here are not accurate. We need to unwrap the events,
    // then get the created at from the unwrapped event and use that to sort them
    let stored_events = wn
        .nostr
        .database()
        .query(vec![group_messages_filter.clone()])
        .await
        .unwrap();

    let relay_events = wn
        .nostr
        .fetch_events(vec![group_messages_filter.clone()], Some(DEFAULT_TIMEOUT))
        .await
        .unwrap();

    let group_events = stored_events.merge(relay_events);

    debug!(target: "nostr_mls::groups::fetch_and_process_mls_messages", "Found {:?} events", group_events.len());

    // Vec of tuples of (mls_group, unwrapped_event)
    let mut unwrapped_events_with_groups: Vec<(NostrMlsGroup, MlsGroup, UnsignedEvent)> =
        Vec::new();

    for event in group_events {
        debug!(target: "nostr_mls::groups::fetch_and_process_mls_messages", "Unwrapping Message Event: {:?}", event.id);

        let nostr_group_id = event
            .tags
            .find(TagKind::SingleLetter(SingleLetterTag::lowercase(
                Alphabet::H,
            )))
            .unwrap()
            .content()
            .unwrap();

        // Get the nostr_mls_group for this event. Log an error and continue if we don't have it
        let nostr_group = match nostr_groups
            .iter()
            .find(|g| g.nostr_group_id == nostr_group_id)
        {
            Some(g) => g,
            None => {
                error!(target: "nostr_mls::groups::fetch_and_process_mls_messages", "No group found for nostr group id: {:?}", nostr_group_id);
                continue;
            }
        };

        // Get the MLS group for this nostr_mls_group. Log an error and continue if we don't have it
        let mls_group = match MlsGroup::load(
            wn.nostr_mls.provider.lock().unwrap().storage(),
            &GroupId::from_slice(&nostr_group.mls_group_id),
        ) {
            Ok(Some(group)) => group,
            Ok(None) => {
                error!(target: "nostr_mls::groups::fetch_and_process_mls_messages", "No MLS group found for nostr group id: {:?}", nostr_group_id);
                continue;
            }
            Err(e) => {
                error!(target: "nostr_mls::groups::fetch_and_process_mls_messages", "Failed to load MLS group: {:?}", e);
                continue;
            }
        };

        // Get the export secret key for this group. If we don't have it, we need to export the secret and store it
        // We do this now in case the message is a commit that will move the epoch forward
        // We need to keep old epoch's secret keys to decrypt old messages if they arrive out of order
        let export_secret_key = match get_export_secret_keys_for_group(
            hex::encode(nostr_group.mls_group_id.clone()).as_str(),
            mls_group.epoch().as_u64(),
        ) {
            Ok(keys) => keys,
            Err(e) => {
                error!(target: "nostr_mls::groups::fetch_and_process_mls_messages", "Failed to get export secret keys for group: {:?}", e);
                // If we don't have the keys for this epoch, we need to export the secret and store it
                let epoch = &mls_group.epoch().as_u64();
                let export_nostr_keys = nostr_group
                    .export_secret_as_keys(&mls_group, &wn)
                    .expect("Failed to get export_secret as keys");
                store_mls_export_secret(
                    hex::encode(nostr_group.mls_group_id.clone()).as_str(),
                    *epoch,
                    export_nostr_keys.secret_key().to_secret_hex().as_str(),
                )
                .expect("Failed to store export secret");
                export_nostr_keys
            }
        };

        // TODO: We need to check the ids of the events we have in the transcript and skip them
        // TODO: Do we need to try and order the created at of the events first? and  then we can more easily handle them in order

        let unwrapped_event =
            extract_rumor(&export_secret_key, &event).expect("Failed to unwrap event");

        match unwrapped_event.rumor.kind {
            Kind::Custom(445) => {
                unwrapped_events_with_groups.push((
                    nostr_group.clone(),
                    mls_group,
                    unwrapped_event.rumor,
                ));
            }
            _ => {
                debug!(target: "nostr_mls::groups::fetch_and_process_mls_messages", "Not processing event with kind: {:?}", unwrapped_event.rumor.kind);
                continue;
            }
        }
    }

    // Sort the unwrapped events by created at and then by id to ensure order (specifically for commit messages)
    unwrapped_events_with_groups.sort_by(|a, b| {
        a.2.created_at.cmp(&b.2.created_at).then_with(|| {
            match (
                hex::decode(a.2.id.unwrap().to_string()),
                hex::decode(b.2.id.unwrap().to_string()),
            ) {
                (Ok(bytes_a), Ok(bytes_b)) => bytes_a.cmp(&bytes_b),
                _ => {
                    error!(target: "nostr_mls::groups::fetch_and_process_mls_messages", "UNHANDLED Error: Found an incoming message with identical created_at and id values. This should never happen.");
                    std::cmp::Ordering::Equal
                }
            }
        })
    });

    // Process unwrapped events
    for (mut nostr_group, mut mls_group, event) in unwrapped_events_with_groups {
        debug!(target: "nostr_mls::groups::fetch_and_process_mls_messages", "Processing Message Event: {:?}", event.id);
        let mls_message = MlsMessageIn::tls_deserialize_exact(hex::decode(&event.content).unwrap())
            .expect("Failed to deserialize MLSMessageIn");

        let protocol_message = mls_message
            .try_into_protocol_message()
            .expect("Failed to convert MLSMessageIn to ProtocolMessage");

        match protocol_message.group_id() == mls_group.group_id() {
            true => {
                let processed_message = mls_group
                    .process_message(&*wn.nostr_mls.provider.lock().unwrap(), protocol_message)
                    .expect("Failed to process message");

                let sender = BasicCredential::try_from(processed_message.credential().clone())
                    .expect("Couldn't get sender credential");

                if PublicKey::parse(hex::encode(sender.identity())).unwrap()
                    == wn
                        .accounts
                        .lock()
                        .unwrap()
                        .get_nostr_keys_for_current_identity()
                        .unwrap()
                        .unwrap()
                        .public_key()
                {
                    debug!(target: "nostr_mls::groups::fetch_and_process_mls_messages", "Sender is the current identity. Not processing message: {:?}", &event.id);
                    continue;
                }

                // Handle the processed message based on its type
                match processed_message.into_content() {
                    ProcessedMessageContent::ApplicationMessage(application_message) => {
                        // This is a message from a group member
                        let message_bytes = application_message.into_bytes();
                        debug!(target: "nostr_mls::groups::fetch_and_process_mls_messages", "Raw message bytes: {:?}", message_bytes);

                        match serde_json::from_slice::<serde_json::Value>(&message_bytes) {
                            Ok(json_value) => {
                                debug!(target: "nostr_mls::groups::fetch_and_process_mls_messages", "Deserialized JSON message: {}", json_value);
                                let json_str = json_value.to_string();
                                let json_event = UnsignedEvent::from_json(&json_str).unwrap();
                                // Check to make sure we don't already have this event in the transcript
                                if !nostr_group.transcript.iter().any(|e| e.id == json_event.id) {
                                    nostr_group.transcript.push(json_event);
                                }
                            }
                            Err(e) => {
                                error!(target: "nostr_mls::groups::fetch_and_process_mls_messages", "Failed to deserialize message into JSON: {}", e);
                            }
                        }
                    }
                    ProcessedMessageContent::ProposalMessage(staged_proposal) => {
                        // This is a proposal message
                        debug!(target: "nostr_mls::groups::fetch_and_process_mls_messages", "Received proposal message: {:?}", staged_proposal);
                        // TODO: Handle proposal message
                    }
                    ProcessedMessageContent::StagedCommitMessage(staged_commit) => {
                        // This is a commit message
                        debug!(target: "nostr_mls::groups::fetch_and_process_mls_messages", "Received commit message: {:?}", staged_commit);
                        // TODO: Handle commit message
                    }
                    // TODO: Handle external join proposal
                    _ => {
                        // Handle any other cases
                        debug!(target: "nostr_mls::groups::fetch_and_process_mls_messages", "Received unhandled message type");
                    }
                }
            }
            false => {
                error!(target: "nostr_mls::groups::fetch_and_process_mls_messages", "ProtocolMessage GroupId doesn't match MlsGroup GroupId. Not processing rumor event: {:?}", &event.id);
                continue;
            }
        }
    }

    Ok(Vec::new())
}
