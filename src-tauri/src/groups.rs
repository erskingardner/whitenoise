use crate::accounts::{Account, AccountError};
use crate::database::DatabaseError;
use crate::messages::Message;
use crate::utils::is_valid_hex_pubkey;
use crate::Whitenoise;
use nostr_openmls::groups::GroupError as NostrMlsError;
use nostr_openmls::nostr_group_data_extension::NostrGroupDataExtension;
use nostr_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

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
    /// Epoch of the group
    pub epoch: u64,
    /// The state of the group
    pub state: GroupState,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GroupType {
    /// A group with only two members
    DirectMessage,
    /// A group with more than two members
    Group,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GroupState {
    Active,
    Inactive,
}

#[derive(Error, Debug)]
pub enum GroupError {
    #[error("Group not found")]
    GroupNotFound,

    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),

    #[error("Account error: {0}")]
    AccountError(#[from] AccountError),

    #[error("Database error: {0}")]
    DatabaseError(#[from] DatabaseError),

    #[error("MLS error: {0}")]
    MlsError(#[from] NostrMlsError),

    #[error("Key error: {0}")]
    KeyError(#[from] nostr_sdk::key::Error),
}

pub type Result<T> = std::result::Result<T, GroupError>;

impl Group {
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
            return Err(GroupError::InvalidParameters(
                "Creator must be an admin".to_string(),
            ));
        }

        // Creator must not be included as a member
        if member_pubkeys.contains(creator_pubkey) {
            return Err(GroupError::InvalidParameters(
                "Creator must not be included as a member".to_string(),
            ));
        }

        // Creator must be valid pubkey
        if !is_valid_hex_pubkey(creator_pubkey) {
            return Err(GroupError::InvalidParameters(format!(
                "Invalid creator pubkey: {}",
                creator_pubkey
            )));
        }

        // Check that members are valid pubkeys
        for pubkey in member_pubkeys.iter() {
            if !is_valid_hex_pubkey(pubkey) {
                return Err(GroupError::InvalidParameters(format!(
                    "Invalid member pubkey: {}",
                    pubkey
                )));
            }
        }

        // Check that admins are valid pubkeys and are members
        for pubkey in admin_pubkeys.iter() {
            if !is_valid_hex_pubkey(pubkey) {
                return Err(GroupError::InvalidParameters(format!(
                    "Invalid admin pubkey: {}",
                    pubkey
                )));
            }
            if !member_pubkeys.contains(pubkey) && creator_pubkey != pubkey {
                return Err(GroupError::InvalidParameters(
                    "Admin must be a member".to_string(),
                ));
            }
        }
        Ok(true)
    }

    /// Creates a compound key for group storage using account pubkey and mls_group_id
    fn create_group_key(account_pubkey: &str, mls_group_id: &[u8]) -> Vec<u8> {
        [account_pubkey.as_bytes(), mls_group_id].concat()
    }

    /// Create and save a new group to the database
    pub fn new(
        account_pubkey: &str,
        mls_group_id: Vec<u8>,
        mls_group_epoch: u64,
        group_type: GroupType,
        group_data: NostrGroupDataExtension,
        wn: &Whitenoise,
        _app_handle: &tauri::AppHandle,
    ) -> Result<Group> {
        tracing::debug!("Creating group with ID: {:?}", hex::encode(&mls_group_id));
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
            state: GroupState::Active,
        };

        group.save(account_pubkey, wn)?;

        // Add the group to the account
        let mut account =
            Account::find_by_pubkey(account_pubkey, wn).map_err(GroupError::AccountError)?;
        account.mls_group_ids.push(group.mls_group_id.clone());
        account.nostr_group_ids.push(group.nostr_group_id.clone());
        account.save(wn).map_err(GroupError::AccountError)?;

        Ok(group)
    }

    /// Find a group by their mls_group_id and the account it belongs to
    pub fn find_by_mls_group_id(
        account_pubkey: &str,
        mls_group_id: &[u8],
        wn: &Whitenoise,
    ) -> Result<Group> {
        if account_pubkey.is_empty() {
            return Err(GroupError::InvalidParameters(
                "Missing account_pubkey param".into(),
            ));
        }
        if mls_group_id.is_empty() {
            return Err(GroupError::InvalidParameters(
                "Missing mls_group_id param".into(),
            ));
        }

        let rtxn = &wn.database.read_txn()?;
        let key = Self::create_group_key(account_pubkey, mls_group_id);
        wn.database
            .groups_db()
            .get(rtxn, &key)
            .map_err(DatabaseError::LmdbError)?
            .ok_or_else(|| GroupError::GroupNotFound)
    }

    pub fn get_by_nostr_group_id(
        account_pubkey: &str,
        nostr_group_id: &str,
        wn: &Whitenoise,
    ) -> Result<Group> {
        if account_pubkey.is_empty() {
            return Err(GroupError::InvalidParameters(
                "Missing account_pubkey param".into(),
            ));
        }
        if nostr_group_id.is_empty() {
            return Err(GroupError::InvalidParameters(
                "Missing nostr_group_id param".into(),
            ));
        }

        let rtxn = wn.database.read_txn()?;
        let mut iter = wn
            .database
            .groups_db()
            .prefix_iter(&rtxn, account_pubkey.as_bytes())
            .map_err(|e| DatabaseError::SerializationError(e.to_string()))?;

        iter.find(|result| {
            result
                .as_ref()
                .map(|(_, group)| group.nostr_group_id == nostr_group_id)
                .unwrap_or(false)
        })
        .ok_or(GroupError::GroupNotFound)?
        .map(|(_, group)| group)
        .map_err(|e| DatabaseError::DeserializationError(e.to_string()).into())
    }

    /// Gets all groups for a given account
    pub fn get_all_groups(account_pubkey: &str, wn: &Whitenoise) -> Result<Vec<Group>> {
        let rtxn = wn.database.read_txn()?;
        let iter = wn
            .database
            .groups_db()
            .prefix_iter(&rtxn, account_pubkey.as_bytes())
            .map_err(|e| DatabaseError::SerializationError(e.to_string()))?;
        iter.map(|result| {
            result
                .map(|(_, group)| group)
                .map_err(|e| GroupError::from(DatabaseError::DeserializationError(e.to_string())))
        })
        .collect()
    }

    // Save the group to the database
    pub fn save(&self, account_pubkey: &str, wn: &Whitenoise) -> Result<Group> {
        if account_pubkey.is_empty() {
            return Err(GroupError::InvalidParameters(
                "Missing account_pubkey param".into(),
            ));
        }
        if self.mls_group_id.is_empty() {
            return Err(GroupError::InvalidParameters(
                "Missing mls_group_id param".into(),
            ));
        }

        let mut wtxn = wn.database.write_txn()?;
        let key = Self::create_group_key(account_pubkey, &self.mls_group_id);
        wn.database
            .groups_db()
            .put(&mut wtxn, &key, self)
            .map_err(|e| DatabaseError::SerializationError(e.to_string()))?;
        wtxn.commit()
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;
        Ok(self.clone())
    }

    pub fn add_message(&self, message: UnsignedEvent, wn: &Whitenoise) -> Result<()> {
        let mut wtxn = wn.database.write_txn()?;
        let key = Message::create_message_key(&self.mls_group_id, &message)
            .map_err(|e| DatabaseError::InvalidKey(e.to_string()))?;
        wn.database
            .messages_db()
            .put(&mut wtxn, &key, &message)
            .map_err(|e| DatabaseError::SerializationError(e.to_string()))?;
        wtxn.commit()
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;
        Ok(())
    }

    pub fn messages(
        &self,
        start_time: Option<u64>,
        end_time: Option<u64>,
        wn: &Whitenoise,
    ) -> Result<Vec<UnsignedEvent>> {
        let rtxn = wn.database.read_txn()?;

        let mut messages: Vec<(u64, UnsignedEvent)> = wn
            .database
            .messages_db()
            .prefix_iter(&rtxn, &self.mls_group_id)
            .map_err(|e| GroupError::DatabaseError(DatabaseError::LmdbError(e)))?
            .filter(|result| {
                if let Ok((key, _)) = result {
                    if key.len() >= self.mls_group_id.len() + 8 {
                        // Extract timestamp from the key (8 bytes after group_id)
                        let timestamp_bytes =
                            &key[self.mls_group_id.len()..self.mls_group_id.len() + 8];
                        if let Ok(timestamp_array) = timestamp_bytes.try_into() {
                            let timestamp = u64::from_be_bytes(timestamp_array);
                            let after_start = start_time.map_or(true, |start| timestamp >= start);
                            let before_end = end_time.map_or(true, |end| timestamp <= end);
                            return after_start && before_end;
                        }
                    }
                }
                true
            })
            .filter_map(|r| {
                r.ok().and_then(|(key, event)| {
                    if key.len() >= self.mls_group_id.len() + 8 {
                        let timestamp_bytes =
                            &key[self.mls_group_id.len()..self.mls_group_id.len() + 8];
                        timestamp_bytes
                            .try_into()
                            .ok()
                            .map(|b| (u64::from_be_bytes(b), event))
                    } else {
                        None
                    }
                })
            })
            .collect();

        // Sort by timestamp in ascending order
        messages.sort_by_key(|(timestamp, _)| *timestamp);

        // Return only the events
        Ok(messages.into_iter().map(|(_, event)| event).collect())
    }

    pub fn members(&self, wn: &tauri::State<'_, Whitenoise>) -> Result<Vec<PublicKey>> {
        let nostr_mls = wn.nostr_mls.lock().unwrap();
        let member_pubkeys = nostr_mls
            .member_pubkeys(self.mls_group_id.clone())
            .map_err(|e| GroupError::MlsError(e))?;
        member_pubkeys
            .iter()
            .try_fold(Vec::with_capacity(member_pubkeys.len()), |mut acc, pk| {
                acc.push(PublicKey::parse(pk)?);
                Ok(acc)
            })
    }

    pub fn admins(&self) -> Result<Vec<PublicKey>> {
        self.admin_pubkeys.iter().try_fold(
            Vec::with_capacity(self.admin_pubkeys.len()),
            |mut acc, pk| {
                acc.push(PublicKey::parse(pk)?);
                Ok(acc)
            },
        )
    }

    // pub fn remove(&self, wn: &tauri::State<'_, Whitenoise>) -> Result<()> {}
}
