use crate::database::DatabaseError;
use crate::groups::{Group, GroupRow};
use crate::invites::{Invite, InviteRow};
use crate::nostr_manager;
use crate::relays::RelayType;
use crate::secrets_store;
use crate::Whitenoise;
use nostr_openmls::NostrMls;
use nostr_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use tauri::Emitter;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AccountError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] DatabaseError),

    #[error("Failed to parse public key: {0}")]
    PublicKeyError(#[from] nostr_sdk::key::Error),

    #[error("Nostr Manager error: {0}")]
    NostrManagerError(#[from] nostr_manager::NostrManagerError),

    #[error("Error with secrets store: {0}")]
    SecretsStoreError(#[from] secrets_store::SecretsStoreError),

    #[error("Tauri error: {0}")]
    TauriError(#[from] tauri::Error),

    #[error("No active account found")]
    NoActiveAccount,

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("SQLx error: {0}")]
    SqlxError(#[from] sqlx::Error),
}

pub type Result<T> = std::result::Result<T, AccountError>;

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)]
pub struct ActiveAccount {
    pub pubkey: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)]
pub struct AccountSettings {
    pub dark_theme: bool,
    pub dev_mode: bool,
    pub lockdown_mode: bool,
}

impl Default for AccountSettings {
    fn default() -> Self {
        Self {
            dark_theme: true,
            dev_mode: false,
            lockdown_mode: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, sqlx::FromRow)]
pub struct AccountOnboarding {
    pub inbox_relays: bool,
    pub key_package_relays: bool,
    pub publish_key_package: bool,
}

/// This is an intermediate struct representing an account in the database
#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)]
pub struct AccountRow {
    pub pubkey: String,
    pub metadata: String,   // JSON string
    pub settings: String,   // JSON string
    pub onboarding: String, // JSON string
    pub last_used: u64,
    pub last_synced: u64,
    pub active: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Account {
    pub pubkey: PublicKey,
    pub metadata: Metadata,
    pub settings: AccountSettings,
    pub onboarding: AccountOnboarding,
    pub last_used: Timestamp,
    pub last_synced: Timestamp,
    pub active: bool,
}

impl Account {
    /// Generates a new keypair and saves the mostly blank account to the database
    pub async fn new(wn: tauri::State<'_, Whitenoise>) -> Result<Account> {
        let keys = Keys::generate();
        let account = Account {
            pubkey: keys.public_key(),
            metadata: Metadata::default(),
            settings: AccountSettings::default(),
            onboarding: AccountOnboarding::default(),
            last_used: Timestamp::now(),
            last_synced: Timestamp::zero(),
            active: false,
        };
        let account = account.save(wn.clone()).await?;

        // If the record saves, add the keys to the secret store
        secrets_store::store_private_key(&keys, &wn.data_dir)?;

        Ok(account)
    }
    /// Adds an account from an existing keypair
    pub async fn add_from_keys(
        keys: &Keys,
        set_active: bool,
        wn: tauri::State<'_, Whitenoise>,
        app_handle: &tauri::AppHandle,
    ) -> Result<Account> {
        let pubkey = keys.public_key();

        tracing::debug!(target: "whitenoise::accounts", "Adding account for pubkey: {}", pubkey.to_hex());

        // Fetch metadata & relays from Nostr
        let metadata = wn
            .nostr
            .fetch_user_metadata(pubkey)
            .await
            .map_err(AccountError::NostrManagerError);
        tracing::debug!(target: "whitenoise::accounts", "Fetched metadata for pubkey: {}", pubkey.to_hex());
        let nostr_relays = wn
            .nostr
            .fetch_user_relays(pubkey)
            .await
            .map_err(AccountError::NostrManagerError);
        tracing::debug!(target: "whitenoise::accounts", "Fetched relays for pubkey: {}", pubkey.to_hex());
        let inbox_relays = wn
            .nostr
            .fetch_user_inbox_relays(pubkey)
            .await
            .map_err(AccountError::NostrManagerError);
        tracing::debug!(target: "whitenoise::accounts", "Fetched inbox relays for pubkey: {}", pubkey.to_hex());
        let key_package_relays = wn
            .nostr
            .fetch_user_key_package_relays(pubkey)
            .await
            .map_err(AccountError::NostrManagerError);
        tracing::debug!(target: "whitenoise::accounts", "Fetched key package relays for pubkey: {}", pubkey.to_hex());
        let key_packages = wn
            .nostr
            .fetch_user_key_packages(pubkey)
            .await
            .map_err(AccountError::NostrManagerError)?;
        tracing::debug!(target: "whitenoise::accounts", "Fetched key packages for pubkey: {}", pubkey.to_hex());

        let mut onboarding = AccountOnboarding::default();

        let unwrapped_metadata = match metadata {
            Ok(Some(metadata)) => metadata.to_owned(),
            _ => Metadata::default(),
        };

        let nostr_relays_unwrapped = nostr_relays.unwrap_or_default();
        let inbox_relays_unwrapped = inbox_relays.unwrap_or_default();
        let key_package_relays_unwrapped = key_package_relays.unwrap_or_default();

        if !inbox_relays_unwrapped.is_empty() {
            onboarding.inbox_relays = true;
        }
        if !key_package_relays_unwrapped.is_empty() {
            onboarding.key_package_relays = true;
        }
        if !key_packages.is_empty() {
            onboarding.publish_key_package = true;
        }

        tracing::debug!(target: "whitenoise::accounts", "Creating account with metadata: {:?}", unwrapped_metadata);

        let account = Account {
            pubkey,
            metadata: unwrapped_metadata,
            settings: AccountSettings::default(),
            onboarding,
            last_used: Timestamp::now(),
            last_synced: Timestamp::zero(),
            active: false,
        };

        tracing::debug!(target: "whitenoise::accounts", "Saving new account to database");
        account.save(wn.clone()).await?;

        tracing::debug!(target: "whitenoise::accounts", "Inserting nostr relays, {:?}", nostr_relays_unwrapped);
        account
            .update_relays(RelayType::Nostr, &nostr_relays_unwrapped, wn.clone())
            .await?;

        tracing::debug!(target: "whitenoise::accounts", "Inserting inbox relays, {:?}", inbox_relays_unwrapped);
        account
            .update_relays(RelayType::Inbox, &inbox_relays_unwrapped, wn.clone())
            .await?;

        tracing::debug!(target: "whitenoise::accounts", "Inserting key package relays, {:?}", key_package_relays_unwrapped);
        account
            .update_relays(
                RelayType::KeyPackage,
                &key_package_relays_unwrapped,
                wn.clone(),
            )
            .await?;

        tracing::debug!(target: "whitenoise::accounts", "Storing private key");
        secrets_store::store_private_key(keys, &wn.data_dir)?;

        // Set active if requested
        if set_active {
            account.set_active(wn.clone(), app_handle).await?;
        }

        Ok(account)
    }

    /// Finds an account by its public key
    pub async fn find_by_pubkey(
        pubkey: &PublicKey,
        wn: tauri::State<'_, Whitenoise>,
    ) -> Result<Account> {
        let mut txn = wn.database.pool.begin().await?;

        let row = sqlx::query_as::<_, AccountRow>("SELECT * FROM accounts WHERE pubkey = ?")
            .bind(pubkey.to_hex().as_str())
            .fetch_one(&mut *txn)
            .await?;

        Ok(Account {
            pubkey: PublicKey::parse(row.pubkey.as_str())?,
            metadata: serde_json::from_str(&row.metadata)?,
            settings: serde_json::from_str(&row.settings)?,
            onboarding: serde_json::from_str(&row.onboarding)?,
            last_used: Timestamp::from(row.last_used),
            last_synced: Timestamp::from(row.last_synced),
            active: row.active,
        })
    }

    /// Returns all accounts
    pub async fn all(wn: tauri::State<'_, Whitenoise>) -> Result<Vec<Account>> {
        let mut txn = wn.database.pool.begin().await?;

        let iter = sqlx::query_as::<_, AccountRow>("SELECT * FROM accounts")
            .fetch_all(&mut *txn)
            .await?;

        iter.into_iter()
            .map(|row| -> Result<Account> {
                Ok(Account {
                    pubkey: PublicKey::parse(row.pubkey.as_str())?,
                    metadata: serde_json::from_str(&row.metadata)?,
                    settings: serde_json::from_str(&row.settings)?,
                    onboarding: serde_json::from_str(&row.onboarding)?,
                    last_used: Timestamp::from(row.last_used),
                    last_synced: Timestamp::from(row.last_synced),
                    active: row.active,
                })
            })
            .collect::<Result<Vec<_>>>()
    }

    /// Returns the currently active account
    pub async fn get_active(wn: tauri::State<'_, Whitenoise>) -> Result<Account> {
        // First validate/fix the active state
        Self::validate_active_state(wn.clone()).await?;

        let mut txn = wn.database.pool.begin().await?;

        let row = sqlx::query_as::<_, AccountRow>("SELECT * FROM accounts WHERE active = TRUE")
            .fetch_optional(&mut *txn)
            .await?;

        match row {
            Some(row) => Ok(Account {
                pubkey: PublicKey::parse(row.pubkey.as_str())?,
                metadata: serde_json::from_str(&row.metadata)?,
                settings: serde_json::from_str(&row.settings)?,
                onboarding: serde_json::from_str(&row.onboarding)?,
                last_used: Timestamp::from(row.last_used),
                last_synced: Timestamp::from(row.last_synced),
                active: row.active,
            }),
            None => Err(AccountError::NoActiveAccount),
        }
    }

    /// Returns the public key of the currently active account
    ///
    /// # Arguments
    /// * `wn` - Whitenoise state handle
    ///
    /// # Returns
    /// * `Ok(PublicKey)` - Public key of active account if successful
    /// * `Err(AccountError)` - Error if no active account or invalid public key
    ///
    /// # Errors
    /// Returns error if:
    /// - No active account is found
    /// - Active account's public key is invalid
    pub async fn get_active_pubkey(wn: tauri::State<'_, Whitenoise>) -> Result<PublicKey> {
        // First validate/fix the active state
        Self::validate_active_state(wn.clone()).await?;

        let mut txn = wn.database.pool.begin().await?;

        let active_pubkey =
            sqlx::query_scalar::<_, String>("SELECT pubkey FROM accounts WHERE active = TRUE")
                .fetch_optional(&mut *txn)
                .await?;

        match active_pubkey {
            Some(pubkey) => Ok(PublicKey::parse(pubkey.as_str())?),
            None => Err(AccountError::NoActiveAccount),
        }
    }

    /// Sets the active account in the database and updates nostr for the active identity
    pub async fn set_active(
        &self,
        wn: tauri::State<'_, Whitenoise>,
        app_handle: &tauri::AppHandle,
    ) -> Result<Account> {
        tracing::debug!(
            target: "whitenoise::accounts::set_active",
            "Starting set_active for pubkey: {}",
            self.pubkey.to_hex()
        );

        let mut txn = wn.database.pool.begin().await?;

        // First set all accounts to inactive
        sqlx::query("UPDATE accounts SET active = FALSE")
            .execute(&mut *txn)
            .await?;

        // Then set this account to active
        sqlx::query(
            r#"
            UPDATE accounts
            SET active = TRUE,
                last_used = ?
            WHERE pubkey = ?
        "#,
        )
        .bind(Timestamp::now().to_string())
        .bind(self.pubkey.to_hex().as_str())
        .execute(&mut *txn)
        .await?;

        txn.commit().await?;

        // Validate the active state as a safeguard
        Self::validate_active_state(wn.clone()).await?;

        // If the database operation is successful, update Nostr client
        wn.nostr
            .set_nostr_identity(self, wn.clone(), app_handle)
            .await?;

        tracing::debug!(
            target: "whitenoise::accounts::set_active",
            "Nostr identity set for: {}",
            self.pubkey.to_hex()
        );

        app_handle.emit("nostr_ready", ())?;

        // Then update Nostr MLS instance
        {
            let mut nostr_mls = wn.nostr_mls.lock().await;
            *nostr_mls = NostrMls::new(wn.data_dir.clone(), Some(self.pubkey.to_hex()));
        }

        tracing::debug!(
            target: "whitenoise::accounts::set_active",
            "Nostr MLS updated for: {}",
            self.pubkey.to_hex()
        );

        app_handle.emit("account_changed", ())?;

        tracing::debug!(
            target: "whitenoise::accounts::set_active",
            "Set active completed successfully for: {}",
            self.pubkey.to_hex()
        );

        Ok(self.clone())
    }

    /// Returns the groups the account is a member of
    pub async fn groups(&self, wn: tauri::State<'_, Whitenoise>) -> Result<Vec<Group>> {
        let mut txn = wn.database.pool.begin().await?;

        let iter = sqlx::query_as::<_, GroupRow>("SELECT * FROM groups WHERE account_pubkey = ?")
            .bind(self.pubkey.to_hex().as_str())
            .fetch_all(&mut *txn)
            .await?;

        iter.into_iter()
            .map(|row| -> Result<Group> {
                Ok(Group {
                    mls_group_id: row.mls_group_id,
                    account_pubkey: PublicKey::parse(row.account_pubkey.as_str())?,
                    nostr_group_id: row.nostr_group_id,
                    name: row.name,
                    description: row.description,
                    admin_pubkeys: serde_json::from_str(&row.admin_pubkeys)?,
                    last_message_id: row.last_message_id,
                    last_message_at: row.last_message_at.map(Timestamp::from),
                    group_type: row.group_type.into(),
                    epoch: row.epoch,
                    state: row.state.into(),
                })
            })
            .collect::<Result<Vec<_>>>()
    }

    /// Returns the invites the account has received
    #[allow(dead_code)]
    pub async fn invites(&self, wn: tauri::State<'_, Whitenoise>) -> Result<Vec<Invite>> {
        let mut txn = wn.database.pool.begin().await?;

        let invite_rows =
            sqlx::query_as::<_, InviteRow>("SELECT * FROM invites WHERE account_pubkey = ?")
                .bind(self.pubkey.to_hex().as_str())
                .fetch_all(&mut *txn)
                .await?;

        invite_rows
            .into_iter()
            .map(|row| -> Result<Invite> {
                Ok(Invite {
                    event_id: row.event_id,
                    account_pubkey: row.account_pubkey,
                    event: serde_json::from_str(&row.event)?,
                    mls_group_id: row.mls_group_id,
                    nostr_group_id: row.nostr_group_id,
                    group_name: row.group_name,
                    group_description: row.group_description,
                    group_admin_pubkeys: serde_json::from_str(&row.group_admin_pubkeys)?,
                    group_relays: serde_json::from_str(&row.group_relays)?,
                    inviter: row.inviter,
                    member_count: row.member_count,
                    state: row.state.into(),
                    outer_event_id: row.outer_event_id,
                })
            })
            .collect::<Result<Vec<_>>>()
    }

    pub async fn nostr_group_ids(&self, wn: tauri::State<'_, Whitenoise>) -> Result<Vec<String>> {
        Ok(self
            .groups(wn)
            .await?
            .iter()
            .map(|g| g.nostr_group_id.clone())
            .collect())
    }

    #[allow(dead_code)]
    pub async fn mls_group_ids(&self, wn: tauri::State<'_, Whitenoise>) -> Result<Vec<Vec<u8>>> {
        Ok(self
            .groups(wn)
            .await?
            .iter()
            .map(|g| g.mls_group_id.clone())
            .collect())
    }

    pub fn keys(&self, wn: tauri::State<'_, Whitenoise>) -> Result<Keys> {
        Ok(secrets_store::get_nostr_keys_for_pubkey(
            self.pubkey.to_hex().as_str(),
            &wn.data_dir,
        )?)
    }

    pub async fn relays(
        &self,
        relay_type: RelayType,
        wn: tauri::State<'_, Whitenoise>,
    ) -> Result<Vec<String>> {
        Ok(sqlx::query_scalar::<_, String>(
            "SELECT url FROM account_relays WHERE relay_type = ? AND account_pubkey = ?",
        )
        .bind(String::from(relay_type))
        .bind(self.pubkey.to_hex().as_str())
        .fetch_all(&wn.database.pool)
        .await?)
    }

    pub async fn update_relays(
        &self,
        relay_type: RelayType,
        relays: &Vec<String>,
        wn: tauri::State<'_, Whitenoise>,
    ) -> Result<Account> {
        if relays.is_empty() {
            return Ok(self.clone());
        }

        let mut txn = wn.database.pool.begin().await?;

        // Then insert the new relays
        for relay in relays {
            sqlx::query(
                "INSERT OR REPLACE INTO account_relays (url, relay_type, account_pubkey)
                 VALUES (?, ?, ?)",
            )
            .bind(relay)
            .bind(String::from(relay_type))
            .bind(self.pubkey.to_hex())
            .execute(&mut *txn)
            .await?;
        }

        txn.commit().await?;

        Ok(self.clone())
    }

    /// Saves the account to the database
    pub async fn save(&self, wn: tauri::State<'_, Whitenoise>) -> Result<Account> {
        tracing::debug!(
            target: "whitenoise::accounts::save",
            "Beginning save transaction for pubkey: {}",
            self.pubkey.to_hex()
        );

        let mut txn = wn.database.pool.begin().await?;

        let result = sqlx::query(
            "INSERT INTO accounts (pubkey, metadata, settings, onboarding, last_used, last_synced, active)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(pubkey) DO UPDATE SET
                metadata = excluded.metadata,
                settings = excluded.settings,
                onboarding = excluded.onboarding,
                last_used = excluded.last_used,
                last_synced = excluded.last_synced,
                active = excluded.active"
        )
        .bind(self.pubkey.to_hex())
        .bind(&serde_json::to_string(&self.metadata)?)
        .bind(&serde_json::to_string(&self.settings)?)
        .bind(&serde_json::to_string(&self.onboarding)?)
        .bind(self.last_used.to_string())
        .bind(self.last_synced.to_string())
        .bind(self.active)
        .execute(&mut *txn)
        .await?;

        tracing::debug!(
            target: "whitenoise::accounts::save",
            "Query executed. Rows affected: {}",
            result.rows_affected()
        );

        txn.commit().await?;

        tracing::debug!(
            target: "whitenoise::accounts::save",
            "Transaction committed successfully for pubkey: {}",
            self.pubkey.to_hex()
        );

        Ok(self.clone())
    }

    /// Removes the account from the database
    pub async fn remove(
        &self,
        wn: tauri::State<'_, Whitenoise>,
        app_handle: tauri::AppHandle,
    ) -> Result<()> {
        let hex_pubkey = self.pubkey.to_hex();

        let mut txn = wn.database.pool.begin().await?;

        // First remove the account from the database, this will cascade to other tables
        sqlx::query("DELETE FROM accounts WHERE pubkey = ?")
            .bind(hex_pubkey.as_str())
            .execute(&mut *txn)
            .await?;

        // Get first remaining account's pubkey (if any)
        let remaining_account_pubkey =
            sqlx::query_scalar::<_, String>("SELECT pubkey FROM accounts")
                .fetch_optional(&mut *txn)
                .await?;

        tracing::debug!(
            target: "whitenoise::accounts::remove",
            "Updating active account. New active pubkey: {:?}",
            remaining_account_pubkey
        );

        // Then set the next account as the active one
        if let Some(pubkey) = remaining_account_pubkey.clone() {
            sqlx::query("UPDATE accounts SET active = TRUE WHERE pubkey = ?")
                .bind(&pubkey)
                .execute(&mut *txn)
                .await?;
        }

        txn.commit().await?;

        // If the database update succeeded, then we continue with other steps

        // Remove the old account's private key from the secrets store
        secrets_store::remove_private_key_for_pubkey(&hex_pubkey, &wn.data_dir)?;

        // Update Nostr client & Nostr MLS
        let account = Account::get_active(wn.clone()).await?;
        wn.nostr
            .set_nostr_identity(&account, wn.clone(), &app_handle)
            .await?;

        app_handle.emit("nostr_ready", ())?;

        // Then update Nostr MLS instance
        {
            let mut nostr_mls = wn.nostr_mls.lock().await;
            *nostr_mls = NostrMls::new(wn.data_dir.clone(), Some(hex_pubkey));
        }

        app_handle.emit("account_changed", ())?;
        Ok(())
    }

    // Add a validation method
    async fn validate_active_state(wn: tauri::State<'_, Whitenoise>) -> Result<()> {
        let mut txn = wn.database.pool.begin().await?;

        // Check if we have multiple active accounts
        let active_count =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM accounts WHERE active = TRUE")
                .fetch_one(&mut *txn)
                .await?;

        if active_count > 1 {
            tracing::warn!(
                target: "whitenoise::accounts",
                "Found {} active accounts, fixing...",
                active_count
            );

            // Fix the issue by keeping only the most recently used account active
            let result = sqlx::query(
                r#"
                WITH RankedAccounts AS (
                    SELECT pubkey,
                           ROW_NUMBER() OVER (ORDER BY last_used DESC) as rn
                    FROM accounts
                    WHERE active = TRUE
                )
                UPDATE accounts
                SET active = FALSE
                WHERE pubkey IN (
                    SELECT pubkey
                    FROM RankedAccounts
                    WHERE rn > 1
                )
            "#,
            )
            .execute(&mut *txn)
            .await?;

            tracing::info!(
                target: "whitenoise::accounts",
                "Fixed active accounts state. Rows affected: {}",
                result.rows_affected()
            );
        }

        txn.commit().await?;
        Ok(())
    }

    /// Stores a Nostr Wallet Connect URI for this account
    pub fn store_nwc_uri(&self, nwc_uri: &str, wn: tauri::State<'_, Whitenoise>) -> Result<()> {
        secrets_store::store_nwc_uri(&self.pubkey.to_hex(), nwc_uri, &wn.data_dir)
            .map_err(AccountError::SecretsStoreError)
    }

    /// Retrieves the Nostr Wallet Connect URI for this account
    pub fn get_nwc_uri(&self, wn: tauri::State<'_, Whitenoise>) -> Result<String> {
        secrets_store::get_nwc_uri(&self.pubkey.to_hex(), &wn.data_dir)
            .map_err(AccountError::SecretsStoreError)
    }

    /// Removes the Nostr Wallet Connect URI for this account
    pub fn remove_nwc_uri(&self, wn: tauri::State<'_, Whitenoise>) -> Result<()> {
        secrets_store::remove_nwc_uri(&self.pubkey.to_hex(), &wn.data_dir)
            .map_err(AccountError::SecretsStoreError)
    }
}
