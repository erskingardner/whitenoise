use crate::account_manager::AccountManager;
use crate::group_manager::{GroupManager, GroupManagerError};
use crate::nostr_client::NostrClient;
use openmls_nostr::NostrMls;
use std::path::PathBuf;
use std::sync::Arc;

pub struct Whitenoise {
    pub account_manager: AccountManager,
    pub group_manager: GroupManager,
    pub nostr: NostrClient,
    pub nostr_mls: NostrMls,
    pub data_dir: PathBuf,
}

impl Whitenoise {
    pub async fn new(data_dir: PathBuf) -> Self {
        tracing::debug!(target: "whitenoise::whitenoise::new", "Creating Whitenoise instance with data_dir: {:?}", data_dir);
        let database = Arc::new(
            sled::open(data_dir.join("whitenoise.sled")).expect("Failed to open database"),
        );

        let account_manager =
            AccountManager::new(database.clone()).expect("Failed to create account manager");

        let active_account_pubkey = match account_manager.get_active_account() {
            Ok(account) => account.pubkey,
            Err(_) => String::from(""),
        };

        let group_tree_name = format!("{}{}", "groups", active_account_pubkey);
        let invite_tree_name = format!("{}{}", "invites", active_account_pubkey);

        let group_manager = GroupManager::new(database.clone(), group_tree_name, invite_tree_name)
            .expect("Failed to create group manager");

        Self {
            account_manager,
            group_manager,
            nostr: NostrClient::new(data_dir.clone())
                .await
                .expect("Failed to create Nostr client"),
            nostr_mls: NostrMls::new(data_dir.clone(), None),
            data_dir,
        }
    }

    /// Persists the current state of the GroupManager to disk
    ///
    /// This function gets the active account's public key and uses it to construct the
    /// tree names for storing group and invite data. It then calls persist_state() on
    /// the GroupManager with these tree names.
    ///
    /// We do this scoping to ensure that we're not leaking groups or invites between different accounts.
    ///
    /// # Errors
    ///
    /// Returns a GroupManagerError if:
    /// - There is no active account
    /// - The GroupManager fails to persist its state
    pub fn persist_group_manager_state(&self) -> Result<(), GroupManagerError> {
        let active_account_pubkey = match self.account_manager.get_active_account() {
            Ok(account) => account.pubkey,
            Err(e) => {
                return Err(GroupManagerError::DatabaseError(format!(
                    "No active account: {}",
                    e
                )));
            }
        };

        let group_tree_name = format!("{}{}", "groups", active_account_pubkey);
        let invite_tree_name = format!("{}{}", "invites", active_account_pubkey);

        self.group_manager
            .persist_state(group_tree_name, invite_tree_name)
    }
}
