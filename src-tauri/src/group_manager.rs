use crate::groups::{Group, GroupState, GroupType};
use crate::invites::{Invite, InviteState};
use nostr_openmls::nostr_group_data_extension::NostrGroupDataExtension;
use nostr_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;

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
pub type GroupAndInviteMap = (HashMap<Vec<u8>, Group>, HashMap<String, Invite>);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupManagerState {
    groups: HashMap<Vec<u8>, Group>,
    invites: HashMap<String, Invite>,
}

#[derive(Debug)]
pub struct GroupManager {
    /// The active account's pubkey, should always be the same as the active account in the AccountManager
    pub active_account: Arc<Mutex<String>>,
    pub state: Arc<Mutex<GroupManagerState>>,
    pub db: Arc<sled::Db>,
}

impl GroupManager {
    pub fn new(database: Arc<sled::Db>, active_account_pubkey: String) -> Result<Self> {
        let (groups, invites) =
            GroupManager::load_state(database.clone(), active_account_pubkey.clone())?;

        Ok(GroupManager {
            active_account: Arc::new(Mutex::new(active_account_pubkey)),
            state: Arc::new(Mutex::new(GroupManagerState { groups, invites })),
            db: database,
        })
    }

    pub fn load_state(database: Arc<sled::Db>, pubkey: String) -> Result<GroupAndInviteMap> {
        // Load groups from database
        let mut groups = HashMap::new();
        let groups_tree_name = format!("{}{}", "groups", pubkey);
        let invites_tree_name = format!("{}{}", "invites", pubkey);

        let groups_tree = database
            .open_tree(groups_tree_name)
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
            .open_tree(invites_tree_name)
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
            "Loaded groups & invites state",
        );

        Ok((groups, invites))
    }

    pub fn persist_state(&self) -> Result<()> {
        {
            let pubkey = self
                .active_account
                .lock()
                .map_err(|e| GroupManagerError::LockError(e.to_string()))?;

            let groups_tree_name = format!("{}{}", "groups", *pubkey);
            let invites_tree_name = format!("{}{}", "invites", *pubkey);

            let state = self
                .state
                .lock()
                .map_err(|e| GroupManagerError::LockError(e.to_string()))?;

            tracing::debug!(
                target: "whitenoise::group_manager::write_groups_state",
                "Writing groups state to database tree: {}",
                groups_tree_name
            );

            // Persist groups
            let groups_tree = self
                .db
                .open_tree(groups_tree_name)
                .map_err(|e| GroupManagerError::DatabaseError(e.to_string()))?;

            for (mls_group_id, group) in state.groups.iter() {
                let group_bytes = serde_json::to_vec(group)
                    .map_err(GroupManagerError::GroupSerializationError)?;
                groups_tree
                    .insert(mls_group_id, group_bytes)
                    .map_err(|e| GroupManagerError::DatabaseError(e.to_string()))?;
            }

            tracing::debug!(
                target: "whitenoise::group_manager::write_invites_state",
                "Writing invites state to database tree: {}",
                invites_tree_name
            );

            // Persist invites
            let invites_tree = self
                .db
                .open_tree(invites_tree_name)
                .map_err(|e| GroupManagerError::DatabaseError(e.to_string()))?;

            for (key, invite) in state.invites.iter() {
                let invite_bytes = serde_json::to_vec(invite)
                    .map_err(GroupManagerError::InviteSerializationError)?;
                invites_tree
                    .insert(key, invite_bytes)
                    .map_err(|e| GroupManagerError::DatabaseError(e.to_string()))?;
            }
        }
        // Flush changes to database to be sure they are written
        self.db
            .flush()
            .map_err(|e| GroupManagerError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub fn set_active_account(&self, pubkey: String) -> Result<()> {
        // First update the active account
        {
            let mut active_account = self
                .active_account
                .lock()
                .map_err(|e| GroupManagerError::LockError(e.to_string()))?;
            *active_account = pubkey.clone();
            tracing::debug!(
                target: "whitenoise::group_manager::set_active_account",
                "Active account updated to: {}",
                pubkey
            );
        }

        // Then load the state for the new active account
        let (groups, invites) = GroupManager::load_state(self.db.clone(), pubkey.clone())?;

        // Then update the in-memory state
        {
            let mut state = self
                .state
                .lock()
                .map_err(|e| GroupManagerError::LockError(e.to_string()))?;

            state.groups = groups;
            state.invites = invites;

            tracing::debug!(
                target: "whitenoise::group_manager::set_active_account",
                "Groups state updated: {:?}",
                state.groups
            );

            tracing::debug!(
                target: "whitenoise::group_manager::set_active_account",
                "Invites state updated: {:?}",
                state.invites
            );
        }

        self.persist_state()
    }

    pub fn add_group(
        &self,
        mls_group_id: Vec<u8>,
        mls_group_epoch: u64,
        group_type: GroupType,
        group_data: NostrGroupDataExtension,
    ) -> Result<Group> {
        tracing::debug!("Adding group with ID: {:?}", hex::encode(&mls_group_id));
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
            epoch: mls_group_epoch,
            transcript: Vec::new(),
            state: GroupState::Active,
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

        self.persist_state()?;
        Ok(group)
    }

    pub fn get_groups(&self) -> Result<Vec<Group>> {
        let state = self
            .state
            .lock()
            .map_err(|e| GroupManagerError::LockError(e.to_string()))?;

        let groups = state.groups.values().cloned().collect();

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

    pub fn get_group_by_nostr_id(&self, nostr_group_id: String) -> Result<Group> {
        let state = self
            .state
            .lock()
            .map_err(|e| GroupManagerError::LockError(e.to_string()))?;

        state
            .groups
            .values()
            .find(|group| group.nostr_group_id == nostr_group_id)
            .cloned()
            .ok_or(GroupManagerError::GroupNotFound(nostr_group_id))
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

        self.persist_state()?;

        Ok(())
    }

    pub fn get_invites(&self) -> Result<Vec<Invite>> {
        let state = self
            .state
            .lock()
            .map_err(|e| GroupManagerError::LockError(e.to_string()))?;

        let invites = state.invites.values().cloned().collect();

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
        let new_invite: Invite;
        {
            let mut state = self
                .state
                .lock()
                .map_err(|e| GroupManagerError::LockError(e.to_string()))?;

            let invite = state
                .invites
                .get_mut(&invite_id)
                .ok_or(GroupManagerError::InviteNotFound(invite_id.clone()))?;

            tracing::debug!(
                target: "whitenoise::group_manager::update_invite_state",
                "Updating invite {} state from {:?} to {:?}",
                invite_id,
                invite.state,
                new_state
            );

            invite.state = new_state;
            new_invite = invite.clone();
        };

        self.persist_state()?;

        tracing::debug!(
            target: "whitenoise::group_manager::update_invite_state",
            "After persist, invite {} state is {:?}",
            invite_id,
            new_invite.state
        );

        Ok(new_invite)
    }

    pub fn add_message_to_group(
        &self,
        mls_group_id: Vec<u8>,
        message: UnsignedEvent,
    ) -> Result<Group> {
        let new_group: Group;
        {
            let mut state = self
                .state
                .lock()
                .map_err(|e| GroupManagerError::LockError(e.to_string()))?;

            let group = state
                .groups
                .get_mut(&mls_group_id)
                .ok_or(GroupManagerError::GroupNotFound(hex::encode(&mls_group_id)))?;

            group.transcript.push(message.clone());
            group
                .transcript
                .sort_by(|a, b| a.created_at.cmp(&b.created_at));
            group.last_message_id = Some(message.id.unwrap().to_string());
            group.last_message_at = Some(Timestamp::now());
            new_group = group.clone();
        }

        self.persist_state()?;

        Ok(new_group)
    }

    pub fn delete_all_data(&self) -> Result<()> {
        {
            let mut state = self
                .state
                .lock()
                .map_err(|e| GroupManagerError::LockError(e.to_string()))?;

            state.groups.clear();
            state.invites.clear();

            let mut active_account = self
                .active_account
                .lock()
                .map_err(|e| GroupManagerError::LockError(e.to_string()))?;

            *active_account = String::new();
        }
        Ok(())
    }
}
