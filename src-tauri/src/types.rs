use nostr_sdk::prelude::*;
use serde::{Deserialize, Serialize};

/// A contact enriched with Nostr metadata and relay information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnrichedContact {
    /// The Nostr metadata of the contact.
    pub metadata: Metadata,
    /// Whether the contact supports NIP-17.
    pub nip17: bool,
    /// Whether the contact supports NIP-104.
    pub nip104: bool,
    /// The relays for the contact's inbox.
    pub inbox_relays: Vec<String>,
    /// The relays for the contact's key package.
    pub key_package_relays: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NostrEncryptionMethod {
    Nip04,
    Nip44,
}
