use crate::types::NostrEncryptionMethod;
use nostr_sdk::prelude::*;
use nostr_sdk::NostrLMDB;
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NostrClientError {
    #[error("Client Error: {0}")]
    Client(#[from] nostr_sdk::client::Error),
    #[error("Metadata Error: {0}")]
    Metadata(#[from] nostr_sdk::types::metadata::Error),
    #[error("Database Error: {0}")]
    Database(#[from] nostr_sdk::database::DatabaseError),
}

#[derive(Debug, Clone)]
pub struct NostrClientSettings {
    timeout: Duration,
    relays: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct NostrClient {
    pub client: Client,
    pub settings: NostrClientSettings,
}

impl Default for NostrClientSettings {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            relays: vec![
                "wss://relay.damus.io".to_string(),
                "wss://relay.primal.net".to_string(),
                "wss://nos.lol".to_string(),
                "wss://purplepag.es".to_string(),
                "ws://localhost:8080".to_string(),
            ],
        }
    }
}

pub type Result<T> = std::result::Result<T, NostrClientError>;

impl NostrClient {
    pub async fn new(db_path: PathBuf) -> Result<Self> {
        let full_path = db_path.join("nostr_lmdb");
        let db = NostrLMDB::open(full_path).expect("Failed to open Nostr database");
        let opts = Options::new();
        let client = Client::builder().database(db).opts(opts).build();

        let settings = NostrClientSettings::default();

        Ok(Self { client, settings })
    }

    pub async fn fetch_user_metadata(&self, pubkey: PublicKey) -> Result<Metadata> {
        let metadata = self
            .client
            .fetch_metadata(pubkey, Some(self.settings.timeout))
            .await?;
        tracing::debug!(
            target: "whitenoise::nostr_client::fetch_user_metadata",
            "Fetched metadata: {:?}",
            metadata
        );
        Ok(metadata)
    }

    pub fn timeout(&self) -> Duration {
        self.settings.timeout
    }

    pub async fn fetch_user_relays(&self, pubkey: PublicKey) -> Result<Vec<String>> {
        let filter = Filter::new().author(pubkey).kind(Kind::RelayList).limit(1);

        let events = self
            .client
            .fetch_events(vec![filter], Some(self.settings.timeout))
            .await
            .map_err(NostrClientError::from)?;

        Ok(Self::relay_urls_from_events(events))
    }

    pub async fn fetch_user_inbox_relays(&self, pubkey: PublicKey) -> Result<Vec<String>> {
        let filter = Filter::new()
            .author(pubkey)
            .kind(Kind::Custom(10050))
            .limit(1);
        let events = self
            .client
            .fetch_events(vec![filter], Some(self.settings.timeout))
            .await
            .map_err(NostrClientError::from)?;

        Ok(Self::relay_urls_from_events(events))
    }

    pub async fn fetch_user_key_package_relays(&self, pubkey: PublicKey) -> Result<Vec<String>> {
        let filter = Filter::new()
            .author(pubkey)
            .kind(Kind::Custom(10051))
            .limit(1);
        let events = self
            .client
            .fetch_events(vec![filter], Some(self.settings.timeout))
            .await
            .map_err(NostrClientError::from)?;

        Ok(Self::relay_urls_from_events(events))
    }

    pub async fn fetch_user_key_packages(&self, pubkey: PublicKey) -> Result<Events> {
        let filter = Filter::new()
            .author(pubkey)
            .kind(Kind::MlsKeyPackage)
            .limit(1);
        let events = self
            .client
            .fetch_events(vec![filter], Some(self.settings.timeout))
            .await
            .map_err(NostrClientError::from)?;

        Ok(events)
    }

    pub async fn fetch_user_welcomes(&self, pubkey: PublicKey) -> Result<Vec<UnsignedEvent>> {
        let gw_events = self.fetch_user_giftwrapped_events(pubkey).await?;
        let invites = self.extract_invite_events(gw_events).await;
        Ok(invites)
    }

    /// Fetches giftwrapped events for a specific public key from Nostr relays.
    ///
    /// This function retrieves all giftwrapped events (Kind::GiftWrap) that are addressed to the
    /// specified public key. It uses the Nostr client to fetch events from both local storage and
    /// connected relays.
    ///
    /// # Arguments
    ///
    /// * `wn` - A Tauri State containing a Whitenoise instance, which provides access to Nostr functionality.
    /// * `pubkey` - A String representing the public key for which to fetch giftwrapped events.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<Event>>` - A Result that is Ok with a vector of Event objects if the fetch is successful,
    ///   or an Err with a descriptive error message if the fetch fails.
    async fn fetch_user_giftwrapped_events(&self, pubkey: PublicKey) -> Result<Vec<Event>> {
        let filter = Filter::new().kind(Kind::GiftWrap).pubkeys(vec![pubkey]);

        let stored_events = self.client.database().query(vec![filter.clone()]).await?;
        let fetched_events = self
            .client
            .fetch_events(
                vec![Filter::new().kind(Kind::GiftWrap).pubkeys(vec![pubkey])],
                Some(self.settings.timeout),
            )
            .await?;

        let events = stored_events.merge(fetched_events);
        Ok(events.into_iter().collect())
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
        self.client.reset().await.map_err(NostrClientError::from)?;

        // Set the new signer
        self.client.set_signer(keys.clone()).await;

        // Add the default relays
        for relay in self.settings.relays.iter() {
            self.client.add_relay(relay).await?;
        }

        // Connect to the default relays
        self.client.connect().await;

        // Add the new user's inbox relays
        let inbox_relays = self.fetch_user_inbox_relays(keys.public_key()).await?;
        for relay in inbox_relays.iter() {
            self.client.add_read_relay(relay).await?;
            self.client.connect_relay(relay).await?;
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
        // Setup subscriptions for the new user
        let client_clone = self.clone();
        tauri::async_runtime::spawn(async move {
            client_clone.setup_subscriptions(keys).await?;
            Ok::<(), NostrClientError>(())
        });

        tracing::debug!(
            target: "whitenoise::nostr_client::update_nostr_identity",
            "Nostr identity fully updated"
        );

        Ok(())
    }

    async fn setup_subscriptions(&self, keys: Keys) -> Result<()> {
        let contacts_filter = Filter::new()
            .kind(Kind::ContactList)
            .author(keys.public_key())
            .since(Timestamp::now());
        tracing::debug!(
            target: "whitenoise::nostr_client::setup_subscriptions",
            "contacts_filter: {:?}",
            contacts_filter
        );
        let _contacts_sub_id = self.client.subscribe(vec![contacts_filter], None).await?;

        let contact_list_pubkeys = self
            .client
            .get_contact_list_public_keys(Some(self.settings.timeout))
            .await?;

        let contact_metadata_filter = Filter::new()
            .kind(Kind::Metadata)
            .authors(contact_list_pubkeys)
            .since(Timestamp::now());
        tracing::debug!(
            target: "whitenoise::nostr_client::setup_subscriptions",
            "contact_metadata_filter: {:?}",
            contact_metadata_filter
        );
        let _contact_metadata_sub_id = self
            .client
            .subscribe(vec![contact_metadata_filter], None)
            .await?;

        let giftwrap_filter = Filter::new()
            .kind(Kind::GiftWrap)
            .pubkey(keys.public_key())
            .since(Timestamp::now());
        tracing::debug!(
            target: "whitenoise::nostr_client::setup_subscriptions",
            "giftwrap_filter: {:?}",
            giftwrap_filter
        );
        let _giftwrap_sub_id = self.client.subscribe(vec![giftwrap_filter], None).await?;

        if let Err(e) = self
            .client
            .handle_notifications(|notification| async {
                match notification {
                    RelayPoolNotification::Event { event, .. } => {
                        tracing::trace!(
                            target: "whitenoise::nostr_client::handle_notifications",
                            "Received event: {:?}",
                            event
                        );
                        Ok(false)
                    }
                    RelayPoolNotification::Message { relay_url, message } => {
                        tracing::trace!(
                            target: "whitenoise::nostr_client::handle_notifications",
                            "Received message from {}: {:?}",
                            relay_url,
                            message
                        );
                        Ok(false)
                    }
                    RelayPoolNotification::RelayStatus { relay_url, status } => {
                        tracing::trace!(
                            target: "whitenoise::nostr_client::handle_notifications",
                            "Relay {}: {:?}",
                            relay_url,
                            status
                        );
                        Ok(false)
                    }
                    RelayPoolNotification::Shutdown => {
                        tracing::debug!(
                            target: "whitenoise::nostr_client::handle_notifications",
                            "Relay pool shutdown"
                        );
                        Ok(true)
                    }
                    RelayPoolNotification::Authenticated { relay_url } => {
                        tracing::trace!(
                            target: "whitenoise::nostr_client::handle_notifications",
                            "Relay pool authenticated on {}",
                            relay_url
                        );
                        Ok(false)
                    }
                }
            })
            .await
        {
            tracing::error!(
                target: "whitenoise::nostr_client::handle_notifications",
                "Notification handler error: {:?}",
                e
            );
        }

        Ok(())
    }

    pub async fn fetch_contacts(&self) -> Result<Vec<Event>> {
        tracing::debug!(
            target: "whitenoise::nostr_client::fetch_contacts",
            "Fetching contacts for: {:?}",
            self.client.signer().await?.get_public_key().await.unwrap().to_hex()
        );
        let contacts_pubkeys = self
            .client
            .get_contact_list_public_keys(Some(self.settings.timeout))
            .await?;

        // If there are no contacts, return an empty vector
        if contacts_pubkeys.is_empty() {
            return Ok(vec![]);
        }

        let filter = Filter::new().kind(Kind::Metadata).authors(contacts_pubkeys);
        let database_contacts = self.client.database().query(vec![filter.clone()]).await?;
        let fetched_contacts = self
            .client
            .fetch_events(vec![filter], Some(self.settings.timeout))
            .await?;

        let contacts = database_contacts.merge(fetched_contacts);
        Ok(contacts.into_iter().collect())
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_keys() -> Keys {
        Keys::generate()
    }

    #[tokio::test]
    async fn test_nostr_client_creation() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("nostr.db");

        let client = NostrClient::new(db_path).await;
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_update_nostr_identity() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("nostr.db");

        let client = NostrClient::new(db_path).await.unwrap();
        let keys = create_test_keys();

        let result = client.update_nostr_identity(keys).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fetch_user_metadata() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("nostr.db");

        let client = NostrClient::new(db_path).await.unwrap();
        let keys = create_test_keys();

        let result = client.fetch_user_metadata(keys.public_key()).await;
        // Note: This might fail in practice as the test key won't have metadata
        // on real relays, but it tests the API call
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fetch_user_relays() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("nostr.db");

        let client = NostrClient::new(db_path).await.unwrap();
        let keys = create_test_keys();

        let result = client.fetch_user_relays(keys.public_key()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty()); // New key should have no relays
    }

    #[tokio::test]
    async fn test_relay_urls_from_events() {
        let events = Events::new(&[Filter::new()]);
        let urls = NostrClient::relay_urls_from_events(events);
        assert!(urls.is_empty());
    }

    #[test]
    fn test_nostr_client_settings_default() {
        let settings = NostrClientSettings::default();
        assert_eq!(settings.timeout, Duration::from_secs(3));
        assert_eq!(settings.relays.len(), 6); // Verify number of default relays
    }
}
