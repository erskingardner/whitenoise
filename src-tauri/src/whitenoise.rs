use crate::account_manager::AccountManager;
use crate::group_manager::GroupManager;
use crate::nostr_manager::NostrManager;
use nostr_openmls::NostrMls;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct Whitenoise {
    pub account_manager: AccountManager,
    pub group_manager: GroupManager,
    pub nostr: NostrManager,
    pub nostr_mls: Arc<Mutex<NostrMls>>,
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

        let active_account = account_manager.get_active_account().ok();

        let active_account_pubkey = match active_account.clone() {
            Some(account) => account.pubkey,
            None => String::from(""),
        };

        let group_manager = GroupManager::new(database.clone(), active_account_pubkey)
            .expect("Failed to create group manager");

        Self {
            account_manager,
            group_manager,
            nostr: NostrManager::new(data_dir.clone())
                .await
                .expect("Failed to create Nostr manager"),
            nostr_mls: Arc::new(Mutex::new(NostrMls::new(
                data_dir.clone(),
                active_account.map(|a| a.pubkey),
            ))),
            data_dir,
        }
    }

    // pub async fn clear_all_data(&self) -> Result<(), Error> {
    //     tracing::debug!(target: "whitenoise::clear_all_data", "Clearing all data");
    //     // Shutdown and reset Nostr
    //     // Delete nostr cache db
    //     // Clear database trees data
    //     // - Accounts
    //     // - Groups
    //     // - Invites
    //     // Clear the all accounts data
    //     // Clear all groups data
    //     // Clear all Nostr MLS data - clear the sled db
    //     Ok(())
    // }
}
