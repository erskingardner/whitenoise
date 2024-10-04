use crate::nostr::DEFAULT_TIMEOUT;
use crate::whitenoise::Whitenoise;
use anyhow::Result;
use log::debug;
use nostr_sdk::prelude::*;
use openmls::prelude::*;
use tauri::State;
use tls_codec::Deserialize as TlsDeserialize;

#[allow(dead_code)]
pub struct WelcomeMessage {
    pub event: UnsignedEvent,
    pub mls_welcome: Welcome,
}

pub fn parse_welcome_message(welcome_message_hex: String) -> Result<Welcome> {
    let welcome_message_bytes = hex::decode(welcome_message_hex)?;
    let welcome_message_in = MlsMessageIn::tls_deserialize(&mut welcome_message_bytes.as_slice())?;
    let welcome = match welcome_message_in.extract() {
        MlsMessageBodyIn::Welcome(welcome) => welcome,
        _ => unreachable!("Failed to parse welcome message"),
    };

    Ok(welcome)
}

#[tauri::command]
pub async fn fetch_welcome_messages_for_user(
    pubkey: String,
    wn: State<'_, Whitenoise>,
) -> Result<Vec<UnsignedEvent>, String> {
    let keys = wn
        .accounts
        .lock()
        .unwrap()
        .get_nostr_keys_for_current_identity()
        .expect("Failed to get nostr keys for current identity")
        .unwrap();

    debug!(target: "nostr_mls::welcome_messages::fetch_welcome_messages_for_user", "Fetching welcome messages for user {:?}", pubkey);

    let gw_events = wn
        .nostr
        .get_events_of(
            vec![Filter::new()
                .kind(Kind::GiftWrap)
                .pubkeys(vec![PublicKey::from_hex(&pubkey).unwrap()])],
            EventSource::Both {
                timeout: Some(DEFAULT_TIMEOUT),
                specific_relays: None,
            },
        )
        .await;

    let mut welcome_events: Vec<UnsignedEvent> = Vec::new();

    if let Ok(events) = gw_events {
        for event in events {
            if let Ok(unwrapped) = extract_rumor(&keys, &event) {
                if unwrapped.rumor.kind == Kind::Custom(444) {
                    welcome_events.push(unwrapped.rumor);
                }
            }
        }
    }

    let mut welcome_messages: Vec<WelcomeMessage> = Vec::new();

    for event in &welcome_events {
        let welcome =
            parse_welcome_message(event.content.clone()).expect("Failed to parse welcome message");

        welcome_messages.push(WelcomeMessage {
            event: event.clone(),
            mls_welcome: welcome,
        });
    }

    debug!(target: "nostr_mls::welcome_messages::fetch_welcome_messages_for_user", "Found {} welcome messages", welcome_messages.len());
    Ok(welcome_events)
}
