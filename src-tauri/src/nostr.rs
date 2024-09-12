use crate::secrets_store;
use anyhow::{Context, Result};
use lazy_static::lazy_static;
use nostr_ndb::NdbDatabase;
use nostr_sdk::prelude::*;
use parking_lot::{Mutex, Once};
use std::time::Duration;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

lazy_static! {
    static ref CLIENT: Mutex<Option<Client>> = Mutex::new(None);
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
