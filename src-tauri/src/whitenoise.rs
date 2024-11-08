use crate::account_manager::AccountManager;
use crate::group_manager::GroupManager;
use crate::nostr_client::NostrClient;
use openmls_nostr::NostrMls;
use std::path::PathBuf;
use std::sync::Arc;

pub struct Whitenoise {
    // pub database: Arc<sled::Db>,
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
        Self {
            // database: database.clone(),
            account_manager: AccountManager::new(database.clone())
                .expect("Failed to create account manager"),
            group_manager: GroupManager::new(database.clone())
                .expect("Failed to create group manager"),
            nostr: NostrClient::new(data_dir.clone())
                .await
                .expect("Failed to create Nostr client"),
            nostr_mls: NostrMls::new(data_dir.clone(), None),
            data_dir,
        }
    }
}
