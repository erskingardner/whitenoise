use crate::group_manager::{GroupManagerError, Result};
use crate::utils::is_valid_hex_pubkey;
use nostr_sdk::{Timestamp, UnsignedEvent};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Group {
    /// This is the MLS group ID, this will serve as the PK in the DB and doesn't change
    pub mls_group_id: Vec<u8>,
    /// Hex encoded (same value as the NostrGroupDataExtension) this is the group_id used in Nostr events
    pub nostr_group_id: String,
    /// UTF-8 encoded (same value as the NostrGroupDataExtension)
    pub name: String,
    /// UTF-8 encoded (same value as the NostrGroupDataExtension)
    pub description: String,
    /// Hex encoded (same value as the NostrGroupDataExtension)
    pub admin_pubkeys: Vec<String>,
    /// Hex encoded Nostr event ID of the last message in the group
    pub last_message_id: Option<String>,
    /// Timestamp of the last message in the group
    pub last_message_at: Option<Timestamp>,
    /// URLs of the Nostr relays this group is using
    pub relay_urls: Vec<String>,
    /// Type of Nostr MLS group
    pub group_type: GroupType,
    /// Chat transscript
    pub transcript: Vec<UnsignedEvent>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GroupType {
    DirectMessage,
    Group,
}

pub fn validate_group_members(
    creator_pubkey: &String,
    member_pubkeys: &[String],
    admin_pubkeys: &[String],
) -> Result<bool> {
    // Creator must be an admin
    if !admin_pubkeys.contains(creator_pubkey) {
        return Err(GroupManagerError::GroupCreationError(
            "Creator must be an admin".to_string(),
        ));
    }

    // Creator must not be included as a member
    if member_pubkeys.contains(creator_pubkey) {
        return Err(GroupManagerError::GroupCreationError(
            "Creator must not be included as a member".to_string(),
        ));
    }

    // Creator must be valid pubkey
    if !is_valid_hex_pubkey(creator_pubkey) {
        return Err(GroupManagerError::GroupCreationError(format!(
            "Invalid creator pubkey: {}",
            creator_pubkey
        )));
    }

    // Check that members are valid pubkeys
    for pubkey in member_pubkeys.iter() {
        if !is_valid_hex_pubkey(pubkey) {
            return Err(GroupManagerError::GroupCreationError(format!(
                "Invalid member pubkey: {}",
                pubkey
            )));
        }
    }

    // Check that admins are valid pubkeys and are members
    for pubkey in admin_pubkeys.iter() {
        if !is_valid_hex_pubkey(pubkey) {
            return Err(GroupManagerError::GroupCreationError(format!(
                "Invalid admin pubkey: {}",
                pubkey
            )));
        }
        if !member_pubkeys.contains(pubkey) && creator_pubkey != pubkey {
            return Err(GroupManagerError::GroupCreationError(
                "Admin must be a member".to_string(),
            ));
        }
    }
    Ok(true)
}
