use crate::account_manager::AccountManager;
use crate::group_manager::GroupManager;
use crate::nostr_manager::NostrManager;
use nostr_openmls::NostrMls;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::AppHandle;

pub struct Whitenoise {
    pub account_manager: AccountManager,
    pub group_manager: GroupManager,
    pub nostr: NostrManager,
    pub nostr_mls: Arc<Mutex<NostrMls>>,
    pub data_dir: PathBuf,
    pub logs_dir: PathBuf,
}

impl Whitenoise {
    pub async fn new(data_dir: PathBuf, logs_dir: PathBuf, app_handle: &AppHandle) -> Self {
        tracing::debug!(
            target: "whitenoise::whitenoise::new",
            "Creating Whitenoise instance with data_dir: {:?}",
            data_dir
        );

        let database = Arc::new(
            sled::open(data_dir.join("whitenoise.sled")).expect("Failed to open database"),
        );

        let account_manager = AccountManager::new(database.clone(), app_handle)
            .expect("Failed to create account manager");

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
            logs_dir,
        }
    }

    pub async fn delete_all_data(&self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::debug!(target: "whitenoise::delete_all_data", "Deleting all data");
        // Shutdown and reset Nostr
        // Delete nostr cache db
        self.nostr.delete_all_data().await?;
        // Drop the app database completely
        let db_path = self.data_dir.join("whitenoise.sled");
        if db_path.exists() {
            std::fs::remove_dir_all(&db_path)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        }
        // Clear all accounts data
        self.account_manager.delete_all_data()?;
        // Clear all groups data
        self.group_manager.delete_all_data()?;
        // Clear Nostr MLS data - this includes all the MLS group state and secrets.
        self.nostr_mls.lock().unwrap().delete_all_data()?;
        // Clear logs
        std::fs::remove_dir_all(&self.logs_dir)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        Ok(())
    }
}
