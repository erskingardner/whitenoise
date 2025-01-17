//! Subscription functions for NostrManager
//! This mostly handles subscribing and processing events as they come in while the user is active.

use crate::nostr_manager::{NostrManager, Result};
use nostr_sdk::prelude::*;
use tauri::Emitter;

const MLS_MESSAGES_SUB: &str = "mls_messages";

impl NostrManager {
    async fn subscribe_contact_list(&self, pubkey: PublicKey) -> Result<Output<SubscriptionId>> {
        let contacts_filter = Filter::new()
            .kind(Kind::ContactList)
            .author(pubkey)
            .since(Timestamp::now());

        Ok(self.client.subscribe(vec![contacts_filter], None).await?)
    }

    async fn subscribe_contacts_metadata(&self) -> Result<Output<SubscriptionId>> {
        let contact_list_pubkeys = self
            .client
            .get_contact_list_public_keys(Some(self.timeout()?))
            .await?;

        let contact_metadata_filter = Filter::new()
            .kind(Kind::Metadata)
            .authors(contact_list_pubkeys)
            .since(Timestamp::now());

        Ok(self
            .client
            .subscribe(vec![contact_metadata_filter], None)
            .await?)
    }

    async fn subscribe_metadata(&self, pubkey: PublicKey) -> Result<Output<SubscriptionId>> {
        let metadata_filter = Filter::new()
            .kind(Kind::Metadata)
            .author(pubkey)
            .since(Timestamp::now());

        Ok(self.client.subscribe(vec![metadata_filter], None).await?)
    }

    async fn subscribe_relay_list(&self, pubkey: PublicKey) -> Result<Output<SubscriptionId>> {
        let relay_list_filter = Filter::new()
            .kind(Kind::RelayList)
            .author(pubkey)
            .since(Timestamp::now());

        Ok(self.client.subscribe(vec![relay_list_filter], None).await?)
    }

    async fn subscribe_inbox_relay_list(
        &self,
        pubkey: PublicKey,
    ) -> Result<Output<SubscriptionId>> {
        let inbox_relay_list_filter = Filter::new()
            .kind(Kind::InboxRelays)
            .author(pubkey)
            .since(Timestamp::now());

        Ok(self
            .client
            .subscribe(vec![inbox_relay_list_filter], None)
            .await?)
    }

    async fn subscribe_giftwraps(&self, pubkey: PublicKey) -> Result<Output<SubscriptionId>> {
        // This is a hack to get the client to do the initial authenticate on relays that require it.
        // https://github.com/rust-nostr/nostr/issues/509
        let null_filter = Filter::new().kind(Kind::GiftWrap).pubkey(pubkey).limit(0);
        self.client
            .fetch_events(vec![null_filter], Some(self.timeout()?))
            .await?;

        let giftwrap_filter = Filter::new()
            .kind(Kind::GiftWrap)
            .pubkey(pubkey)
            .since(Timestamp::now());

        Ok(self.client.subscribe(vec![giftwrap_filter], None).await?)
    }

    pub async fn subscribe_mls_group_messages(&self, group_ids: Vec<String>) -> Result<Output<()>> {
        let sub_id = SubscriptionId::new(MLS_MESSAGES_SUB);
        let mls_message_filter = Filter::new()
            .kind(Kind::MlsGroupMessage)
            .custom_tag(SingleLetterTag::lowercase(Alphabet::H), group_ids)
            .since(Timestamp::now());

        Ok(self
            .client
            .subscribe_with_id(sub_id, vec![mls_message_filter], None)
            .await?)
    }

    pub async fn setup_subscriptions(
        &self,
        pubkey: PublicKey,
        nostr_group_ids: Vec<String>,
        app_handle: tauri::AppHandle,
    ) -> Result<()> {
        self.subscribe_contact_list(pubkey).await?;
        self.subscribe_contacts_metadata().await?;
        self.subscribe_metadata(pubkey).await?;
        self.subscribe_relay_list(pubkey).await?;
        self.subscribe_inbox_relay_list(pubkey).await?;
        self.subscribe_giftwraps(pubkey).await?;

        if !nostr_group_ids.is_empty() {
            self.subscribe_mls_group_messages(nostr_group_ids).await?;
        }

        if let Err(e) = self
            .client
            .handle_notifications(|notification| async {
                match notification {
                    RelayPoolNotification::Event { event, .. } => {
                        self.handle_event(*event, app_handle.clone()).await?;
                        Ok(false)
                    }
                    RelayPoolNotification::Message { relay_url, message } => {
                        self.handle_message(relay_url, message)?;
                        Ok(false)
                    }
                    RelayPoolNotification::Shutdown => {
                        self.handle_shutdown()?;
                        Ok(true)
                    }
                    _ => {
                        tracing::debug!(
                            target: "whitenoise::nostr_client::handle_notifications",
                            "Received unknown notification: {:?}",
                            notification
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

    // Handle events
    async fn handle_event(&self, event: Event, app_handle: tauri::AppHandle) -> Result<()> {
        match event.kind {
            Kind::GiftWrap => self.handle_giftwrap(event).await?,
            Kind::MlsGroupMessage => self.handle_mls_message(event, app_handle)?,
            _ => {}
        }
        Ok(())
    }

    async fn handle_giftwrap(&self, event: Event) -> Result<()> {
        if let Ok(unwrapped) = extract_rumor(&self.client.signer().await?, &event).await {
            match unwrapped.rumor.kind {
                Kind::MlsWelcome => self.handle_invite(unwrapped.rumor)?,
                Kind::PrivateDirectMessage => {
                    self.handle_private_direct_message(unwrapped.rumor)?
                }
                _ => {
                    tracing::info!(
                        target: "whitenoise::nostr_client::subscriptions::handle_giftwrap",
                        "Received unhandled giftwrap of kind {:?}",
                        unwrapped.rumor.kind
                    );
                }
            }
        }
        Ok(())
    }

    fn handle_invite(&self, rumor: UnsignedEvent) -> Result<()> {
        // TODO: We would like to be able to store these invites in the cache but since they're not signed we can't do that yet.
        // TODO: Remove the identifying info from the log
        tracing::info!(
            target: "whitenoise::nostr_client::handle_notifications",
            "Received invite: {:?}",
            rumor
        );
        Ok(())
    }

    fn handle_private_direct_message(&self, rumor: UnsignedEvent) -> Result<()> {
        // TODO: We would like to be able to store these invites in the cache but since they're not signed we can't do that yet.
        // TODO: Remove the identifying info from the log
        tracing::info!(
            target: "whitenoise::nostr_client::handle_notifications",
            "Received private direct message: {:?}",
            rumor
        );
        Ok(())
    }

    fn handle_mls_message(&self, event: Event, app_handle: tauri::AppHandle) -> Result<()> {
        // TODO: Remove the identifying info from the log
        tracing::info!(
            target: "whitenoise::nostr_client::handle_notifications",
            "Received MLS message: {:?}",
            event
        );

        // TODO: Process the message into an unsigned event and add to the right group transcript
        app_handle
            .emit("mls_message_received", event.clone())
            .expect("Couldn't emit mls_message_received event");
        Ok(())
    }

    // Handle other types of notifications

    fn handle_message(&self, relay_url: RelayUrl, message: RelayMessage) -> Result<()> {
        tracing::debug!(
            target: "whitenoise::nostr_client::handle_notifications",
            "Received message from {}: {:?}",
            relay_url,
            message
        );
        Ok(())
    }

    fn handle_shutdown(&self) -> Result<()> {
        tracing::debug!(
            target: "whitenoise::nostr_client::handle_notifications",
            "Relay pool shutdown"
        );
        Ok(())
    }
}
