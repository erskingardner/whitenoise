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
    /// A group with only two members
    DirectMessage,
    /// A group with more than two members
    Group,
}

/// Validates the members and admins of a group during creation
///
/// # Arguments
/// * `creator_pubkey` - The public key of the group creator
/// * `member_pubkeys` - List of public keys for group members
/// * `admin_pubkeys` - List of public keys for group admins
///
/// # Returns
/// * `Ok(true)` if validation passes
/// * `Err(GroupManagerError)` if validation fails
///
/// # Validation Rules
/// - Creator must be an admin but not included in member list
/// - Creator must have a valid public key
/// - All member public keys must be valid
/// - All admin public keys must be valid
/// - All admins must also be members (except creator)
///
/// # Errors
/// Returns `GroupManagerError::GroupCreationError` with descriptive message if:
/// - Creator is not an admin
/// - Creator is in member list
/// - Creator has invalid public key
/// - Any member has invalid public key
/// - Any admin has invalid public key
/// - Any admin is not a member
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
