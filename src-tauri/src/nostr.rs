use crate::nostr_mls::groups::NostrMlsGroup;
use crate::whitenoise::Whitenoise;
use log::{debug, error};
use nostr_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use std::{collections::HashMap, time::Duration};
use tauri::State;

pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);
pub const DEFAULT_RELAYS: [&str; 5] = [
    "wss://relay.damus.io",
    "wss://relay.snort.social",
    "wss://relay.primal.net",
    "wss://nos.lol",
    "wss://purplepag.es",
    // "ws://localhost:8080",
];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Conversation {
    pub latest: Timestamp,
    pub metadata: Metadata,
    pub events: Vec<RawEvent>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RawEvent {
    id: String,
    created_at: Timestamp,
    content: String,
    kind: Kind,
    tags: Vec<Tag>,
    pubkey: PublicKey,
    sig: Signature,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct EnrichedContact {
    pub metadata: Metadata,
    pub nip17: bool,
    pub nip104: bool,
    pub inbox_relays: Vec<String>,
    pub key_package_relays: Vec<String>,
}

#[allow(dead_code)]
pub struct WelcomeMessage {
    pub sender: EnrichedContact,
    pub welcome_message: String,
}

pub fn is_valid_hex_pubkey(pubkey: &str) -> bool {
    pubkey.len() == 64 && pubkey.chars().all(|c| c.is_ascii_hexdigit())
}

// --- Commands ---

#[tauri::command]
pub async fn get_contact(
    pubkey: String,
    wn: State<'_, Whitenoise>,
) -> Result<EnrichedContact, String> {
    let public_key = PublicKey::from_hex(&pubkey).unwrap();
    let metadata = wn
        .nostr
        .fetch_metadata(public_key, Some(DEFAULT_TIMEOUT))
        .await
        .unwrap_or(Metadata::default());

    let mut enriched_contact = EnrichedContact {
        metadata,
        nip17: false,
        nip104: false,
        inbox_relays: Vec::new(),
        key_package_relays: Vec::new(),
    };

    // Prepare filters for messaging capabilities
    let dm_relay_list_filter = Filter::new().kind(Kind::Custom(10050)).author(public_key);
    let prekey_filter = Filter::new().kind(Kind::KeyPackage).author(public_key);
    let key_package_list_filter = Filter::new().kind(Kind::Custom(10051)).author(public_key);

    // Fetch messaging capabilities for all contacts in a single query
    let messaging_capabilities_events = wn
        .nostr
        .fetch_events(
            vec![dm_relay_list_filter, prekey_filter, key_package_list_filter],
            Some(DEFAULT_TIMEOUT),
        )
        .await
        .expect("Failed to fetch messaging capabilities");

    // Process messaging capabilities
    for event in messaging_capabilities_events {
        match event.kind {
            Kind::Replaceable(10050) => {
                enriched_contact.nip17 = true;
                enriched_contact.inbox_relays.extend(
                    event
                        .tags
                        .iter()
                        .filter(|tag| tag.kind() == TagKind::Relay)
                        .filter_map(|tag| tag.content())
                        .map(|s| s.to_string()),
                );
            }
            Kind::Replaceable(10051) => {
                enriched_contact.key_package_relays.extend(
                    event
                        .tags
                        .iter()
                        .filter(|tag| tag.kind() == TagKind::Relay)
                        .filter_map(|tag| tag.content())
                        .map(|s| s.to_string()),
                );
            }
            Kind::KeyPackage => {
                if event.tags.find(TagKind::MlsProtocolVersion).is_some() {
                    enriched_contact.nip104 = true;
                }
            }
            _ => {}
        }
    }

    Ok(enriched_contact)
}

#[tauri::command]
pub async fn get_contacts(
    wn: State<'_, Whitenoise>,
) -> Result<HashMap<String, EnrichedContact>, String> {
    // Fetch contact list public keys
    let contact_list_pubkeys = wn
        .nostr
        .get_contact_list_public_keys(Some(DEFAULT_TIMEOUT))
        .await
        .expect("Failed to fetch contact list public keys");

    debug!(
        "contact_list_pubkeys length: {:?}",
        contact_list_pubkeys.len()
    );

    // Fetch metadata for all contacts in a single query
    let metadata_filter = Filter::new()
        .kind(Kind::Metadata)
        .authors(contact_list_pubkeys.clone());
    let contacts = wn
        .nostr
        .database()
        .query(vec![metadata_filter])
        .await
        .expect("Failed to query metadata");

    // Prepare filters for messaging capabilities
    let dm_relay_list_filter = Filter::new()
        .kind(Kind::Custom(10050))
        .authors(contact_list_pubkeys.clone());
    let prekey_filter = Filter::new()
        .kind(Kind::KeyPackage)
        .authors(contact_list_pubkeys.clone());
    let key_package_list_filter = Filter::new()
        .kind(Kind::Custom(10051))
        .authors(contact_list_pubkeys.clone());

    // Fetch messaging capabilities for all contacts in a single query
    let messaging_capabilities_events = wn
        .nostr
        .fetch_events(
            vec![dm_relay_list_filter, prekey_filter, key_package_list_filter],
            Some(DEFAULT_TIMEOUT),
        )
        .await
        .expect("Failed to fetch messaging capabilities");

    // Process contacts and messaging capabilities
    let mut contacts_map: HashMap<String, EnrichedContact> = HashMap::new();
    for contact in contacts {
        let metadata = Metadata::from_json(contact.content).expect("Failed to parse metadata");
        let author = contact.pubkey.to_string();

        let mut enriched_contact = EnrichedContact {
            metadata,
            nip17: false,
            nip104: false,
            inbox_relays: Vec::new(),
            key_package_relays: Vec::new(),
        };

        // Process messaging capabilities
        for event in messaging_capabilities_events.clone() {
            if event.pubkey.to_string() == author {
                match event.kind {
                    Kind::Replaceable(10050) => {
                        enriched_contact.nip17 = true;
                        enriched_contact.inbox_relays.extend(
                            event
                                .tags
                                .iter()
                                .filter(|tag| tag.kind() == TagKind::Relay)
                                .filter_map(|tag| tag.content())
                                .map(|s| s.to_string()),
                        );
                    }
                    Kind::Replaceable(10051) => {
                        enriched_contact.key_package_relays.extend(
                            event
                                .tags
                                .iter()
                                .filter(|tag| tag.kind() == TagKind::Relay)
                                .filter_map(|tag| tag.content())
                                .map(|s| s.to_string()),
                        );
                    }
                    Kind::KeyPackage => {
                        if event.tags.find(TagKind::MlsProtocolVersion).is_some() {
                            enriched_contact.nip104 = true;
                        }
                    }
                    _ => {}
                }
            }
        }

        contacts_map.insert(author, enriched_contact);
    }

    Ok(contacts_map)
}

#[tauri::command]
pub async fn get_metadata_for_pubkey(
    pubkey: String,
    wn: State<'_, Whitenoise>,
) -> Result<Metadata, String> {
    let public_key = PublicKey::from_hex(pubkey).unwrap();
    Ok(wn
        .nostr
        .fetch_metadata(public_key, Some(DEFAULT_TIMEOUT))
        .await
        .unwrap_or(Metadata::default()))
}

#[tauri::command]
pub async fn get_legacy_chats(
    pubkey: String,
    wn: State<'_, Whitenoise>,
) -> Result<HashMap<String, Conversation>, String> {
    let start = Instant::now();
    let current_pubkey = PublicKey::from_hex(&pubkey).expect("Invalid pubkey");
    let events = wn
        .nostr
        .fetch_events(
            vec![
                Filter::new()
                    .kind(Kind::EncryptedDirectMessage)
                    .author(current_pubkey),
                Filter::new()
                    .kind(Kind::EncryptedDirectMessage)
                    .pubkeys(vec![current_pubkey]),
            ],
            Some(DEFAULT_TIMEOUT),
        )
        .await
        .unwrap();

    debug!(target: "whitenoise::nostr::get_legacy_chats", "Found {} events in {:?}", events.len(), start.elapsed());

    let decrypt_start = Instant::now();
    let mut chats: HashMap<String, Conversation> = HashMap::new();
    let signer = wn.nostr.signer().await.unwrap();
    let signer_pubkey = signer.public_key().await.unwrap();

    // filter some gross test events
    let filtered_events = events
        .iter()
        .filter(|event| {
            ![
                "97e876268705d844061cd4f05a2dd48eda126fe8f6d6ab8b373eb2d54f04389d",
                "ae5c1c3575897df77a904a3e506456f8cd683db0ba04f22863e7c174ab79deac",
            ]
            .contains(&event.pubkey.to_string().as_str())
        })
        .cloned()
        .collect::<Vec<Event>>();

    for event in filtered_events {
        let (other_party_pubkey, decrypt_pubkey) = if event.pubkey == signer_pubkey {
            let other_pubkey = PublicKey::parse(
                event
                    .tags
                    .find(TagKind::SingleLetter(SingleLetterTag::lowercase(
                        Alphabet::P,
                    )))
                    .unwrap()
                    .content()
                    .unwrap(),
            )
            .unwrap();
            (other_pubkey, other_pubkey)
        } else {
            (event.pubkey, event.pubkey)
        };

        if let Ok(decrypted) = signer.nip04_decrypt(&decrypt_pubkey, &event.content).await {
            let raw_event = RawEvent {
                id: event.id.to_string(),
                created_at: event.created_at,
                content: decrypted,
                kind: event.kind,
                tags: event.tags.to_vec(),
                pubkey: event.pubkey,
                sig: event.sig,
            };

            chats
                .entry(other_party_pubkey.to_string())
                .and_modify(|conv| {
                    conv.events.push(raw_event.clone());
                    conv.latest = conv.latest.max(event.created_at);
                })
                .or_insert_with(|| Conversation {
                    latest: event.created_at,
                    events: vec![raw_event],
                    metadata: Metadata::default(),
                });
        }
    }
    debug!(target: "whitenoise::nostr::get_legacy_chats", "Decrypted events in {:?}", decrypt_start.elapsed());

    let metadata_start = Instant::now();
    // Fetch metadata for each chat
    for (pubkey, conv) in chats.iter_mut() {
        let filter = Filter::new()
            .kind(Kind::Metadata)
            .authors(vec![PublicKey::from_hex(pubkey).unwrap()])
            .limit(1);
        let meta_events = wn
            .nostr
            .fetch_events(vec![filter], Some(DEFAULT_TIMEOUT))
            .await
            .unwrap();
        let meta = if let Some(meta_event) = meta_events.first() {
            Metadata::from_json(meta_event.content.clone()).unwrap_or(Metadata::default())
        } else {
            Metadata::default()
        };
        conv.metadata = meta;
    }
    debug!(target: "whitenoise::nostr::get_legacy_chats", "Fetched metadata in {:?}", metadata_start.elapsed());

    // Sort events within each conversation
    for conv in chats.values_mut() {
        conv.events.sort_by_key(|e| e.created_at);
    }

    // Sort conversations by latest timestamp
    let mut sorted_chats: Vec<_> = chats.into_iter().collect();
    sorted_chats.sort_by(|a, b| b.1.latest.cmp(&a.1.latest));
    debug!(target: "whitenoise::nostr::get_legacy_chats", "Sorted {} chats in total time: {:?}", sorted_chats.len(), start.elapsed());
    Ok(sorted_chats.into_iter().collect())
}

/// Updates the Nostr identity with new keys and sets up subscriptions.
///
/// This function performs the following tasks:
/// 1. Unsubscribes from all existing subscriptions
/// 2. Updates the signer for the Nostr client with the new keys
/// 3. Clears existing relays and adds default relays
/// 4. Fetches and applies DM relay lists for the user
/// 5. Sets up subscriptions for contacts, metadata, messaging, and gift-wrapped messages
///
/// # Arguments
///
/// * `keys` - The new Keys to be used for the Nostr identity
/// * `wn` - A reference to the Whitenoise state
///
/// # Returns
///
/// Returns `Ok(())` if the update is successful, or an error if any step fails
///
/// # Errors
///
/// This function will return an error if:
/// - Unsubscribing from existing subscriptions fails
/// - Setting the new signer fails
/// - Removing or adding relays fails
/// - Fetching events or setting up new subscriptions fails
pub async fn update_nostr_identity(keys: Keys, wn: &State<'_, Whitenoise>) -> Result<()> {
    let mut start = Instant::now();
    debug!(target: "whitenoise::nostr::update_nostr_identity", "Updating nostr identity");

    // Unsubscribe from all existing subscriptions
    wn.nostr.unsubscribe_all().await;
    debug!(target: "whitenoise::nostr::update_nostr_identity", "Unsubscribed from all in {:?}", start.elapsed());
    start = Instant::now();

    // Update the signer for the Nostr client
    wn.nostr
        .set_signer(Some(NostrSigner::Keys(keys.clone())))
        .await;
    debug!(target: "whitenoise::nostr::update_nostr_identity", "Set signer in {:?}", start.elapsed());
    start = Instant::now();

    // Clear existing relays and add default ones
    for relay in DEFAULT_RELAYS {
        wn.nostr.remove_relay(relay).await.unwrap();
    }

    debug!(target: "whitenoise::nostr::update_nostr_identity", "Removed all relays in {:?}", start.elapsed());
    start = Instant::now();

    for relay in DEFAULT_RELAYS {
        debug!(target: "whitenoise::nostr::update_nostr_identity", "Adding relay: {}", relay);
        wn.nostr.add_relay(relay).await?;
    }
    debug!(target: "whitenoise::nostr::update_nostr_identity", "Added default relays in {:?}", start.elapsed());
    start = Instant::now();

    wn.nostr.connect().await;
    debug!(target: "whitenoise::nostr::update_nostr_identity", "Connected to relays in {:?}", start.elapsed());
    start = Instant::now();

    // Fetch and apply DM relay lists for user
    debug!(target: "whitenoise::nostr::update_nostr_identity", "Fetching DM relay lists");
    let relay_list_events = wn
        .nostr
        .fetch_events(
            vec![Filter::new()
                .kind(Kind::Replaceable(10050))
                .author(keys.public_key())
                .limit(1)],
            Some(DEFAULT_TIMEOUT),
        )
        .await
        .expect("Failed to fetch DM relay lists");
    debug!(target: "whitenoise::nostr::update_nostr_identity", "Fetched DM relay lists in {:?}", start.elapsed());
    start = Instant::now();

    if let Some(event) = relay_list_events.first() {
        let relay_tags = event
            .tags
            .iter()
            .filter(|tag| matches!(tag.kind(), TagKind::Relay));
        for tag in relay_tags {
            if let Some(relay_url) = tag.content() {
                match wn.nostr.add_relay(relay_url).await {
                    Ok(_) => {
                        debug!(target: "whitenoise::nostr::update_nostr_identity", "Added relay: {}", relay_url)
                    }
                    Err(e) => {
                        error!(target: "whitenoise::nostr::update_nostr_identity", "Failed to add relay {}: {}", relay_url, e)
                    }
                }
            } else {
                error!(target: "whitenoise::nostr::update_nostr_identity", "DM Relay List tag has no content");
            }
        }
    } else {
        debug!(target: "whitenoise::nostr::update_nostr_identity", "No DM relay list events found");
    }
    debug!(target: "whitenoise::nostr::update_nostr_identity", "Added DM relay lists in {:?}", start.elapsed());
    start = Instant::now();

    // Set up subscriptions
    setup_subscriptions(&keys, wn).await?;

    debug!(target: "whitenoise::nostr::update_nostr_identity", "Updated nostr identity & subscriptions for user {:?} in {:?}", keys.public_key(), start.elapsed());

    Ok(())
}

/// Sets up various subscriptions for the Nostr client
///
/// # Arguments
///
/// * `keys` - The Keys used for the current Nostr identity
/// * `wn` - A reference to the Whitenoise state
///
/// # Returns
///
/// Returns `Ok(())` if all subscriptions are set up successfully, or an error if any subscription fails
async fn setup_subscriptions(keys: &Keys, wn: &State<'_, Whitenoise>) -> Result<()> {
    debug!(target: "whitenoise::nostr::update_nostr_identity", "Setting up subscriptions");
    // Subscribe for contacts
    let contacts_filter = Filter::new()
        .kind(Kind::ContactList)
        .author(keys.public_key());
    let _contacts_sub = wn.nostr.subscribe(vec![contacts_filter], None).await;
    debug!(target: "whitenoise::nostr::update_nostr_identity", "Subscribed for contacts list updates");

    // Subscribe for contact list metadata
    let contact_list_pubkeys = wn
        .nostr
        .get_contact_list_public_keys(Some(DEFAULT_TIMEOUT))
        .await
        .unwrap();
    debug!(target: "whitenoise::nostr::update_nostr_identity", "Got {:?} contact list pubkeys", contact_list_pubkeys.len());
    let metadata_filter = Filter::new()
        .kind(Kind::Metadata)
        .authors(contact_list_pubkeys);
    let _metadata_sub = wn.nostr.subscribe(vec![metadata_filter], None).await;
    debug!(target: "whitenoise::nostr::update_nostr_identity", "Subscribed for metadata updates");

    // Subscribe for messaging (NIP-04)
    let nip_4_sent = Filter::new()
        .kind(Kind::EncryptedDirectMessage)
        .author(keys.public_key());
    let nip_4_received = Filter::new()
        .kind(Kind::EncryptedDirectMessage)
        .pubkeys(vec![keys.public_key()]);
    let _nip_4_sent_sub = wn.nostr.subscribe(vec![nip_4_sent], None).await;
    let _nip_4_received_sub = wn.nostr.subscribe(vec![nip_4_received], None).await;
    debug!(target: "whitenoise::nostr::update_nostr_identity", "Subscribed for nip4 messaging");

    let gift_wrap_filter = Filter::new()
        .kind(Kind::GiftWrap)
        .pubkeys(vec![keys.public_key()]);
    let _gift_wrap_sub = wn.nostr.subscribe(vec![gift_wrap_filter], None).await;
    debug!(target: "whitenoise::nostr::update_nostr_identity", "Subscribed for gift wrapped messages");

    Ok(())
}

pub async fn subscribe_to_mls_group_messages(wn: &State<'_, Whitenoise>) -> Result<()> {
    if wn.accounts.lock().unwrap().current_identity.is_some() {
        // Subscribe for Gift-wrapped messages addressed to the user's groups)
        let nostr_groups = NostrMlsGroup::get_groups(wn.clone()).expect("Failed to get groups");

        let group_ids = &nostr_groups
            .iter()
            .map(|g| g.nostr_group_id.clone())
            .collect::<Vec<String>>();

        let group_messages_filter = Filter::new()
            .kind(Kind::GiftWrap)
            .custom_tag(SingleLetterTag::lowercase(Alphabet::H), group_ids);

        let _group_messages_sub = wn.nostr.subscribe(vec![group_messages_filter], None).await;
        debug!(target: "whitenoise::nostr::subscribe_to_mls_group_messages", "Subscribed for group messages");
    }
    Ok(())
}

#[tauri::command]
pub async fn send_message(
    pubkey: String,
    message: String,
    wn: State<'_, Whitenoise>,
) -> Result<EventId, String> {
    let pubkey = PublicKey::from_hex(&pubkey).unwrap();
    let signer = wn.nostr.signer().await.unwrap();

    let event = EventBuilder::new(
        Kind::EncryptedDirectMessage,
        signer.nip04_encrypt(&pubkey, message).await.unwrap(),
        [Tag::public_key(pubkey)],
    );

    let event_output = wn.nostr.send_event_builder(event).await.unwrap();
    let event_id = event_output.id();

    Ok(*event_id)
}

#[tauri::command]
pub async fn decrypt_content(
    content: String,
    pubkey: String,
    wn: State<'_, Whitenoise>,
) -> Result<String, String> {
    let author_pubkey = PublicKey::from_hex(&pubkey).unwrap();
    let signer = wn.nostr.signer().await.unwrap();
    let decrypted = signer.nip04_decrypt(&author_pubkey, content).await.unwrap();
    Ok(decrypted)
}
