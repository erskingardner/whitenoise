use crate::accounts::Account;
use crate::Whitenoise;
use nostr_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MessageError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Account error: {0}")]
    Account(#[from] crate::accounts::AccountError),
}

pub type Result<T> = std::result::Result<T, MessageError>;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct MessageRow {
    pub event_id: String,
    pub account_pubkey: String,
    pub author_pubkey: String,
    pub mls_group_id: Vec<u8>,
    pub created_at: u64,
    pub content: String,
    pub tags: String,  // JSON string for Vec<Vec<String>>
    pub event: String, // JSON string for UnsignedEvent
    pub outer_event_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub event_id: EventId,
    pub account_pubkey: PublicKey,
    pub author_pubkey: PublicKey,
    pub mls_group_id: Vec<u8>,
    pub created_at: Timestamp,
    pub content: String,
    pub tags: Tags,
    pub event: UnsignedEvent,
    pub outer_event_id: EventId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProcessedMessageState {
    Processed,
    Failed,
}

impl From<String> for ProcessedMessageState {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "processed" => ProcessedMessageState::Processed,
            "failed" => ProcessedMessageState::Failed,
            _ => panic!("Invalid processed message state: {}", s),
        }
    }
}

impl From<ProcessedMessageState> for String {
    fn from(state: ProcessedMessageState) -> Self {
        match state {
            ProcessedMessageState::Processed => "processed".to_string(),
            ProcessedMessageState::Failed => "failed".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct ProcessedMessageRow {
    pub event_id: String,
    pub message_event_id: Option<String>,
    pub account_pubkey: String,
    pub processed_at: u64,
    pub state: String,
    pub failure_reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessedMessage {
    pub event_id: EventId,
    pub message_event_id: Option<EventId>,
    pub account_pubkey: PublicKey,
    pub processed_at: u64,
    pub state: ProcessedMessageState,
    pub failure_reason: String,
}

impl From<ProcessedMessageRow> for ProcessedMessage {
    fn from(row: ProcessedMessageRow) -> Self {
        ProcessedMessage {
            event_id: EventId::parse(&row.event_id).unwrap(),
            message_event_id: row.message_event_id.map(|id| EventId::parse(&id).unwrap()),
            account_pubkey: PublicKey::from_hex(&row.account_pubkey).unwrap(),
            processed_at: row.processed_at,
            state: ProcessedMessageState::from(row.state),
            failure_reason: row.failure_reason,
        }
    }
}

impl From<ProcessedMessage> for ProcessedMessageRow {
    fn from(message: ProcessedMessage) -> Self {
        ProcessedMessageRow {
            event_id: message.event_id.to_string(),
            message_event_id: message.message_event_id.map(|id| id.to_string()),
            account_pubkey: message.account_pubkey.to_hex(),
            processed_at: message.processed_at,
            state: String::from(message.state),
            failure_reason: message.failure_reason,
        }
    }
}

impl ProcessedMessage {
    pub async fn find_by_event_id(
        event_id: EventId,
        wn: tauri::State<'_, Whitenoise>,
    ) -> Result<Option<ProcessedMessage>> {
        let active_account = Account::get_active(wn.clone()).await?;

        let processed_message_row = sqlx::query_as::<_, ProcessedMessageRow>(
            "SELECT * FROM processed_messages WHERE event_id = ? AND account_pubkey = ?",
        )
        .bind(event_id.to_string())
        .bind(active_account.pubkey.to_hex())
        .fetch_optional(&wn.database.pool)
        .await?;

        match processed_message_row {
            Some(row) => Ok(Some(row.into())),
            None => Ok(None),
        }
    }

    #[allow(dead_code)]
    pub async fn failed_with_reason(
        wn: tauri::State<'_, Whitenoise>,
    ) -> Result<Vec<(EventId, String)>> {
        let active_account = Account::get_active(wn.clone()).await?;

        let processed_message_rows = sqlx::query_as::<_, ProcessedMessageRow>(
            "SELECT * FROM processed_messages WHERE state = 'failed' AND account_pubkey = ?",
        )
        .bind(active_account.pubkey.to_hex())
        .fetch_all(&wn.database.pool)
        .await?;
        Ok(processed_message_rows
            .into_iter()
            .map(|row| (EventId::parse(&row.event_id).unwrap(), row.failure_reason))
            .collect())
    }

    pub async fn create_with_state_and_reason(
        event_id: EventId,
        message_event_id: Option<EventId>,
        state: ProcessedMessageState,
        reason: String,
        wn: tauri::State<'_, Whitenoise>,
    ) -> Result<ProcessedMessage> {
        let active_account = Account::get_active(wn.clone()).await?;

        let mut txn = wn.database.pool.begin().await?;
        let processed_at = chrono::Utc::now().timestamp() as u64;
        sqlx::query("INSERT INTO processed_messages (event_id, message_event_id, account_pubkey, processed_at, state, failure_reason) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(event_id.to_string())
            .bind(message_event_id.map(|id| id.to_string()))
            .bind(active_account.pubkey.to_hex())
            .bind(processed_at as i64)
            .bind(String::from(state.clone()))
            .bind(reason.clone())
            .execute(&mut *txn)
            .await?;
        txn.commit().await?;

        Ok(ProcessedMessage {
            event_id,
            message_event_id,
            account_pubkey: active_account.pubkey,
            processed_at,
            state,
            failure_reason: reason,
        })
    }
}
