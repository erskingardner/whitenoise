use crate::database::DatabaseError;
use crate::Whitenoise;
use nostr_sdk::{JsonUtil, UnsignedEvent};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InviteError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("Event error: {0}")]
    Event(#[from] nostr_sdk::event::unsigned::Error),

    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, InviteError>;

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)]
pub struct InviteRow {
    pub event_id: String,
    pub account_pubkey: String,
    pub event: String, // JSON string for UnsignedEvent
    pub mls_group_id: Vec<u8>,
    pub nostr_group_id: String,
    pub group_name: String,
    pub group_description: String,
    pub group_admin_pubkeys: String, // JSON array of strings
    pub group_relays: String,        // JSON array of strings
    pub inviter: String,
    pub member_count: u32,
    pub outer_event_id: String,
    pub processed: bool,
    pub state: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Invite {
    /// The event id of the invite
    pub event_id: String,
    /// The account that owns this invite
    pub account_pubkey: String,
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
    pub member_count: u32,
    /// The state of the invite
    pub state: InviteState,
    /// The event id of the 1059 event that contained the invite
    pub outer_event_id: String,
    /// Whether the invite has been processed
    pub processed: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum InviteState {
    Pending,
    Accepted,
    Declined,
    Ignored,
}

impl From<String> for InviteState {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "pending" => InviteState::Pending,
            "accepted" => InviteState::Accepted,
            "declined" => InviteState::Declined,
            "ignored" => InviteState::Ignored,
            _ => panic!("Invalid invite state: {}", s),
        }
    }
}

impl From<InviteState> for String {
    fn from(state: InviteState) -> Self {
        match state {
            InviteState::Pending => "pending".to_string(),
            InviteState::Accepted => "accepted".to_string(),
            InviteState::Declined => "declined".to_string(),
            InviteState::Ignored => "ignored".to_string(),
        }
    }
}

impl Invite {
    pub async fn find_by_id(
        account_pubkey: &str,
        invite_event_id: &str,
        wn: tauri::State<'_, Whitenoise>,
    ) -> Result<Invite> {
        let invite_row = sqlx::query_as::<_, InviteRow>(
            "SELECT * FROM invites WHERE account_pubkey = ? AND event_id = ?",
        )
        .bind(account_pubkey)
        .bind(invite_event_id)
        .fetch_one(&wn.database.pool)
        .await?;

        Ok(Invite {
            event_id: invite_row.event_id,
            account_pubkey: invite_row.account_pubkey,
            event: UnsignedEvent::from_json(&invite_row.event)?,
            mls_group_id: invite_row.mls_group_id,
            nostr_group_id: invite_row.nostr_group_id,
            group_name: invite_row.group_name,
            group_description: invite_row.group_description,
            group_admin_pubkeys: serde_json::from_str(&invite_row.group_admin_pubkeys)?,
            group_relays: serde_json::from_str(&invite_row.group_relays)?,
            inviter: invite_row.inviter,
            member_count: invite_row.member_count,
            state: InviteState::from(invite_row.state),
            outer_event_id: invite_row.outer_event_id,
            processed: invite_row.processed,
        })
    }

    pub async fn save(&self, wn: tauri::State<'_, Whitenoise>) -> Result<Invite> {
        let mut txn = wn.database.pool.begin().await?;
        sqlx::query("INSERT INTO invites (event_id, account_pubkey, event, mls_group_id, nostr_group_id, group_name, group_description, group_admin_pubkeys, group_relays, inviter, member_count, outer_event_id, processed, state) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(&self.event_id)
            .bind(&self.account_pubkey)
            .bind(serde_json::to_string(&self.event)?)
            .bind(&self.mls_group_id)
            .bind(&self.nostr_group_id)
            .bind(&self.group_name)
            .bind(&self.group_description)
            .bind(serde_json::to_string(&self.group_admin_pubkeys)?)
            .bind(serde_json::to_string(&self.group_relays)?)
            .bind(&self.inviter)
            .bind(self.member_count)
            .bind(&self.outer_event_id)
            .bind(self.processed)
            .bind(String::from(self.state.clone()))
            .execute(&mut *txn)
            .await?;
        txn.commit().await?;
        Ok(self.clone())
    }

    // pub fn new(event: UnsignedEvent, database: &Database) -> Result<Invite> {}
    // pub fn find_by_event_id(event_id: &str, database: &Database) -> Result<Option<Invite>> {}
    // pub fn fetch_invites_from_relays(database: &Database) -> Result<()> {}

    // pub fn accept(&self, database: &Database) -> Result<()> {}
    // pub fn decline(&self, database: &Database) -> Result<()> {}
    // pub fn remove(&self, database: &Database) -> Result<()> {}
}
