use nostr_sdk::UnsignedEvent;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Invite {
    /// The event that contains the welcome message
    pub event: UnsignedEvent,
    /// MLS group id
    pub mls_group_id: Vec<u8>,
    /// Nostr group id (from NostrGroupDataExtension)
    pub nostr_group_id: String,
    /// Group name (from NostrGroupDataExtension)
    pub group_name: String,
    /// Group description (from NostrGroupDataExtension)
    pub group_description: String,
    /// Group admin pubkeys (from NostrGroupDataExtension)
    pub group_admin_pubkeys: Vec<String>,
    /// Group relays (from NostrGroupDataExtension)
    pub group_relays: Vec<String>,
    /// Pubkey of the user that sent the invite
    pub inviter: String,
    /// Member count of the group
    pub member_count: usize,
    /// The state of the invite
    pub state: InviteState,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum InviteState {
    Pending,
    Accepted,
    Declined,
}
