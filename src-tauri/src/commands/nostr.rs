use crate::accounts::Account;
use crate::relays::RelayType;
use crate::secrets_store;
use crate::types::{EnrichedContact, NostrEncryptionMethod};
use crate::whitenoise::Whitenoise;
use nostr_openmls::NostrMls;
use nostr_sdk::prelude::*;
use std::collections::HashMap;
use tauri::Emitter;

#[tauri::command]
pub async fn init_nostr_for_current_user(
    wn: tauri::State<'_, Whitenoise>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let current_account = Account::get_active(wn.clone())
        .await
        .map_err(|e| e.to_string())?;

    // Update Nostr identity and connect relays
    wn.nostr
        .set_nostr_identity(&current_account, wn.clone(), &app_handle)
        .await
        .map_err(|e| e.to_string())?;

    // Then update Nostr MLS instance
    {
        let mut nostr_mls = wn.nostr_mls.lock().await;
        *nostr_mls = NostrMls::new(wn.data_dir.clone(), Some(current_account.pubkey.to_hex()));
    }

    tracing::debug!(
        target: "whitenoise::commands::nostr::init_nostr_for_current_user",
        "Nostr initialized for current user"
    );
    Ok(())
}

#[tauri::command]
pub async fn fetch_enriched_contact(
    pubkey: String,
    update_account: bool,
    wn: tauri::State<'_, Whitenoise>,
    app_handle: tauri::AppHandle,
) -> Result<EnrichedContact, String> {
    let pubkey = PublicKey::from_hex(&pubkey).map_err(|_| "Invalid pubkey".to_string())?;

    let metadata = wn
        .nostr
        .fetch_user_metadata(pubkey)
        .await
        .map_err(|_| "Failed to get metadata".to_string())?;
    let nostr_relays = wn
        .nostr
        .fetch_user_relays(pubkey)
        .await
        .map_err(|_| "Failed to get user relays".to_string())?;
    let inbox_relays = wn
        .nostr
        .fetch_user_inbox_relays(pubkey)
        .await
        .map_err(|_| "Failed to get inbox relays".to_string())?;
    let key_package_relays = wn
        .nostr
        .fetch_user_key_package_relays(pubkey)
        .await
        .map_err(|_| "Failed to get key package relays".to_string())?;
    let key_packages = wn
        .nostr
        .fetch_user_key_packages(pubkey)
        .await
        .map_err(|_| "Failed to get key packages".to_string())?;

    let enriched_contact = EnrichedContact {
        metadata: metadata.unwrap_or_default(),
        nip17: !inbox_relays.is_empty(),
        nip104: !key_packages.is_empty(),
        nostr_relays,
        inbox_relays,
        key_package_relays,
    };

    if update_account {
        let mut account = Account::find_by_pubkey(&pubkey, wn.clone())
            .await
            .map_err(|e| format!("Failed to find account: {}", e))?;

        account.metadata = enriched_contact.metadata.clone();
        account
            .update_relays(RelayType::Nostr, &enriched_contact.nostr_relays, wn.clone())
            .await
            .map_err(|e| format!("Failed to update relays: {}", e))?;
        account
            .update_relays(RelayType::Inbox, &enriched_contact.inbox_relays, wn.clone())
            .await
            .map_err(|e| format!("Failed to update relays: {}", e))?;
        account
            .update_relays(
                RelayType::KeyPackage,
                &enriched_contact.key_package_relays,
                wn.clone(),
            )
            .await
            .map_err(|e| format!("Failed to update relays: {}", e))?;
        account
            .save(wn.clone())
            .await
            .map_err(|e| format!("Failed to save account: {}", e))?;

        app_handle
            .emit("account_changed", ())
            .map_err(|e| e.to_string())?;
    }

    Ok(enriched_contact)
}

#[tauri::command]
pub async fn query_enriched_contact(
    pubkey: String,
    update_account: bool,
    wn: tauri::State<'_, Whitenoise>,
    app_handle: tauri::AppHandle,
) -> Result<EnrichedContact, String> {
    let pubkey = PublicKey::from_hex(&pubkey).map_err(|_| "Invalid pubkey".to_string())?;

    let metadata = wn
        .nostr
        .query_user_metadata(pubkey)
        .await
        .map_err(|_| "Failed to get metadata".to_string())?;
    let nostr_relays = wn
        .nostr
        .query_user_relays(pubkey)
        .await
        .map_err(|_| "Failed to get user relays".to_string())?;
    let inbox_relays = wn
        .nostr
        .query_user_inbox_relays(pubkey)
        .await
        .map_err(|_| "Failed to get inbox relays".to_string())?;
    let key_package_relays = wn
        .nostr
        .query_user_key_package_relays(pubkey)
        .await
        .map_err(|_| "Failed to get key package relays".to_string())?;
    let key_packages = wn
        .nostr
        .query_user_key_packages(pubkey)
        .await
        .map_err(|_| "Failed to get key packages".to_string())?;

    let enriched_contact = EnrichedContact {
        metadata: metadata.unwrap_or_default(),
        nip17: !inbox_relays.is_empty(),
        nip104: !key_packages.is_empty(),
        nostr_relays,
        inbox_relays,
        key_package_relays,
    };

    if update_account {
        let mut account = Account::find_by_pubkey(&pubkey, wn.clone())
            .await
            .map_err(|e| format!("Failed to find account: {}", e))?;

        account.metadata = enriched_contact.metadata.clone();
        account
            .update_relays(RelayType::Nostr, &enriched_contact.nostr_relays, wn.clone())
            .await
            .map_err(|e| format!("Failed to update relays: {}", e))?;
        account
            .update_relays(RelayType::Inbox, &enriched_contact.inbox_relays, wn.clone())
            .await
            .map_err(|e| format!("Failed to update relays: {}", e))?;
        account
            .update_relays(
                RelayType::KeyPackage,
                &enriched_contact.key_package_relays,
                wn.clone(),
            )
            .await
            .map_err(|e| format!("Failed to update relays: {}", e))?;

        account
            .save(wn.clone())
            .await
            .map_err(|e| format!("Failed to save account: {}", e))?;
        app_handle
            .emit("account_changed", ())
            .map_err(|e| e.to_string())?;
    }

    Ok(enriched_contact)
}

#[tauri::command]
pub async fn fetch_enriched_contacts(
    wn: tauri::State<'_, Whitenoise>,
) -> Result<HashMap<String, EnrichedContact>, String> {
    // Fetch contact list public keys
    let contact_list_pubkeys = wn
        .nostr
        .client
        .get_contact_list_public_keys(wn.nostr.timeout().await.unwrap())
        .await
        .expect("Failed to fetch contact list public keys");

    tracing::debug!(
        "fetch_enriched_contacts contact_list_pubkeys length: {:?}",
        contact_list_pubkeys.len()
    );

    let mut contacts_map: HashMap<String, EnrichedContact> = HashMap::new();

    // Bail early if there are no contact list pubkeys
    if contact_list_pubkeys.is_empty() {
        tracing::debug!(
            target: "whitenoise::commands::nostr::fetch_enriched_contacts",
            "No contact list pubkeys found"
        );
        return Ok(contacts_map);
    }
    // Initialize the map with default EnrichedContact for all pubkeys
    for pubkey in &contact_list_pubkeys {
        contacts_map.insert(
            pubkey.to_string(),
            EnrichedContact {
                metadata: Metadata::default(),
                nip17: false,
                nip104: false,
                nostr_relays: Vec::new(),
                inbox_relays: Vec::new(),
                key_package_relays: Vec::new(),
            },
        );
    }

    let filter = Filter::new()
        .kinds(vec![
            Kind::Metadata,
            Kind::RelayList,
            Kind::InboxRelays,
            Kind::MlsKeyPackage,
            Kind::MlsKeyPackageRelays,
        ])
        .authors(contact_list_pubkeys.clone());

    // Fetch all events in parallel using a single request
    let (stored_events, fetched_events) = tokio::join!(
        wn.nostr.client.database().query(vec![filter.clone()]),
        wn.nostr
            .client
            .fetch_events(vec![filter.clone()], wn.nostr.timeout().await.unwrap())
    );

    let all_events = stored_events
        .expect("Failed to query stored events")
        .merge(fetched_events.expect("Failed to fetch events"));

    // Process all events
    for event in all_events {
        let author = event.pubkey.to_string();
        if let Some(contact) = contacts_map.get_mut(&author) {
            match event.kind {
                Kind::Metadata => {
                    if let Ok(metadata) = Metadata::from_json(&event.content) {
                        contact.metadata = metadata;
                    }
                }
                Kind::RelayList => {
                    contact.nostr_relays.extend(
                        event
                            .tags
                            .iter()
                            .filter(|tag| tag.kind() == TagKind::Relay)
                            .filter_map(|tag| tag.content())
                            .map(|s| s.to_string()),
                    );
                }
                Kind::InboxRelays => {
                    contact.nip17 = true;
                    contact.inbox_relays.extend(
                        event
                            .tags
                            .iter()
                            .filter(|tag| tag.kind() == TagKind::Relay)
                            .filter_map(|tag| tag.content())
                            .map(|s| s.to_string()),
                    );
                }
                Kind::MlsKeyPackageRelays => {
                    contact.key_package_relays.extend(
                        event
                            .tags
                            .iter()
                            .filter(|tag| tag.kind() == TagKind::Relay)
                            .filter_map(|tag| tag.content())
                            .map(|s| s.to_string()),
                    );
                }
                Kind::MlsKeyPackage => {
                    if event.tags.find(TagKind::MlsProtocolVersion).is_some() {
                        contact.nip104 = true;
                    }
                }
                _ => {}
            }
        }
    }

    Ok(contacts_map)
}

#[tauri::command]
pub async fn query_enriched_contacts(
    wn: tauri::State<'_, Whitenoise>,
) -> Result<HashMap<String, EnrichedContact>, String> {
    // Query contact list public keys from local database
    let contact_list_pubkeys = wn
        .nostr
        .query_contact_list_pubkeys()
        .await
        .map_err(|e| e.to_string())?;

    tracing::debug!(
        "query_enriched_contacts contact_list_pubkeys length: {:?}",
        contact_list_pubkeys.len()
    );

    let mut contacts_map: HashMap<String, EnrichedContact> = HashMap::new();

    // Bail early if there are no contact list pubkeys
    if contact_list_pubkeys.is_empty() {
        return Ok(contacts_map);
    }

    // Initialize the map with default EnrichedContact for all pubkeys
    for pubkey in &contact_list_pubkeys {
        contacts_map.insert(
            pubkey.to_string(),
            EnrichedContact {
                metadata: Metadata::default(),
                nip17: false,
                nip104: false,
                nostr_relays: Vec::new(),
                inbox_relays: Vec::new(),
                key_package_relays: Vec::new(),
            },
        );
    }

    let filter = Filter::new()
        .kinds(vec![
            Kind::Metadata,
            Kind::RelayList,
            Kind::InboxRelays,
            Kind::MlsKeyPackage,
            Kind::MlsKeyPackageRelays,
        ])
        .authors(contact_list_pubkeys.clone());

    let stored_events = wn
        .nostr
        .client
        .database()
        .query(vec![filter.clone()])
        .await
        .map_err(|e| e.to_string())?;

    // Process all events
    for event in stored_events {
        let author = event.pubkey.to_string();
        if let Some(contact) = contacts_map.get_mut(&author) {
            match event.kind {
                Kind::Metadata => {
                    if let Ok(metadata) = Metadata::from_json(&event.content) {
                        contact.metadata = metadata;
                    }
                }
                Kind::RelayList => {
                    contact.nostr_relays.extend(
                        event
                            .tags
                            .iter()
                            .filter(|tag| tag.kind() == TagKind::Relay)
                            .filter_map(|tag| tag.content())
                            .map(|s| s.to_string()),
                    );
                }
                Kind::InboxRelays => {
                    contact.nip17 = true;
                    contact.inbox_relays.extend(
                        event
                            .tags
                            .iter()
                            .filter(|tag| tag.kind() == TagKind::Relay)
                            .filter_map(|tag| tag.content())
                            .map(|s| s.to_string()),
                    );
                }
                Kind::MlsKeyPackageRelays => {
                    contact.key_package_relays.extend(
                        event
                            .tags
                            .iter()
                            .filter(|tag| tag.kind() == TagKind::Relay)
                            .filter_map(|tag| tag.content())
                            .map(|s| s.to_string()),
                    );
                }
                Kind::MlsKeyPackage => {
                    if event.tags.find(TagKind::MlsProtocolVersion).is_some() {
                        contact.nip104 = true;
                    }
                }
                _ => {}
            }
        }
    }

    Ok(contacts_map)
}

#[tauri::command]
pub async fn fetch_relays(
    wn: tauri::State<'_, Whitenoise>,
) -> Result<HashMap<String, String>, String> {
    Ok(wn
        .nostr
        .client
        .relays()
        .await
        .into_iter()
        .map(|(url, relay)| (url.to_string(), relay.status().to_string()))
        .collect::<HashMap<String, String>>())
}

#[tauri::command]
pub async fn fetch_contacts_with_metadata(
    wn: tauri::State<'_, Whitenoise>,
) -> Result<HashMap<String, Metadata>, String> {
    let events = wn.nostr.fetch_contacts().await.map_err(|e| e.to_string())?;
    let mut metadata_map = HashMap::new();

    for event in events {
        if let Ok(metadata) = serde_json::from_str::<Metadata>(&event.content) {
            metadata_map.insert(event.pubkey.to_hex(), metadata);
        }
    }

    Ok(metadata_map)
}

#[tauri::command]
pub async fn query_contacts_with_metadata(
    wn: tauri::State<'_, Whitenoise>,
) -> Result<HashMap<String, Metadata>, String> {
    let events = wn.nostr.query_contacts().await.map_err(|e| e.to_string())?;

    let mut metadata_map = HashMap::new();

    for event in events {
        if let Ok(metadata) = serde_json::from_str::<Metadata>(&event.content) {
            metadata_map.insert(event.pubkey.to_hex(), metadata);
        }
    }

    Ok(metadata_map)
}

#[tauri::command]
pub async fn encrypt_content(
    content: String,
    pubkey: String,
    method: NostrEncryptionMethod,
    wn: tauri::State<'_, Whitenoise>,
) -> Result<String, String> {
    wn.nostr
        .encrypt_content(content, pubkey, method)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn decrypt_content(
    content: String,
    pubkey: String,
    method: NostrEncryptionMethod,
    wn: tauri::State<'_, Whitenoise>,
) -> Result<String, String> {
    wn.nostr
        .decrypt_content(content, pubkey, method)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn publish_relay_list(
    relays: Vec<String>,
    kind: u64,
    wn: tauri::State<'_, Whitenoise>,
) -> Result<(), String> {
    let signer = wn.nostr.client.signer().await.map_err(|e| e.to_string())?;

    let mut tags: Vec<Tag> = Vec::new();
    for relay in relays.clone() {
        tags.push(Tag::custom(TagKind::Relay, [relay]));
    }

    let event_kind = match kind {
        10050 => Kind::InboxRelays,
        10051 => Kind::MlsKeyPackageRelays,
        _ => return Err("Invalid relay list kind".to_string()),
    };

    let event = EventBuilder::new(event_kind, "")
        .tags(tags)
        .sign(&signer)
        .await
        .map_err(|e| e.to_string())?;

    wn.nostr
        .client
        .send_event(event)
        .await
        .map_err(|e| e.to_string())?;

    let active_account = Account::get_active(wn.clone())
        .await
        .map_err(|e| e.to_string())?;

    match kind {
        10050 => {
            active_account
                .update_relays(RelayType::Inbox, &relays, wn.clone())
                .await
                .map_err(|e| format!("Failed to update relays: {}", e))?;
        }
        10051 => {
            active_account
                .update_relays(RelayType::KeyPackage, &relays, wn.clone())
                .await
                .map_err(|e| format!("Failed to update relays: {}", e))?;
        }
        _ => return Err("Invalid relay list kind".to_string()),
    }
    Ok(())
}

#[tauri::command]
pub async fn search_for_enriched_contacts(
    query: String,
    wn: tauri::State<'_, Whitenoise>,
) -> Result<HashMap<String, EnrichedContact>, String> {
    let enriched_users = wn
        .nostr
        .search_users(query, wn.clone())
        .await
        .map_err(|e| e.to_string())?;

    Ok(enriched_users)
}

#[tauri::command]
pub async fn invite_to_white_noise(
    pubkey: String,
    wn: tauri::State<'_, Whitenoise>,
) -> Result<(), String> {
    let public_key = PublicKey::from_hex(&pubkey).map_err(|e| e.to_string())?;
    let content = "Hi, I'm using White Noise to chat securely on Nostr. Join me! https://github.com/erskingardner/whitenoise/releases".to_string();
    let encrypted_content = wn
        .nostr
        .encrypt_content(content, pubkey, NostrEncryptionMethod::Nip04)
        .await
        .map_err(|e| e.to_string())?;

    let event = EventBuilder::new(Kind::EncryptedDirectMessage, encrypted_content)
        .tag(Tag::public_key(public_key));

    tracing::debug!(
        target: "whitenoise::commands::nostr::invite_to_white_noise",
        "Sending event: {:?}",
        event
    );
    wn.nostr
        .client
        .send_event_builder(event)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn export_nsec(
    pubkey: String,
    wn: tauri::State<'_, Whitenoise>,
) -> Result<String, String> {
    let keys = secrets_store::get_nostr_keys_for_pubkey(&pubkey, &wn.data_dir)
        .map_err(|e| e.to_string())?;

    keys.secret_key().to_bech32().map_err(|e| e.to_string())
}
