use crate::nostr::DEFAULT_TIMEOUT;
use crate::nostr_mls::key_packages::{
    delete_key_package_from_relays, generate_and_publish_key_package,
};
use crate::nostr_mls::nostr_group_data::NostrGroupDataExtension;
use crate::whitenoise::Whitenoise;
use anyhow::{anyhow, Result};
use bincode;
use log::debug;
use nostr_sdk::prelude::*;
use openmls::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::mem;
use tauri::State;
use tls_codec::Deserialize as TlsDeserialize;

/// Key used to store and retrieve welcome messages data in the database
const WELCOME_MESSAGES_KEY: &str = "welcomes";

#[derive(Serialize, Deserialize)]
pub struct WelcomeMessage {
    /// The event that contains the welcome message
    pub event: UnsignedEvent,
    /// Nostr group data
    pub nostr_group_data: Option<NostrGroupDataExtension>,
    /// Serialized raw MLS processed welcome message
    #[serde(
        serialize_with = "serialize_processed_welcome",
        deserialize_with = "deserialize_processed_welcome"
    )]
    pub processed_welcome: Option<ProcessedWelcome>,
    /// Pubkey of the user that sent the invite
    pub invitee: String,
}

// Custom serialization for ProcessedWelcome
fn serialize_processed_welcome<S>(
    processed_welcome: &Option<ProcessedWelcome>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match processed_welcome {
        Some(pw) => {
            let bytes = unsafe {
                std::slice::from_raw_parts(
                    (pw as *const ProcessedWelcome) as *const u8,
                    mem::size_of::<ProcessedWelcome>(),
                )
            };
            serializer.serialize_some(bytes)
        }
        None => serializer.serialize_none(),
    }
}

// Custom deserialization for ProcessedWelcome
fn deserialize_processed_welcome<'de, D>(
    deserializer: D,
) -> Result<Option<ProcessedWelcome>, D::Error>
where
    D: Deserializer<'de>,
{
    let bytes: Option<Vec<u8>> = Option::deserialize(deserializer)?;
    match bytes {
        Some(b) => {
            if b.len() != mem::size_of::<ProcessedWelcome>() {
                return Err(serde::de::Error::custom(
                    "Invalid byte length for ProcessedWelcome",
                ));
            }
            let pw = unsafe {
                let ptr = b.as_ptr() as *const ProcessedWelcome;
                ptr.read()
            };
            Ok(Some(pw))
        }
        None => Ok(None),
    }
}

impl WelcomeMessage {
    fn welcome_message_tree_key(wn: State<'_, Whitenoise>) -> Result<String> {
        let accounts = wn.accounts.lock().expect("Failed to lock accounts");
        let current_identity = match &accounts.current_identity {
            Some(identity) => identity,
            None => return Err(anyhow!("No current identity found")),
        };
        Ok(format!("{}{}", WELCOME_MESSAGES_KEY, current_identity))
    }

    pub fn save(&self, wn: State<'_, Whitenoise>) -> Result<()> {
        let key = match &self.event.id {
            Some(id) => id.to_string(),
            None => return Err(anyhow!("Event ID is None")),
        };
        let serialized = bincode::serialize(self)?;
        wn.wdb.insert_in_tree(
            WelcomeMessage::welcome_message_tree_key(wn.clone())?,
            key,
            &serialized,
        )?;
        Ok(())
    }

    pub fn get_welcomes(wn: State<'_, Whitenoise>) -> Result<Vec<WelcomeMessage>> {
        let welcome_tree = wn
            .wdb
            .db
            .open_tree(WelcomeMessage::welcome_message_tree_key(wn.clone())?)?;
        let welcomes = welcome_tree
            .iter()
            .filter_map(|result| {
                result
                    .ok()
                    .and_then(|(_, value)| bincode::deserialize::<WelcomeMessage>(&value).ok())
            })
            .collect::<Vec<WelcomeMessage>>();
        debug!(target: "nostr_mls::welcome_messages::get_welcomes", "Loaded {:?} welcomes from the database", welcomes.len());
        Ok(welcomes)
    }
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
) -> Result<Vec<WelcomeMessage>, String> {
    let keys = wn
        .accounts
        .lock()
        .unwrap()
        .get_nostr_keys_for_current_identity()
        .expect("Failed to get nostr keys for current identity")
        .unwrap();

    debug!(target: "nostr_mls::welcome_messages::fetch_welcome_messages_for_user", "Fetching welcome messages for user {:?}", pubkey);

    // TODO: Should only fetch from inbox relays
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
        debug!(target: "nostr_mls::welcome_messages::fetch_welcome_messages_for_user", "Found {:?} Gift-wrapped messages for user {:?}", events.len(), pubkey);
        for event in events {
            if let Ok(unwrapped) = extract_rumor(&keys, &event) {
                if unwrapped.rumor.kind == Kind::Custom(444) {
                    welcome_events.push(unwrapped.rumor);
                }
            }
        }
    }

    // Get the welcome messages that we have already processed from the database
    let mut welcome_messages: Vec<WelcomeMessage> = WelcomeMessage::get_welcomes(wn.clone())
        .expect("Failed to get welcome messages from the database");

    let mls_group_config = MlsGroupJoinConfig::builder()
        .use_ratchet_tree_extension(true)
        .build();

    // For each welcome message that we find, we need to process it and then save it to the database
    // so that we can create a group from it later if the user accepts the invite.
    for event in &welcome_events {
        match (|| async {
            // Check if the welcome message has already been processed and bail early if it has
            let event_id = match &event.id {
                Some(id) => id.to_string(),
                None => return Err(anyhow!("Event ID is None")),
            };
            if welcome_messages
                .iter()
                .any(|wm| wm.event.id.unwrap().to_string() == event_id)
            {
                debug!(target: "nostr_mls::welcome_messages::fetch_welcome_messages_for_user", "Welcome message {:?} already processed", event_id);
                return Ok(None);
            }

            // Parse the welcome message and process it
            let welcome = parse_welcome_message(event.content.clone())?;

            let processed_welcome = ProcessedWelcome::new_from_welcome(
                &wn.nostr_mls.provider,
                &mls_group_config,
                welcome,
            )?;

            // Get the unverified group info and create Nostr group data from it
            let unverified_group_info = processed_welcome.unverified_group_info();
            let nostr_group_data = NostrGroupDataExtension::from_group_info(unverified_group_info)?;
            debug!(target: "nostr_mls::welcome_messages::fetch_welcome_messages_for_user", "Nostr group data: {:?}", nostr_group_data);

            // Check the key_package used, delete from relays and replace with a new one
            let key_package_event_id: Option<&str> = event
                .tags
                .iter()
                .find(|tag| {
                    tag.kind() == TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::E))
                })
                .map(|tag| tag.content().unwrap());

            // TODO: Refactor this into a background task
            if let Some(key_package_event_id) = key_package_event_id {
                let event_id = EventId::parse(key_package_event_id).unwrap();

                if let Err(e) = delete_key_package_from_relays(event_id, wn.clone()).await {
                    debug!(target: "nostr_mls::welcome_messages", "Failed to delete key package: {:?}", e);
                }
                if let Err(e) = generate_and_publish_key_package(pubkey.clone(), wn.clone()).await {
                    debug!(target: "nostr_mls::welcome_messages", "Failed to generate and publish new key package: {:?}", e);
                }
            }

            // Create the welcome message and save it to the database
            let welcome_message = WelcomeMessage {
                event: event.clone(),
                nostr_group_data: Some(nostr_group_data),
                processed_welcome: Some(processed_welcome),
                invitee: event.pubkey.to_string(),
            };

            welcome_message.save(wn.clone())?;
            Ok(Some(welcome_message))
        })().await {
            Ok(Some(welcome_message)) => welcome_messages.push(welcome_message),
            Ok(None) => continue,
            Err(e) => {
                debug!(target: "nostr_mls::welcome_messages::fetch_welcome_messages_for_user", "Failed to process welcome message: {:?}", e)
            }
        }
    }

    debug!(target: "nostr_mls::welcome_messages::fetch_welcome_messages_for_user", "Found {} welcome messages", welcome_messages.len());

    // TODO: We need to filter and only show the latest welcome message for a given group if there are duplicates
    Ok(welcome_messages)
}
