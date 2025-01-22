use serde::{Deserialize, Serialize};

/// A row in the relays table
#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)]
pub struct RelayRow {
    pub url: String,
    pub relay_type: String,
    pub account_pubkey: Option<String>,
    pub group_id: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Relay {
    pub url: String,
    pub relay_type: RelayType,
    pub account_pubkey: Option<String>,
    pub group_id: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum RelayType {
    Nostr,
    Inbox,
    KeyPackage,
    Group,
}

impl From<String> for RelayType {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "nostr" => RelayType::Nostr,
            "inbox" => RelayType::Inbox,
            "key_package" => RelayType::KeyPackage,
            "group" => RelayType::Group,
            _ => panic!("Invalid relay type: {}", s),
        }
    }
}

impl From<RelayType> for String {
    fn from(relay_type: RelayType) -> Self {
        match relay_type {
            RelayType::Nostr => "nostr".to_string(),
            RelayType::Inbox => "inbox".to_string(),
            RelayType::KeyPackage => "key_package".to_string(),
            RelayType::Group => "group".to_string(),
        }
    }
}
