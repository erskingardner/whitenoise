use crate::database::DatabaseError;
use crate::whitenoise::Whitenoise;
use nostr_sdk::UnsignedEvent;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub event: UnsignedEvent,
}

#[derive(Error, Debug)]
pub enum MessageError {
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] DatabaseError),
}

impl Message {
    /// Creates a compound key for message storage using binary mls_group_id
    /// Key format: mls_group_id bytes + timestamp bytes + event_id bytes
    pub fn create_message_key(
        mls_group_id: &[u8],
        event: &UnsignedEvent,
    ) -> Result<Vec<u8>, MessageError> {
        if event.id.is_none() {
            return Err(MessageError::InvalidParameters(
                "Event ID is required".into(),
            ));
        }
        let timestamp_bytes = event.created_at.as_u64().to_be_bytes();
        let mut key =
            Vec::with_capacity(mls_group_id.len() + 8 + event.id.unwrap().as_bytes().len());

        key.extend_from_slice(mls_group_id);
        key.extend_from_slice(&timestamp_bytes);
        key.extend_from_slice(event.id.unwrap().as_bytes());
        Ok(key)
    }

    /// Saves a message with compound binary key
    pub fn save(
        &self,
        mls_group_id: &[u8],
        event: &UnsignedEvent,
        wn: &Whitenoise,
    ) -> Result<(), MessageError> {
        if mls_group_id.is_empty() {
            return Err(MessageError::InvalidParameters(
                "Missing mls_group_id param".into(),
            ));
        }
        if event.id.is_none() {
            return Err(MessageError::InvalidParameters(
                "Missing event param".into(),
            ));
        }

        let mut wtxn = wn.database.write_txn()?;
        let key = Self::create_message_key(mls_group_id, event)?;
        wn.database
            .messages_db()
            .put(&mut wtxn, &key, event)
            .map_err(|e| DatabaseError::SerializationError(e.to_string()))?;
        wtxn.commit()
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;
        Ok(())
    }
}
