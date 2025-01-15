use crate::database::DatabaseError;
use crate::Whitenoise;
use nostr_sdk::UnsignedEvent;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InviteError {
    #[error("Invite not found")]
    InviteNotFound,

    #[error("Missing pubkey")]
    MissingPubkey,

    #[error("Missing event id")]
    MissingEventId,

    #[error("Database error: {0}")]
    DatabaseError(#[from] DatabaseError),
}

pub type Result<T> = std::result::Result<T, InviteError>;

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

impl Invite {
    /// Creates a compound key for invite storage using account pubkey and invite event id
    fn create_invite_key(pubkey: &str, invite_event_id: &str) -> String {
        format!("{pubkey}{invite_event_id}")
    }

    pub fn find_by_id(
        account_pubkey: &str,
        invite_event_id: &str,
        wn: &Whitenoise,
    ) -> Result<Invite> {
        let rtxn = wn.database.read_txn()?;
        let key = Self::create_invite_key(account_pubkey, invite_event_id);
        wn.database
            .invites_db()
            .get(&rtxn, &key)
            .map_err(|e| DatabaseError::SerializationError(e.to_string()))?
            .ok_or(InviteError::InviteNotFound)
    }

    pub fn save(&self, account_pubkey: &str, wn: &Whitenoise) -> Result<Invite> {
        if account_pubkey.is_empty() {
            return Err(InviteError::MissingPubkey);
        }
        if self.event.id.is_none() {
            return Err(InviteError::MissingEventId);
        }

        let mut wtxn = wn
            .database
            .write_txn()
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

        let key = Self::create_invite_key(
            account_pubkey,
            self.event.id.as_ref().unwrap().to_hex().as_str(),
        );
        wn.database
            .invites_db()
            .put(&mut wtxn, &key, self)
            .map_err(|e| DatabaseError::SerializationError(e.to_string()))?;
        wtxn.commit()
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;
        Ok(self.clone())
    }

    // pub fn new(event: UnsignedEvent, database: &Database) -> Result<Invite> {}
    // pub fn find_by_event_id(event_id: &str, database: &Database) -> Result<Option<Invite>> {}
    // pub fn fetch_invites_from_relays(database: &Database) -> Result<()> {}

    // pub fn accept(&self, database: &Database) -> Result<()> {}
    // pub fn decline(&self, database: &Database) -> Result<()> {}
    // pub fn remove(&self, database: &Database) -> Result<()> {}
}
