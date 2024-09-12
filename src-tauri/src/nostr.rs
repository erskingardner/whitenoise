use crate::secrets_store;
use anyhow::{Context, Result};
use lazy_static::lazy_static;
use nostr_ndb::NdbDatabase;
use nostr_sdk::prelude::*;
use parking_lot::{Mutex, Once};
use std::time::Duration;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(3);

lazy_static! {
    static ref CLIENT: Mutex<Option<Client>> = Mutex::new(None);
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Conversation {
    pub latest: Timestamp,
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

static INIT: Once = Once::new();

pub async fn init_nostr_for_pubkey(pubkey: String) -> Result<()> {
    INIT.call_once(|| {
        tokio::spawn(async move {
            let keys = secrets_store::get_nostr_keys_for_pubkey(pubkey.as_str())
                .expect("Failed to get Nostr keys");

            let database = NdbDatabase::open("./db/ndb").expect("Failed to open database");

            let nostr_client = Client::builder().signer(keys).database(database).build();

            let relays = vec![
                "wss://relay.damus.io",
                "wss://relay.snort.social",
                "wss://relay.primal.net",
                "wss://purplepag.es",
            ];

            for relay in relays {
                let _ = nostr_client.add_relay(relay).await;
            }

            nostr_client.connect().await;
            *CLIENT.lock() = Some(nostr_client.clone());
        });
    });

    Ok(())
}

pub fn get_client() -> Result<Client> {
    let nostr_client = CLIENT
        .lock()
        .as_ref()
        .cloned()
        .context("Client not initialized")?;
    Ok(nostr_client)
}

pub async fn update_signer_with_keys(keys: Keys) -> Result<()> {
    let client = {
        let client_guard = CLIENT.lock();
        client_guard.as_ref().cloned()
    };

    if let Some(client) = client {
        let signer = NostrSigner::from(keys.clone());
        client.set_signer(Some(signer)).await;
        *CLIENT.lock() = Some(client);
    }

    Ok(())
}

pub async fn _update_relays(_relays: Vec<String>) -> Result<()> {
    todo!()
}

// --- Commands ---
#[tauri::command]
pub async fn get_contacts() -> Vec<Contact> {
    get_client()
        .expect("Couldn't get the nostr client")
        .get_contact_list(Some(DEFAULT_TIMEOUT))
        .await
        .unwrap()
}

#[tauri::command]
pub async fn get_legacy_chats(pubkey: String) -> HashMap<String, Conversation> {
    let client = get_client().expect("Couldn't get the nostr client");
    let current_pubkey = PublicKey::from_hex(&pubkey).unwrap();
    let filter = Filter::new()
        .kind(Kind::EncryptedDirectMessage)
        .authors(vec![current_pubkey]);

    let filter2 = Filter::new()
        .kind(Kind::EncryptedDirectMessage)
        .pubkeys(vec![current_pubkey]);
    
    let events = client.get_events_of(vec![filter, filter2], EventSource::both(Some(DEFAULT_TIMEOUT))).await.unwrap();
    
    let mut chats: HashMap<String, Conversation> = HashMap::new();
    let signer = client.signer().await.unwrap();
    let signer_pubkey = signer.public_key().await.unwrap();

    for event in events {
        let (other_party_pubkey, decrypt_pubkey) = if event.author() == signer_pubkey {
            let other_pubkey = PublicKey::parse(event.get_tag_content(TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::P)))
                .unwrap()).unwrap();
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

            chats.entry(other_party_pubkey.to_string())
                .and_modify(|conv| {
                    conv.events.push(raw_event.clone());
                    conv.latest = conv.latest.max(event.created_at());
                })
                .or_insert_with(|| Conversation {
                    latest: event.created_at(),
                    events: vec![raw_event],
                });
        }
    }

    // Sort events within each conversation
    for conv in chats.values_mut() {
        conv.events.sort_by_key(|e| e.created_at);
    }

    // Sort conversations by latest timestamp
    let mut sorted_chats: Vec<_> = chats.into_iter().collect();
    sorted_chats.sort_by(|a, b| b.1.latest.cmp(&a.1.latest));
    
    sorted_chats.into_iter().collect()
}
