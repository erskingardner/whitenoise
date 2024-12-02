use crate::types::NostrEncryptionMethod;
use nostr_sdk::prelude::*;
use nostr_sdk::NostrLMDB;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use thiserror::Error;

pub mod fetch;
pub mod query;
pub mod subscriptions;
pub mod sync;

#[derive(Error, Debug)]
pub enum NostrManagerError {
    #[error("Client Error: {0}")]
    Client(#[from] nostr_sdk::client::Error),
    #[error("Metadata Error: {0}")]
    Metadata(#[from] nostr_sdk::types::metadata::Error),
    #[error("Database Error: {0}")]
    Database(#[from] DatabaseError),
    #[error("Signer Error: {0}")]
    Signer(#[from] nostr_sdk::signer::SignerError),
    #[error("Failed to acquire lock: {0}")]
    LockError(String),
}

#[derive(Debug, Clone)]
pub struct NostrManagerSettings {
    pub timeout: Duration,
    pub relays: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct NostrManager {
    pub client: Client,
    pub settings: Arc<Mutex<NostrManagerSettings>>,
}

impl Default for NostrManagerSettings {
    fn default() -> Self {
        let mut relays = vec![];
        if cfg!(dev) {
            relays.push("ws://localhost:8080".to_string());
            // relays.push("wss://purplepag.es".to_string());
            // relays.push("wss://relay.nostr.net".to_string());
        } else {
            relays.push("wss://relay.damus.io".to_string());
            relays.push("wss://relay.primal.net".to_string());
            relays.push("wss://nostr.oxtr.dev".to_string());
        }

        Self {
            timeout: Duration::from_secs(10),
            relays,
        }
    }
}
pub type Result<T> = std::result::Result<T, NostrManagerError>;

impl NostrManager {
    pub async fn new(db_path: PathBuf) -> Result<Self> {
        let full_path = db_path.join("nostr_lmdb");
        let db = NostrLMDB::open(full_path).expect("Failed to open Nostr database");
        let opts = Options::default();
        let client = Client::builder().database(db).opts(opts).build();

        let settings = NostrManagerSettings::default();

        Ok(Self {
            client,
            settings: Arc::new(Mutex::new(settings)),
        })
    }

    pub fn timeout(&self) -> Result<Duration> {
        Ok(self
            .settings
            .lock()
            .map_err(|e| NostrManagerError::LockError(e.to_string()))?
            .timeout)
    }

    pub fn relays(&self) -> Result<Vec<String>> {
        Ok(self
            .settings
            .lock()
            .map_err(|e| NostrManagerError::LockError(e.to_string()))?
            .relays
            .clone())
    }

    /// Extracts welcome events from a list of giftwrapped events.
    ///
    /// This function processes a list of giftwrapped events and extracts the welcome events
    /// (events with Kind::MlsWelcome) from them.
    ///
    /// # Arguments
    ///
    /// * `keys` - A reference to the Keys struct containing the necessary keys for decryption.
    /// * `gw_events` - A vector of giftwrapped Event objects to process.
    ///
    /// # Returns
    ///
    /// A vector of UnsignedEvent objects representing the extracted welcome events.
    async fn extract_invite_events(&self, gw_events: Vec<Event>) -> Vec<UnsignedEvent> {
        let mut invite_events: Vec<UnsignedEvent> = Vec::new();

        for event in gw_events {
            if let Ok(unwrapped) = extract_rumor(&self.client.signer().await.unwrap(), &event).await
            {
                if unwrapped.rumor.kind == Kind::MlsWelcome {
                    invite_events.push(unwrapped.rumor);
                }
            }
        }

        invite_events
    }

    pub async fn update_nostr_identity(&self, keys: Keys) -> Result<()> {
        tracing::debug!(
            target: "whitenoise::nostr_client::update_nostr_identity",
            "Updating Nostr identity"
        );

        // Reset the client
        self.client.reset().await.map_err(NostrManagerError::from)?;

        // Set the new signer
        self.client.set_signer(keys.clone()).await;

        // Add the default relays
        for relay in self.relays()? {
            self.client.add_relay(relay).await?;
        }

        // Connect to the default relays
        self.client.connect().await;

        // We only want to connect to user relays in release mode
        if !cfg!(dev) {
            // Add the new user's relays
            // TODO: We should query first and only fetch if we don't have them
            let relays = self.fetch_user_relays(keys.public_key()).await?;
            for relay in relays.iter() {
                self.client.add_relay(relay).await?;
                self.client.connect_relay(relay).await?;
                tracing::debug!(
                    target: "whitenoise::nostr_client::update_nostr_identity",
                    "Connected to user relay: {}",
                    relay
                );
            }

            // Add the new user's inbox relays
            // TODO: We should query first and only fetch if we don't have them
            let inbox_relays = self.fetch_user_inbox_relays(keys.public_key()).await?;
            for relay in inbox_relays.iter() {
                self.client.add_read_relay(relay).await?;
                self.client.connect_relay(relay).await?;
                tracing::debug!(
                    target: "whitenoise::nostr_client::update_nostr_identity",
                    "Connected to user inbox relay: {}",
                    relay
                );
            }

            // Add the new user's key package relays
            // TODO: We should query first and only fetch if we don't have them
            let key_package_relays = self
                .fetch_user_key_package_relays(keys.public_key())
                .await?;
            for relay in key_package_relays.iter() {
                self.client.add_relay(relay).await?;
                self.client.connect_relay(relay).await?;
                tracing::debug!(
                    target: "whitenoise::nostr_client::update_nostr_identity",
                    "Connected to user key package relay: {}",
                    relay
                );
            }
        }

        tracing::debug!(
            target: "whitenoise::nostr_client::update_nostr_identity",
            "Connected to relays: {:?}",
            self.client
                .relays()
                .await
                .keys()
                .map(|url| url.to_string())
                .collect::<Vec<_>>()
        );

        tracing::debug!(
            target: "whitenoise::nostr_client::update_nostr_identity",
            "Nostr identity updated and connected to relays"
        );

        Ok(())
    }

    pub async fn encrypt_content(
        &self,
        content: String,
        pubkey: String,
        method: NostrEncryptionMethod,
    ) -> Result<String> {
        let recipient_pubkey = PublicKey::from_hex(&pubkey).unwrap();
        let signer = self.client.signer().await.unwrap();
        match method {
            NostrEncryptionMethod::Nip04 => {
                let encrypted = signer
                    .nip04_encrypt(&recipient_pubkey, &content)
                    .await
                    .unwrap();
                Ok(encrypted)
            }
            NostrEncryptionMethod::Nip44 => {
                let encrypted = signer
                    .nip44_encrypt(&recipient_pubkey, &content)
                    .await
                    .unwrap();
                Ok(encrypted)
            }
        }
    }

    pub async fn decrypt_content(
        &self,
        content: String,
        pubkey: String,
        method: NostrEncryptionMethod,
    ) -> Result<String> {
        let author_pubkey = PublicKey::from_hex(&pubkey).unwrap();
        let signer = self.client.signer().await.unwrap();
        match method {
            NostrEncryptionMethod::Nip04 => {
                let decrypted = signer
                    .nip04_decrypt(&author_pubkey, &content)
                    .await
                    .unwrap();
                Ok(decrypted)
            }
            NostrEncryptionMethod::Nip44 => {
                let decrypted = signer
                    .nip44_decrypt(&author_pubkey, &content)
                    .await
                    .unwrap();
                Ok(decrypted)
            }
        }
    }

    fn relay_urls_from_events(events: Events) -> Vec<String> {
        events
            .into_iter()
            .flat_map(|e| e.tags)
            .filter(|tag| tag.kind() == TagKind::Relay)
            .map_while(|tag| tag.content().map(|c| c.to_string()))
            .collect()
    }

    pub async fn delete_all_data(&self) -> Result<()> {
        tracing::debug!(
            target: "whitenoise::nostr_manager::delete_all_data",
            "Deleting Nostr data"
        );
        self.client.reset().await.map_err(NostrManagerError::from)?;
        self.client
            .database()
            .wipe()
            .await
            .map_err(NostrManagerError::from)?;
        Ok(())
    }
}
