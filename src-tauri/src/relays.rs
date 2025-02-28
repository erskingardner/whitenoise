use nostr_sdk::prelude::*;
use serde::{Deserialize, Serialize};

/// A row in the relays table
#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)]
pub struct RelayRow {
    pub url: String,
    pub relay_type: String,
    pub account_pubkey: String,
    pub group_id: Option<Vec<u8>>,
    pub relay_meta: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Relay {
    pub url: String,
    pub relay_type: RelayType,
    pub account_pubkey: PublicKey,
    pub group_id: Option<Vec<u8>>,
    pub relay_meta: RelayMeta,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum RelayType {
    Nostr,
    Inbox,
    KeyPackage,
    Group,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum RelayMeta {
    Read,
    Write,
    ReadWrite,
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

impl From<String> for RelayMeta {
    fn from(s: String) -> Self {
        match s.as_str() {
            "read" => RelayMeta::Read,
            "write" => RelayMeta::Write,
            "read-write" => RelayMeta::ReadWrite,
            _ => panic!("Invalid relay metadata: {}", s),
        }
    }
}

impl From<RelayMeta> for String {
    fn from(relay_meta: RelayMeta) -> Self {
        match relay_meta {
            RelayMeta::Read => "read".to_string(),
            RelayMeta::Write => "write".to_string(),
            RelayMeta::ReadWrite => "read-write".to_string(),
        }
    }
}

impl From<RelayMeta> for Option<String> {
    fn from(relay_meta: RelayMeta) -> Self {
        match relay_meta {
            RelayMeta::Read => Some("read".to_string()),
            RelayMeta::Write => Some("write".to_string()),
            RelayMeta::ReadWrite => Some("read-write".to_string()),
        }
    }
}

impl From<Option<String>> for RelayMeta {
    fn from(s: Option<String>) -> Self {
        s.map(|s| s.into()).unwrap_or(RelayMeta::ReadWrite)
    }
}

impl RelayMeta {
    pub fn to_relay_metadata(&self) -> Option<RelayMetadata> {
        match self {
            RelayMeta::Read => Some(RelayMetadata::Read),
            RelayMeta::Write => Some(RelayMetadata::Write),
            RelayMeta::ReadWrite => None,
        }
    }
}
