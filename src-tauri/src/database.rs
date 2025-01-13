use std::path::Path;
use std::sync::Arc;

use crate::account_manager::Account;
use crate::groups::Group;
use crate::invites::Invite;
use nostr_sdk::UnsignedEvent;

#[derive(Debug, Clone)]
pub struct Database {
    env: Arc<heed::Env>,
    /// Key: "Active"; Value: Account pubkey
    active_account_db: heed::Database<heed::types::Str, heed::types::Str>,
    /// Key: Account pubkey; Value: Account
    accounts_db: heed::Database<heed::types::Str, heed::types::SerdeJson<Account>>,
    /// Key: MLS Group ID; Value: Group
    groups_db: heed::Database<heed::types::Bytes, heed::types::SerdeJson<Group>>,
    /// Key: Nostr event ID; Value: Invite
    invites_db: heed::Database<heed::types::Str, heed::types::SerdeJson<Invite>>,
    /// Key: Compound binary key (group_id + timestamp + event_id); Value: UnsignedEvent
    chats_db: heed::Database<heed::types::Bytes, heed::types::SerdeJson<UnsignedEvent>>,
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
    #[error("Account not found: {0}")]
    AccountNotFound(String),
    #[error("Group not found: {0}")]
    GroupNotFound(String),
    #[error("Invite not found: {0}")]
    InviteNotFound(String),
    #[error("No active account set")]
    NoActiveAccount,
    #[error("Transaction error: {0}")]
    TransactionError(String),
    #[error("Database is full")]
    DatabaseFull,
    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, DatabaseError> {
        let env = unsafe {
            heed::EnvOpenOptions::new()
                .map_size(100 * 1024 * 1024)
                .max_dbs(5)
                .open(path)
                .map_err(|e| DatabaseError::LmdbError(e))?
        };

        let mut wtxn = env
            .write_txn()
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

        let active_account_db = env
            .create_database(&mut wtxn, Some("active_account"))
            .map_err(|e| DatabaseError::LmdbError(e))?;
        let accounts_db = env
            .create_database(&mut wtxn, Some("accounts"))
            .map_err(|e| DatabaseError::LmdbError(e))?;
        let groups_db = env
            .create_database(&mut wtxn, Some("groups"))
            .map_err(|e| DatabaseError::LmdbError(e))?;
        let invites_db = env
            .create_database(&mut wtxn, Some("invites"))
            .map_err(|e| DatabaseError::LmdbError(e))?;
        let chats_db = env
            .create_database(&mut wtxn, Some("chats"))
            .map_err(|e| DatabaseError::LmdbError(e))?;

        wtxn.commit()
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

        Ok(Self {
            env: Arc::new(env),
            active_account_db,
            accounts_db,
            groups_db,
            invites_db,
            chats_db,
        })
    }

    pub fn write_txn(&self) -> Result<heed::RwTxn, DatabaseError> {
        Ok(self.env.write_txn()?)
    }

    pub fn read_txn(&self) -> Result<heed::RoTxn, DatabaseError> {
        Ok(self.env.read_txn()?)
    }

    /// Gets the active account pubkey
    pub fn get_active_account(&self) -> Result<String, DatabaseError> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

        match self
            .active_account_db
            .get(&rtxn, "Active")
            .map_err(|e| DatabaseError::LmdbError(e))?
        {
            Some(pubkey) => Ok(pubkey.to_string()),
            None => Err(DatabaseError::NoActiveAccount),
        }
    }

    /// Sets the active account pubkey
    pub fn set_active_account(&self, pubkey: &str) -> Result<(), DatabaseError> {
        if pubkey.is_empty() {
            return Err(DatabaseError::InvalidKeyFormat("Empty pubkey".into()));
        }

        let mut wtxn = self
            .env
            .write_txn()
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

        self.active_account_db
            .put(&mut wtxn, "Active", pubkey)
            .map_err(|e| DatabaseError::LmdbError(e))?;

        wtxn.commit()
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

        Ok(())
    }

    /// Gets an account by its pubkey
    pub fn get_account(&self, pubkey: &str) -> Result<Account, DatabaseError> {
        if pubkey.is_empty() {
            return Err(DatabaseError::InvalidKeyFormat("Empty pubkey".into()));
        }

        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

        self.accounts_db
            .get(&rtxn, pubkey)
            .map_err(|e| DatabaseError::LmdbError(e))?
            .ok_or_else(|| DatabaseError::AccountNotFound(pubkey.to_string()))
    }

    /// Saves an account
    pub fn save_account(&self, account: &Account) -> Result<(), DatabaseError> {
        if account.pubkey.is_empty() {
            return Err(DatabaseError::InvalidKeyFormat(
                "Empty account pubkey".into(),
            ));
        }

        let mut wtxn = self
            .env
            .write_txn()
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

        self.accounts_db
            .put(&mut wtxn, &account.pubkey, account)
            .map_err(|e| DatabaseError::SerializationError(e.to_string()))?;

        wtxn.commit()
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

        Ok(())
    }

    /// Gets all accounts with better error handling
    pub fn get_all_accounts(&self) -> Result<Vec<Account>, DatabaseError> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

        let iter = self
            .accounts_db
            .iter(&rtxn)
            .map_err(|e| DatabaseError::LmdbError(e))?;

        iter.map(|r| r.map(|(_, v)| v))
            .map(|res| res.map_err(|e| DatabaseError::DeserializationError(e.to_string())))
            .collect()
    }

    /// Gets a group by its MLS group ID
    pub fn get_group(&self, group_id: &[u8]) -> Result<Option<Group>, DatabaseError> {
        let rtxn = self.env.read_txn()?;

        self.groups_db
            .get(&rtxn, group_id)
            .map_err(|e| DatabaseError::LmdbError(e))
    }

    /// Saves a group
    pub fn save_group(&self, group_id: &[u8], group: &Group) -> Result<(), DatabaseError> {
        let mut wtxn = self.env.write_txn()?;
        self.groups_db.put(&mut wtxn, group_id, group)?;
        wtxn.commit()?;
        Ok(())
    }

    /// Gets all groups
    pub fn get_all_groups(&self) -> Result<Vec<Group>, DatabaseError> {
        let rtxn = self.env.read_txn()?;
        let iter = self.groups_db.iter(&rtxn)?;
        let groups: Result<Vec<_>, _> = iter.map(|r| r.map(|(_, v)| v)).collect();
        Ok(groups?)
    }

    /// Gets an invite by its Nostr event ID
    pub fn get_invite(&self, event_id: &str) -> Result<Option<Invite>, DatabaseError> {
        let rtxn = self.env.read_txn()?;
        self.invites_db
            .get(&rtxn, event_id)
            .map_err(|e| DatabaseError::LmdbError(e))
    }

    /// Saves an invite
    pub fn save_invite(&self, event_id: &str, invite: &Invite) -> Result<(), DatabaseError> {
        let mut wtxn = self.env.write_txn()?;
        self.invites_db.put(&mut wtxn, event_id, invite)?;
        wtxn.commit()?;
        Ok(())
    }

    /// Gets all invites
    pub fn get_all_invites(&self) -> Result<Vec<Invite>, DatabaseError> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

        let iter = self.invites_db.iter(&rtxn)?;
        let invites: Result<Vec<_>, _> = iter.map(|r| r.map(|(_, v)| v)).collect();
        Ok(invites?)
    }

    /// Creates a compound key for chat storage using binary group_id
    /// Key format: group_id bytes + timestamp bytes + event_id bytes
    fn create_chat_key(
        &self,
        group_id: &[u8],
        event: &UnsignedEvent,
    ) -> Result<Vec<u8>, DatabaseError> {
        if event.id.is_none() {
            return Err(DatabaseError::InvalidKeyFormat(
                "Event ID is required".into(),
            ));
        }
        let timestamp_bytes = event.created_at.as_u64().to_be_bytes();
        let mut key = Vec::with_capacity(group_id.len() + 8 + event.id.unwrap().as_bytes().len());

        key.extend_from_slice(group_id);
        key.extend_from_slice(&timestamp_bytes);
        key.extend_from_slice(event.id.unwrap().as_bytes());
        Ok(key)
    }

    /// Saves a chat message with compound binary key
    pub fn save_chat(&self, group_id: &[u8], event: &UnsignedEvent) -> Result<(), DatabaseError> {
        let mut wtxn = self
            .env
            .write_txn()
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

        let key = self.create_chat_key(group_id, event)?;
        self.chats_db.put(&mut wtxn, &key, event)?;
        wtxn.commit()?;
        Ok(())
    }

    /// Gets chat messages for a given MLS group ID, optionally filtered by time range
    pub fn get_chats(
        &self,
        group_id: &[u8],
        start_time: Option<u64>,
        end_time: Option<u64>,
    ) -> Result<Vec<UnsignedEvent>, DatabaseError> {
        let rtxn = self.env.read_txn()?;

        let iter = self
            .chats_db
            .prefix_iter(&rtxn, group_id)?
            .filter(|result| {
                if let Ok((key, _)) = result {
                    if key.len() >= group_id.len() + 8 {
                        // Extract timestamp from the key (8 bytes after group_id)
                        let timestamp_bytes = &key[group_id.len()..group_id.len() + 8];
                        if let Ok(timestamp_array) = timestamp_bytes.try_into() {
                            let timestamp = u64::from_be_bytes(timestamp_array);
                            let after_start = start_time.map_or(true, |start| timestamp >= start);
                            let before_end = end_time.map_or(true, |end| timestamp <= end);
                            return after_start && before_end;
                        }
                    }
                }
                true
            });

        let chats: Result<Vec<_>, _> = iter.map(|r| r.map(|(_, v)| v)).collect();
        Ok(chats?)
    }

    /// Deletes all data from the database
    pub fn delete_all_data(&self) -> Result<(), DatabaseError> {
        let mut wtxn = self
            .env
            .write_txn()
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

        // Using a macro to reduce repetition
        macro_rules! clear_db {
            ($db:expr, $name:expr) => {
                $db.clear(&mut wtxn).map_err(|e| {
                    DatabaseError::TransactionError(format!(
                        "Failed to clear {} database: {}",
                        $name, e
                    ))
                })?;
            };
        }

        clear_db!(self.active_account_db, "active account");
        clear_db!(self.accounts_db, "accounts");
        clear_db!(self.groups_db, "groups");
        clear_db!(self.invites_db, "invites");
        clear_db!(self.chats_db, "chats");

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
