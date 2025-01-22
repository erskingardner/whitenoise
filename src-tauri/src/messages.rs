use nostr_sdk::{PublicKey, Tags, Timestamp, UnsignedEvent};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct MessageRow {
    pub event_id: String,
    pub account_pubkey: String,
    pub author_pubkey: String,
    pub mls_group_id: Vec<u8>,
    pub created_at: u64,
    pub content: String,
    pub tags: String,  // JSON string for Vec<Vec<String>>
    pub event: String, // JSON string for UnsignedEvent
    pub outer_event_id: String,
    pub processed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub event_id: String,
    pub account_pubkey: PublicKey,
    pub author_pubkey: PublicKey,
    pub mls_group_id: Vec<u8>,
    pub created_at: Timestamp,
    pub content: String,
    pub tags: Tags,
    pub event: UnsignedEvent,
    pub outer_event_id: String,
    pub processed: bool,
}
