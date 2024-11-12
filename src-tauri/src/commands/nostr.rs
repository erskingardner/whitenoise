use crate::secrets_store;
use crate::types::{EnrichedContact, NostrEncryptionMethod};
use crate::whitenoise::Whitenoise;
use nostr_sdk::prelude::*;
use std::collections::HashMap;
use tauri::Emitter;

#[tauri::command]
pub async fn init_nostr_for_current_user(
    wn: tauri::State<'_, Whitenoise>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let current_account = wn
        .account_manager
        .get_active_account()
        .map_err(|e| e.to_string())?;
    let keys = secrets_store::get_nostr_keys_for_pubkey(&current_account.pubkey, &wn.data_dir)
        .map_err(|e| e.to_string())?;
    wn.nostr
        .update_nostr_identity(keys)
        .await
        .map_err(|e| e.to_string())?;

    app_handle
        .emit("nostr_ready", ())
        .map_err(|e| e.to_string())?;

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

    let enriched_contact = EnrichedContact {
        metadata,
        nip17: !inbox_relays.is_empty(),
        nip104: !key_package_relays.is_empty(),
        inbox_relays,
        key_package_relays,
    };

    if update_account {
        wn.account_manager
            .update_account(pubkey.to_hex(), enriched_contact.clone())
            .map_err(|e| format!("Failed to update account: {}", e))?;
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
        .get_contact_list_public_keys(Some(wn.nostr.timeout()))
        .await
        .expect("Failed to fetch contact list public keys");

    tracing::debug!(
        "contact_list_pubkeys length: {:?}",
        contact_list_pubkeys.len()
    );

    let mut contacts_map: HashMap<String, EnrichedContact> = HashMap::new();

    if !contact_list_pubkeys.is_empty() {
        // Fetch metadata for all contacts in a single query
        let metadata_filter = Filter::new()
            .kind(Kind::Metadata)
            .authors(contact_list_pubkeys.clone());

        tracing::debug!(
            target: "whitenoise::commands::nostr::fetch_enriched_contacts",
            "contact_metadata_filter: {:?}",
            metadata_filter
        );

        let stored_contacts = wn
            .nostr
            .client
            .database()
            .query(vec![metadata_filter.clone()])
            .await
            .expect("Failed to query metadata");

        let fetched_contacts = wn
            .nostr
            .client
            .fetch_events(vec![metadata_filter.clone()], Some(wn.nostr.timeout()))
            .await
            .expect("Failed to fetch metadata");

        let contacts = stored_contacts.merge(fetched_contacts);

        // Prepare filters for messaging capabilities
        let dm_relay_list_filter = Filter::new()
            .kind(Kind::Custom(10050))
            .authors(contact_list_pubkeys.clone());
        let prekey_filter = Filter::new()
            .kind(Kind::MlsKeyPackage)
            .authors(contact_list_pubkeys.clone());
        let key_package_list_filter = Filter::new()
            .kind(Kind::Custom(10051))
            .authors(contact_list_pubkeys.clone());

        // Fetch messaging capabilities for all contacts in a single query
        let messaging_capabilities_events = wn
            .nostr
            .client
            .fetch_events(
                vec![dm_relay_list_filter, prekey_filter, key_package_list_filter],
                Some(wn.nostr.timeout()),
            )
            .await
            .expect("Failed to fetch messaging capabilities");

        // Process contacts and messaging capabilities
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
                        Kind::MlsKeyPackage => {
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
    };

    Ok(contacts_map)
}

#[tauri::command]
pub async fn fetch_metadata(
    pubkey: String,
    wn: tauri::State<'_, Whitenoise>,
) -> Result<Metadata, String> {
    let pubkey = PublicKey::from_hex(&pubkey).map_err(|_| "Invalid pubkey".to_string())?;
    wn.nostr
        .fetch_user_metadata(pubkey)
        .await
        .map_err(|e| format!("Failed to get metadata: {}", e))
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
        10050 => Kind::Replaceable(10050),
        10051 => Kind::Replaceable(10051),
        _ => return Err("Invalid relay list kind".to_string()),
    };

    let event = EventBuilder::new(event_kind, "", tags)
        .sign(&signer)
        .await
        .map_err(|e| e.to_string())?;

    wn.nostr
        .client
        .send_event(event)
        .await
        .map_err(|e| e.to_string())?;

    let active_account = wn
        .account_manager
        .get_active_account()
        .map_err(|e| e.to_string())?;

    let new_enriched_contact = match kind {
        10050 => EnrichedContact {
            metadata: active_account.metadata,
            nip17: true,
            nip104: true,
            inbox_relays: relays.clone(),
            key_package_relays: active_account.key_package_relays,
        },
        10051 => EnrichedContact {
            metadata: active_account.metadata,
            nip17: true,
            nip104: true,
            inbox_relays: active_account.inbox_relays,
            key_package_relays: relays.clone(),
        },
        _ => return Err("Invalid relay list kind".to_string()),
    };

    wn.account_manager
        .update_account(
            signer
                .get_public_key()
                .await
                .map_err(|e| e.to_string())?
                .to_hex(),
            new_enriched_contact,
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}
