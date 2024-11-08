use crate::utils::is_valid_hex_pubkey;
use nostr_sdk::prelude::*;
use openmls_nostr::nostr_group_data_extension::NostrGroupDataExtension;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;

const GROUP_DB_TREE_NAME: &str = "groups";
const INVITE_DB_TREE_NAME: &str = "invites";

#[derive(Error, Debug)]
pub enum GroupManagerError {
    #[error("Group already exists: {}", hex::encode(.0))]
    GroupAlreadyExists(Vec<u8>),

    #[error("Invite already exists: {0}")]
    InviteAlreadyExists(String),

    #[error("Group not found: {0}")]
    GroupNotFound(String),

    #[error("Invite not found: {0}")]
    InviteNotFound(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Failed to serialize group: {0}")]
    GroupSerializationError(serde_json::Error),

    #[error("Failed to serialize invite: {0}")]
    InviteSerializationError(serde_json::Error),

    #[error("Failed to acquire lock: {0}")]
    LockError(String),

    #[error("Group creation error: {0}")]
    GroupCreationError(String),
}

pub type Result<T> = std::result::Result<T, GroupManagerError>;

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Invite {
    /// The event that contains the welcome message
    pub event: UnsignedEvent,
    /// MLS group id (from NostrGroupDataExtension)
    pub mls_group_id: String,
    /// Group name (from NostrGroupDataExtension)
    pub group_name: String,
    /// Group description (from NostrGroupDataExtension)
    pub group_description: String,
    /// Group admin pubkeys (from NostrGroupDataExtension)
    pub group_admin_pubkeys: Vec<String>,
    /// Group relays (from NostrGroupDataExtension)
    pub group_relays: Vec<String>,
    /// Pubkey of the user that sent the invite
    pub invitee: String,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupManagerState {
    groups: HashMap<Vec<u8>, Group>,
    invites: HashMap<String, Invite>,
}

#[derive(Debug)]
pub struct GroupManager {
    state: Mutex<GroupManagerState>,
    db: Arc<sled::Db>,
}

impl GroupManager {
    pub fn new(database: Arc<sled::Db>) -> Result<Self> {
        // Load groups from database
        let mut groups = HashMap::new();
        let groups_tree = database
            .open_tree(GROUP_DB_TREE_NAME)
            .map_err(|e| GroupManagerError::DatabaseError(e.to_string()))?;

        for result in groups_tree.iter() {
            let (key, value) =
                result.map_err(|e| GroupManagerError::DatabaseError(e.to_string()))?;
            let group: Group = serde_json::from_slice(&value)
                .map_err(GroupManagerError::GroupSerializationError)?;
            groups.insert(key.to_vec(), group);
        }

        // Load invites from database
        let mut invites = HashMap::new();
        let invites_tree = database
            .open_tree(INVITE_DB_TREE_NAME)
            .map_err(|e| GroupManagerError::DatabaseError(e.to_string()))?;

        for result in invites_tree.iter() {
            let (key, value) =
                result.map_err(|e| GroupManagerError::DatabaseError(e.to_string()))?;
            let invite: Invite = serde_json::from_slice(&value)
                .map_err(GroupManagerError::InviteSerializationError)?;
            invites.insert(String::from_utf8_lossy(&key).to_string(), invite);
        }

        tracing::debug!(
            target: "whitenoise::group_manager::new",
            "Loaded groups state: {:?}",
            groups
        );

        tracing::debug!(
            target: "whitenoise::group_manager::new",
            "Loaded invites state: {:?}",
            invites
        );

        Ok(GroupManager {
            state: Mutex::new(GroupManagerState { groups, invites }),
            db: database,
        })
    }

    fn persist_state(&self) -> Result<()> {
        tracing::debug!(
            target: "whitenoise::group_manager::persist_state",
            "Persisting groups state"
        );

        let state = self
            .state
            .lock()
            .map_err(|e| GroupManagerError::LockError(e.to_string()))?;

        // Persist groups
        let groups_tree = self
            .db
            .open_tree(GROUP_DB_TREE_NAME)
            .map_err(|e| GroupManagerError::DatabaseError(e.to_string()))?;

        for (mls_group_id, group) in state.groups.iter() {
            let group_bytes =
                serde_json::to_vec(group).map_err(GroupManagerError::GroupSerializationError)?;
            groups_tree
                .insert(mls_group_id, group_bytes)
                .map_err(|e| GroupManagerError::DatabaseError(e.to_string()))?;
        }

        // Persist invites
        let invites_tree = self
            .db
            .open_tree(INVITE_DB_TREE_NAME)
            .map_err(|e| GroupManagerError::DatabaseError(e.to_string()))?;

        for (key, invite) in state.invites.iter() {
            let invite_bytes =
                serde_json::to_vec(invite).map_err(GroupManagerError::InviteSerializationError)?;
            invites_tree
                .insert(key, invite_bytes)
                .map_err(|e| GroupManagerError::DatabaseError(e.to_string()))?;
        }

        // Flush changes to database to be sure they are written
        self.db
            .flush()
            .map_err(|e| GroupManagerError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub fn add_group(
        &self,
        mls_group_id: Vec<u8>,
        group_type: GroupType,
        group_data: NostrGroupDataExtension,
    ) -> Result<Group> {
        let group = Group {
            mls_group_id,
            nostr_group_id: group_data.nostr_group_id(),
            name: group_data.name(),
            description: group_data.description(),
            admin_pubkeys: group_data.admin_pubkeys(),
            last_message_id: None,
            last_message_at: None,
            relay_urls: group_data.relays(),
            group_type,
            transcript: Vec::new(),
        };

        {
            let mut state = self
                .state
                .lock()
                .map_err(|e| GroupManagerError::LockError(e.to_string()))?;

            if state.groups.contains_key(&group.mls_group_id) {
                return Err(GroupManagerError::GroupAlreadyExists(group.mls_group_id));
            }

            state
                .groups
                .insert(group.mls_group_id.clone(), group.clone());
        }

        self.persist_state()
            .map_err(|e| GroupManagerError::DatabaseError(e.to_string()))?;

        Ok(group)
    }

    pub fn get_groups(&self) -> Result<Vec<Group>> {
        let state = self
            .state
            .lock()
            .map_err(|e| GroupManagerError::LockError(e.to_string()))?;

        Ok(state.groups.values().cloned().collect())
    }

    pub fn get_invites(&self) -> Result<Vec<Invite>> {
        let state = self
            .state
            .lock()
            .map_err(|e| GroupManagerError::LockError(e.to_string()))?;

        Ok(state.invites.values().cloned().collect())
    }
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
