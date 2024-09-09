use crate::secrets_store;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use nostr_sdk::prelude::*;

// --- Commands ---

#[tauri::command]
pub async fn init_nostr_with_pubkey(pubkey: String) {
    // Configure client to use proxy for `.onion` relays
    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050));
    let connection: Connection = Connection::new()
        .proxy(addr) // Use `.embedded_tor()` instead to enable the embedded tor client (require `tor` feature)
        .target(ConnectionTarget::Onion);
    let opts = Options::new().connection(connection);

    let keys = secrets_store::get_nostr_keys_for_pubkey(pubkey.as_str())
        .expect("Couldn't parse nostr keys");

    let client = Client::with_opts(&keys, opts);

    let relays = vec![
        "wss://relay.damus.io",
        "wss://relay.snort.social",
        "wss://relay.primal.net",
        "wss://purplepag.es",
    ];
    client.add_relays(relays).await.expect("Couldn't add relay");

    client.connect().await;
}
