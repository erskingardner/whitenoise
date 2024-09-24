use crate::whitenoise::Whitenoise;
use log::debug;
use nostr_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use std::{collections::HashMap, time::Duration};
use tauri::State;
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

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
}

// --- Commands ---

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
        .query(vec![metadata_filter], Order::Asc)
        .await
        .expect("Failed to query metadata");

    // Prepare filters for messaging capabilities
    let dm_relay_list_filter = Filter::new()
        .kind(Kind::Custom(10050))
        .authors(contact_list_pubkeys.clone());
    let prekey_filter = Filter::new()
        .kind(Kind::Custom(443))
        .authors(contact_list_pubkeys);

    // Fetch messaging capabilities for all contacts in a single query
    let messaging_capabilities_events = wn
        .nostr
        .get_events_of(
            vec![dm_relay_list_filter, prekey_filter],
            EventSource::Both {
                timeout: Some(DEFAULT_TIMEOUT),
                specific_relays: None,
            },
        )
        .await
        .expect("Failed to fetch messaging capabilities");

    // Process contacts and messaging capabilities
    let mut contacts_map: HashMap<String, EnrichedContact> = HashMap::new();
    for contact in contacts {
        let metadata = Metadata::from_json(contact.content()).expect("Failed to parse metadata");
        let author = contact.author().to_string();

        let mut enriched_contact = EnrichedContact {
            metadata,
            nip17: false,
            nip104: false,
            inbox_relays: Vec::new(),
        };

        // Process messaging capabilities
        for event in &messaging_capabilities_events {
            if event.author().to_string() == author {
                match event.kind() {
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
                    Kind::Custom(443) => {
                        if event
                            .get_tag_content(TagKind::Custom("mls_protocol_version".into()))
                            .is_some()
                        {
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
        .metadata(public_key)
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
        .get_events_of(
            vec![
                Filter::new()
                    .kind(Kind::EncryptedDirectMessage)
                    .author(current_pubkey),
                Filter::new()
                    .kind(Kind::EncryptedDirectMessage)
                    .pubkeys(vec![current_pubkey]),
            ],
            EventSource::both(Some(DEFAULT_TIMEOUT)),
        )
        .await
        .unwrap();

    debug!(target: "whitenoise::nostr::get_legacy_chats", "Found {} events in {:?}", events.len(), start.elapsed());

    let decrypt_start = Instant::now();
    let mut chats: HashMap<String, Conversation> = HashMap::new();
    let signer = wn.nostr.signer().await.unwrap();
    let signer_pubkey = signer.public_key().await.unwrap();

    for event in events {
        debug!("event: {:?}", event);
        let (other_party_pubkey, decrypt_pubkey) = if event.author() == signer_pubkey {
            let other_pubkey = PublicKey::parse(
                event
                    .get_tag_content(TagKind::SingleLetter(SingleLetterTag::lowercase(
                        Alphabet::P,
                    )))
                    .unwrap(),
            )
            .unwrap();
            (other_pubkey, other_pubkey)
        } else {
            (event.author(), event.author())
        };

        if let Ok(decrypted) = signer.nip04_decrypt(decrypt_pubkey, event.content()).await {
            let raw_event = RawEvent {
                id: event.id().to_string(),
                created_at: event.created_at(),
                content: decrypted,
                kind: event.kind(),
                tags: event.tags().to_vec(),
                pubkey: event.author(),
                sig: event.signature(),
            };

            chats
                .entry(other_party_pubkey.to_string())
                .and_modify(|conv| {
                    conv.events.push(raw_event.clone());
                    conv.latest = conv.latest.max(event.created_at());
                })
                .or_insert_with(|| Conversation {
                    latest: event.created_at(),
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
            .database()
            .query(vec![filter], Order::Desc)
            .await
            .unwrap();
        let meta = if let Some(meta_event) = meta_events.first() {
            Metadata::from_json(meta_event.content()).unwrap_or(Metadata::default())
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

pub async fn update_nostr_identity(keys: Keys, wn: &State<'_, Whitenoise>) -> Result<(), String> {
    debug!(target: "whitenoise::nostr::update_nostr_identity", "Updating nostr identity");
    // Unsubscribe from all existing subscriptions
    wn.nostr.unsubscribe_all().await;
    debug!(target: "whitenoise::nostr::update_nostr_identity", "Unsubscribed from all");
    // Update the signer for the Nostr client
    wn.nostr
        .set_signer(Some(NostrSigner::Keys(keys.clone())))
        .await;
    debug!(target: "whitenoise::nostr::update_nostr_identity", "Set signer");
    // Event notification listener
    // let mut notifications = wn.nostr.notifications();

    // TODO: Update relays for new user
    // Clear existing relays
    // wn.nostr.remove_all_relays().await;
    // Fetch relay lists for user and apply them.

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
    debug!(target: "whitenoise::nostr::update_nostr_identity", "Got contact list pubkeys: {:?}", contact_list_pubkeys);
    let metadata_filter = Filter::new()
        .kind(Kind::Metadata)
        .authors(contact_list_pubkeys);
    let _metadata_sub = wn.nostr.subscribe(vec![metadata_filter], None).await;
    debug!(target: "whitenoise::nostr::update_nostr_identity", "Subscribed for metadata updates");
    // Subscribe for messaging (NIP-04)
    let nip_4_sent = Filter::new()
        .kind(Kind::EncryptedDirectMessage)
        .author(keys.public_key());
    debug!(target: "whitenoise::nostr::update_nostr_identity", "Created nip4 sent filter");
    let nip_4_received = Filter::new()
        .kind(Kind::EncryptedDirectMessage)
        .pubkeys(vec![keys.public_key()]);
    debug!(target: "whitenoise::nostr::update_nostr_identity", "Created nip4 received filter");
    let _nip_4_sent_sub = wn.nostr.subscribe(vec![nip_4_sent], None).await;
    let _nip_4_received_sub = wn.nostr.subscribe(vec![nip_4_received], None).await;
    debug!(target: "whitenoise::nostr::update_nostr_identity", "Subscribed for nip4 messaging");
    debug!(target: "whitenoise::nostr::update_nostr_identity", "Updated nostr identity & subscriptions for user {:?}", keys.public_key());

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
        signer.nip04_encrypt(pubkey, message).await.unwrap(),
        [Tag::public_key(pubkey)],
    );

    let event_output = wn.nostr.send_event_builder(event).await.unwrap();
    let event_id = event_output.id();

    Ok(*event_id)
}

#[tauri::command]
pub async fn fetch_dev_events(wn: State<'_, Whitenoise>) -> Result<HashMap<String, usize>, String> {
    let keys = wn.nostr.signer().await.unwrap();

    let relay_contacts = wn
        .nostr
        .get_contact_list_public_keys(Some(DEFAULT_TIMEOUT))
        .await
        .unwrap();

    let database_contact_list_events = wn
        .nostr
        .database()
        .query(
            vec![Filter::new()
                .kind(Kind::ContactList)
                .author(keys.public_key().await.unwrap())
                .limit(1)],
            Order::Desc,
        )
        .await
        .unwrap();
    let database_contact_list_event = database_contact_list_events.first().unwrap();
    let database_contacts = database_contact_list_event.get_tags_content(TagKind::SingleLetter(
        SingleLetterTag::lowercase(Alphabet::P),
    ));

    let relay_chats = wn
        .nostr
        .get_events_of(
            vec![
                Filter::new()
                    .kind(Kind::EncryptedDirectMessage)
                    .author(keys.public_key().await.unwrap()),
                Filter::new()
                    .kind(Kind::EncryptedDirectMessage)
                    .pubkeys(vec![keys.public_key().await.unwrap()]),
            ],
            EventSource::Relays {
                timeout: Some(DEFAULT_TIMEOUT),
                specific_relays: None,
            },
        )
        .await
        .unwrap();
    let mut database_chats: Vec<Event> = Vec::new();
    let sent_database_chats = wn
        .nostr
        .database()
        .query(
            vec![Filter::new()
                .kind(Kind::EncryptedDirectMessage)
                .author(keys.public_key().await.unwrap())],
            Order::Desc,
        )
        .await
        .unwrap();
    let received_database_chats = wn
        .nostr
        .database()
        .query(
            vec![Filter::new()
                .kind(Kind::EncryptedDirectMessage)
                .pubkeys(vec![keys.public_key().await.unwrap()])],
            Order::Desc,
        )
        .await
        .unwrap();
    database_chats.extend(sent_database_chats);
    database_chats.extend(received_database_chats);

    let mut events_map: HashMap<String, usize> = HashMap::new();
    events_map.insert("database_contacts".to_string(), database_contacts.len());
    events_map.insert("relay_contacts".to_string(), relay_contacts.len());
    events_map.insert("relay_chats".to_string(), relay_chats.len());
    events_map.insert("database_chats".to_string(), database_chats.len());

    Ok(events_map) // Return the events_map wrapped in Ok
}
