use crate::accounts::Accounts;
use crate::app_settings::AppSettings;
use crate::database::Database;
use crate::nostr::DEFAULT_RELAYS;
use crate::nostr_mls::NostrMls;
use crate::secrets_store::remove_private_key_for_pubkey;
use anyhow::Result;
use log::debug;
use nostr_sdk::prelude::*;
use nostrdb::{Config, Ndb};
use std::path::Path;
use std::sync::Mutex;
use tauri::State;

/// Represents the main Whitenoise application structure.
pub struct Whitenoise {
    /// The application's database for storing local data.
    pub wdb: Database,
    /// The Nostr database for efficient event storage and retrieval.
    #[allow(dead_code)]
    pub ndb: Ndb,
    /// The Nostr client for Nostr protocol operations.
    pub nostr: Client,
    /// The Nostr MLS client for Nostr MLS protocol operations.
    pub nostr_mls: NostrMls,
    /// Application settings.
    #[allow(dead_code)]
    pub settings: Mutex<AppSettings>,
    /// User accounts information.
    pub accounts: Mutex<Accounts>,
}

impl Whitenoise {
    /// Creates a new instance of the Whitenoise application.
    ///
    /// # Arguments
    ///
    /// * `app_data_dir` - A PathBuf representing the directory where application data should be stored.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a new Whitenoise instance if successful, or an error if initialization fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The Database initialization fails
    /// - The Ndb initialization fails
    /// - Loading settings or accounts from the database fails
    pub async fn new(data_dir: &Path) -> Result<Self> {
        // Set up the database
        debug!(target: "whitenoise::new", "Initializing Whitenoise with data dir: {:?}", data_dir);
        let wdb = Database::new(data_dir)?;

        // Set up settings and accounts from database
        debug!(target: "whitenoise::new", "Loading settings and accounts from database");
        let settings = AppSettings::from_database(&wdb)?;
        let accounts = Accounts::from_database(&wdb)?;

        // Set up Nostrdb
        debug!(target: "whitenoise::new", "Setting up Nostrdb");
        let mut config = Config::new();
        config.set_ingester_threads(4);
        let ndb_path = format!("{}/{}", data_dir.to_string_lossy(), "ndb");
        let ndb = Ndb::new(&ndb_path, &config)?;

        // Set up Nostr client
        debug!(target: "whitenoise::new", "Setting up Nostr client");
        let database = NdbDatabase::open(&ndb_path).expect("Failed to open database");
        let nostr = Client::builder().database(database).build();

        for relay in DEFAULT_RELAYS {
            let _ = nostr.add_relay(relay).await;
        }

        // Set up Nostr MLS client
        debug!(target: "whitenoise::new", "Setting up Nostr MLS client");
        let nostr_mls = NostrMls::new(data_dir.to_path_buf());

        nostr.connect().await;

        Ok(Self {
            wdb,
            ndb,
            nostr,
            nostr_mls,
            settings: Mutex::new(settings),
            accounts: Mutex::new(accounts),
        })
    }

    /// Updates the Nostr signer with the provided keys.
    ///
    /// # Arguments
    ///
    /// * `keys` - The keys to set as the Nostr signer.
    ///
    /// # Returns
    ///
    pub async fn update_nostr_signer_with_keys(&self, keys: Keys) -> Result<()> {
        let signer = NostrSigner::from(keys);
        self.nostr.set_signer(Some(signer)).await;
        Ok(())
    }

    /// Clears all data associated with the application.
    ///
    /// This function performs the following actions:
    /// 1. Clears the local app database
    /// 2. Clears the Nostr MLS store
    /// 3. Clears the accounts data
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - It fails to clear the app database
    /// - It fails to clear the Nostr MLS store
    /// - It fails to acquire the lock on the accounts or clear the accounts data
    pub async fn clear_all_data(&self) -> Result<()> {
        debug!(target: "whitenoise::clear_all_data", "Clearing all data");
        // Clear the local app database
        self.wdb.delete_data().expect("Couldn't clear app database");
        // Clear the Nostr MLS store
        self.nostr_mls
            .delete_data()
            .expect("Couldn't clear Nostr MLS store");
        // Clear all nostr stuff
        self.nostr.unsubscribe_all().await;
        self.nostr.remove_all_relays().await?;
        // Clear the private keys
        let accounts = self.accounts.lock().unwrap().clone().accounts;
        if let Some(accounts) = accounts {
            for pubkey in accounts.keys() {
                remove_private_key_for_pubkey(pubkey).expect("Couldn't remove private key");
            }
        }
        // Clear the accounts data
        self.accounts
            .lock()
            .unwrap()
            .delete_data(&self.wdb)
            .expect("Couldn't clear accounts");
        *self.accounts.lock().unwrap() = Accounts::default();
        // Clear the app settings
        self.settings
            .lock()
            .unwrap()
            .delete_data(&self.wdb)
            .expect("Couldn't clear app settings");
        *self.settings.lock().unwrap() = AppSettings::default();
        Ok(())
    }
}

#[tauri::command]
pub async fn delete_data(wn: State<'_, Whitenoise>) -> Result<(), String> {
    wn.clear_all_data().await.map_err(|e| e.to_string())
}
