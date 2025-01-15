use std::path::Path;
use std::sync::Arc;

use crate::accounts::Account;
use crate::groups::Group;
use crate::invites::Invite;
use nostr_sdk::UnsignedEvent;

#[derive(Debug, Clone)]
pub struct Database {
    env: Arc<heed::Env>,
    /// Active account
    /// Key: "Active"; Value: <account_pubkey>
    active_account_db: heed::Database<heed::types::Str, heed::types::Str>,

    /// Accounts
    /// Key: <account_pubkey>; Value: <Account>
    accounts_db: heed::Database<heed::types::Str, heed::types::SerdeJson<Account>>,

    /// Groups
    /// Key: <mls_group_id>; Value: <Group>
    groups_db: heed::Database<heed::types::Bytes, heed::types::SerdeJson<Group>>,

    /// Invites
    /// The <event_id> is the id of the 444 rumor event (the invite)
    /// Key: <event_id>; Value: <Invite>
    invites_db: heed::Database<heed::types::Str, heed::types::SerdeJson<Invite>>,

    /// Processed Invites Index
    /// This indexes the id of the giftwrapped invites that have been processed so we don't process them again
    /// The <event_id> is the id of the 1059 event that contains the giftwrapped invite
    /// Key: <event_id>; Value: <event_id>;
    processed_invites_index: heed::Database<heed::types::Str, heed::types::Str>,

    /// Messages
    /// Processed (decrypted) messages that are group specific
    /// Key: <mls_group_id> + <timestamp> + <event_id>; Value: <UnsignedEvent>
    messages_db: heed::Database<heed::types::Bytes, heed::types::SerdeJson<UnsignedEvent>>,

    /// Processed Messages Index
    /// This indexes the id of the unprocessed/encrypted 445 messages that have been processed so we don't process them again
    /// Key: <event_id>; Value: <mls_group_id> + <timestamp> + <event_id>;
    processed_messages_index: heed::Database<heed::types::Str, heed::types::Bytes>,
}

/// Errors thrown by the database
#[derive(thiserror::Error, Debug)]
pub enum DatabaseError {
    #[error("LMDB error: {0}")]
    LmdbError(#[from] heed::Error),
    #[error("Failed to serialize data: {0}")]
    SerializationError(String),
    #[error("Failed to deserialize data: {0}")]
    DeserializationError(String),
    #[error("Transaction error: {0}")]
    TransactionError(String),
    #[error("Invalid key: {0}")]
    InvalidKey(String),
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, DatabaseError> {
        // Create directory if it doesn't exist
        std::fs::create_dir_all(&path).map_err(|e| {
            DatabaseError::TransactionError(format!("Failed to create directory: {}", e))
        })?;

        let env = unsafe {
            heed::EnvOpenOptions::new()
                .map_size(10 * 1024 * 1024 * 1024) // 10GB
                .max_dbs(7)
                .open(path)
                .map_err(DatabaseError::LmdbError)?
        };

        let mut wtxn = env
            .write_txn()
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

        let active_account_db = env
            .create_database(&mut wtxn, Some("active_account"))
            .map_err(DatabaseError::LmdbError)?;
        let accounts_db = env
            .create_database(&mut wtxn, Some("accounts"))
            .map_err(DatabaseError::LmdbError)?;
        let groups_db = env
            .create_database(&mut wtxn, Some("groups"))
            .map_err(DatabaseError::LmdbError)?;
        let invites_db = env
            .create_database(&mut wtxn, Some("invites"))
            .map_err(DatabaseError::LmdbError)?;
        let processed_invites_index = env
            .create_database(&mut wtxn, Some("processed_invites_index"))
            .map_err(DatabaseError::LmdbError)?;
        let messages_db = env
            .create_database(&mut wtxn, Some("messages"))
            .map_err(DatabaseError::LmdbError)?;
        let processed_messages_index = env
            .create_database(&mut wtxn, Some("processed_messages_index"))
            .map_err(DatabaseError::LmdbError)?;
        wtxn.commit()
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

        Ok(Self {
            env: Arc::new(env),
            active_account_db,
            accounts_db,
            groups_db,
            invites_db,
            processed_invites_index,
            messages_db,
            processed_messages_index,
        })
    }

    pub fn write_txn(&self) -> Result<heed::RwTxn, DatabaseError> {
        self.env
            .write_txn()
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))
    }

    pub fn read_txn(&self) -> Result<heed::RoTxn, DatabaseError> {
        self.env
            .read_txn()
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))
    }

    pub fn active_account_db(&self) -> &heed::Database<heed::types::Str, heed::types::Str> {
        &self.active_account_db
    }

    pub fn accounts_db(
        &self,
    ) -> &heed::Database<heed::types::Str, heed::types::SerdeJson<Account>> {
        &self.accounts_db
    }

    pub fn groups_db(&self) -> &heed::Database<heed::types::Bytes, heed::types::SerdeJson<Group>> {
        &self.groups_db
    }

    pub fn invites_db(&self) -> &heed::Database<heed::types::Str, heed::types::SerdeJson<Invite>> {
        &self.invites_db
    }

    pub fn messages_db(
        &self,
    ) -> &heed::Database<heed::types::Bytes, heed::types::SerdeJson<UnsignedEvent>> {
        &self.messages_db
    }

    pub fn processed_messages_ids(&self) -> Result<Vec<String>, DatabaseError> {
        let rtxn = self.read_txn()?;
        let iter = self.processed_messages_index.iter(&rtxn)?;
        let mut ids = Vec::new();
        for result in iter {
            let (key, _) = result.map_err(DatabaseError::from)?;
            ids.push(key.to_string());
        }
        Ok(ids)
    }

    /// Deletes all data from the database
    pub fn delete_all_data(&self) -> Result<(), DatabaseError> {
        let mut wtxn = self.env.write_txn()?;

        // Using a macro to reduce repetition
        macro_rules! clear_db {
            ($db:expr, $name:expr) => {
                $db.clear(&mut wtxn)
                    .map_err(|e| DatabaseError::LmdbError(e))?;
            };
        }

        clear_db!(self.active_account_db, "active account");
        clear_db!(self.accounts_db, "accounts");
        clear_db!(self.groups_db, "groups");
        clear_db!(self.invites_db, "invites");
        clear_db!(self.processed_invites_index, "processed_invites_index");
        clear_db!(self.messages_db, "messages");
        clear_db!(self.processed_messages_index, "processed_messages_index");

        wtxn.commit()
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

        Ok(())
    }
}

// Add some helper traits for better error handling
impl From<std::io::Error> for DatabaseError {
    fn from(err: std::io::Error) -> Self {
        DatabaseError::SerializationError(err.to_string())
    }
}
