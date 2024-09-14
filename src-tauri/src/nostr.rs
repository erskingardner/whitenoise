use crate::secrets_store;
use nostr_sdk::prelude::*;
use std::time::Duration;
use tokio::sync::OnceCell;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

static CLIENT: OnceCell<Client> = OnceCell::const_new();

/// Get or init client
///
/// If the client not exists, initialize it without signer.
pub async fn get_client() -> &'static Client {
    CLIENT
        .get_or_init(|| async {
            let database = NdbDatabase::open("./db/ndb").expect("Failed to open database");

            let nostr_client = Client::builder().database(database).build();

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

            nostr_client
        })
        .await
}

/// Get keys from secrets store, get (or init) nostr client and set signer.
pub async fn init_nostr_for_pubkey(pubkey: &str) {
    let keys = secrets_store::get_nostr_keys_for_pubkey(pubkey).expect("Failed to get Nostr keys");
    update_signer_with_keys(keys).await
}

/// Update nostr client signer.
pub async fn update_signer_with_keys(keys: Keys) {
    let signer = NostrSigner::from(keys);
    let client = get_client().await;
    client.set_signer(Some(signer)).await;
}

// pub async fn _update_relays(_relays: Vec<String>) -> Result<()> {
//     todo!()
// }

// --- Commands ---
#[tauri::command]
pub async fn get_contacts() -> Vec<Contact> {
    get_client()
        .await
        .get_contact_list(Some(DEFAULT_TIMEOUT))
        .await
        .unwrap()
}
