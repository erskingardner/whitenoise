use crate::groups::{Group, GroupType};
use crate::invites::{Invite, InviteState};
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

    // This is scoped so that we can only get the groups that the user is a member of.
    pub fn get_groups(&self, mls_group_ids: Vec<Vec<u8>>) -> Result<Vec<Group>> {
        let state = self
            .state
            .lock()
            .map_err(|e| GroupManagerError::LockError(e.to_string()))?;

        let groups = state
            .groups
            .values()
            .filter(|group| mls_group_ids.contains(&group.mls_group_id))
            .cloned()
            .collect();

        Ok(groups)
    }

    pub fn get_group(&self, mls_group_id: String) -> Result<Group> {
        let state = self
            .state
            .lock()
            .map_err(|e| GroupManagerError::LockError(e.to_string()))?;

        // Decode the hex string into bytes
        let group_id_bytes = hex::decode(&mls_group_id)
            .map_err(|_| GroupManagerError::GroupNotFound(mls_group_id.clone()))?;

        state
            .groups
            .get(&group_id_bytes)
            .cloned()
            .ok_or(GroupManagerError::GroupNotFound(mls_group_id))
    }

    pub fn add_invite(&self, invite: Invite) -> Result<()> {
        {
            let mut state = self
                .state
                .lock()
                .map_err(|e| GroupManagerError::LockError(e.to_string()))?;

            state
                .invites
                .insert(invite.event.id.unwrap().to_hex(), invite);
        }

        self.persist_state()
            .map_err(|e| GroupManagerError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    // This is scoped so that we can only get the invites that the user is a member of.
    pub fn get_invites(&self, mls_group_ids: Vec<Vec<u8>>) -> Result<Vec<Invite>> {
        let state = self
            .state
            .lock()
            .map_err(|e| GroupManagerError::LockError(e.to_string()))?;

        let invites = state
            .invites
            .values()
            .filter(|invite| mls_group_ids.contains(&invite.mls_group_id))
            .cloned()
            .collect();

        Ok(invites)
    }

    pub fn get_invite(&self, invite_id: String) -> Result<Invite> {
        let state = self
            .state
            .lock()
            .map_err(|e| GroupManagerError::LockError(e.to_string()))?;

        state
            .invites
            .get(&invite_id)
            .cloned()
            .ok_or(GroupManagerError::InviteNotFound(invite_id))
    }

    pub fn update_invite_state(&self, invite_id: String, new_state: InviteState) -> Result<Invite> {
        let invite = {
            let mut state = self
                .state
                .lock()
                .map_err(|e| GroupManagerError::LockError(e.to_string()))?;

            let invite = state
                .invites
                .get_mut(&invite_id)
                .ok_or(GroupManagerError::InviteNotFound(invite_id))?;

            invite.state = new_state;
            invite.clone()
        };

        self.persist_state()
            .map_err(|e| GroupManagerError::DatabaseError(e.to_string()))?;

        Ok(invite)
    }
}
