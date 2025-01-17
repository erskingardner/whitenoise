use crate::accounts::{Account, AccountError};
use crate::database::DatabaseError;
use crate::whitenoise::Whitenoise;
use nostr_sdk::{PublicKey, UnsignedEvent};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub event: UnsignedEvent,
}

#[derive(Error, Debug)]
pub enum MessageError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] DatabaseError),

    #[error("Account error: {0}")]
    AccountError(#[from] AccountError),
}

impl Message {
    /// Creates a compound key for message storage using binary pubkey + mls_group_id + timestamp + event_id
    /// Key format: account_pubkey bytes + mls_group_id bytes + timestamp bytes + event_id bytes
    pub fn message_key(
        account_pubkey: PublicKey,
        mls_group_id: &[u8],
        event: &UnsignedEvent,
    ) -> Result<Vec<u8>, MessageError> {
        let timestamp_bytes = event.created_at.as_u64().to_be_bytes();
        let mut key = Vec::with_capacity(
            account_pubkey.to_bytes().len()
                + mls_group_id.len()
                + 8
                + event.id.unwrap().as_bytes().len(),
        );

        key.extend_from_slice(&account_pubkey.to_bytes());
        key.extend_from_slice(mls_group_id);
        key.extend_from_slice(&timestamp_bytes);
        key.extend_from_slice(event.id.unwrap().as_bytes());
        Ok(key)
    }

    pub fn message_iter_key(
        account_pubkey: PublicKey,
        mls_group_id: &[u8],
    ) -> Result<Vec<u8>, MessageError> {
        let mut key = Vec::with_capacity(account_pubkey.to_bytes().len() + mls_group_id.len());
        key.extend_from_slice(&account_pubkey.to_bytes());
        key.extend_from_slice(mls_group_id);
        Ok(key)
    }

    /// Saves a message with compound binary key
    #[allow(dead_code)]
    pub fn save(
        &self,
        mls_group_id: &[u8],
        event: &UnsignedEvent,
        wn: &Whitenoise,
    ) -> Result<(), MessageError> {
        let pubkey = Account::get_active_pubkey(wn).map_err(MessageError::AccountError)?;

        let mut wtxn = wn.database.write_txn()?;
        let key = Self::message_key(pubkey, mls_group_id, event)?;
        wn.database
            .messages_db()
            .put(&mut wtxn, &key, event)
            .map_err(|e| DatabaseError::SerializationError(e.to_string()))?;
        wtxn.commit()
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;
        Ok(())
    }
}
