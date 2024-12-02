use crate::nostr_manager::{NostrManager, Result};
use nostr_sdk::prelude::*;

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
        let opts = SubscribeAutoCloseOptions::default().filter(FilterOptions::ExitOnEOSE);
        self.client.subscribe(vec![null_filter], Some(opts)).await?;


        let giftwrap_filter = Filter::new()
            .kind(Kind::GiftWrap)
            .pubkey(pubkey)
            .since(Timestamp::now());

        Ok(self.client.subscribe(vec![giftwrap_filter], None).await?)
    }

    async fn subscribe_mls_group_messages(
        &self,
        group_ids: Vec<String>,
    ) -> Result<Output<SubscriptionId>> {
        let mls_message_filter = Filter::new()
            .kind(Kind::MlsGroupMessage)
            .custom_tag(SingleLetterTag::lowercase(Alphabet::H), group_ids)
            .since(Timestamp::now());

        Ok(self
            .client
            .subscribe(vec![mls_message_filter], None)
            .await?)
    }

    pub async fn setup_subscriptions(
        &self,
        pubkey: PublicKey,
        nostr_group_ids: Vec<String>,
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
                        self.handle_event(*event).await?;
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
                    RelayPoolNotification::Authenticated { relay_url } => {
                        self.handle_authenticated(relay_url)?;
                        Ok(false)
                    }
                    // TODO: Remove this once we update to 0.37
                    #[allow(deprecated)]
                    RelayPoolNotification::RelayStatus { relay_url, status } => {
                        self.handle_relay_status(relay_url, status)?;
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
    async fn handle_event(&self, event: Event) -> Result<()> {
        match event.kind {
            Kind::GiftWrap => self.handle_giftwrap(event).await?,
            Kind::MlsGroupMessage => self.handle_mls_message(event)?,
            _ => {}
        }
        Ok(())
    }

    async fn handle_giftwrap(&self, event: Event) -> Result<()> {
        if let Ok(unwrapped) = extract_rumor(&self.client.signer().await?, &event).await {
            match unwrapped.rumor.kind {
                Kind::MlsWelcome => self.handle_invite(unwrapped.rumor)?,
                _ => {
                    // TODO: Handle other giftwrap kinds (NIP-17)
                    tracing::info!(
                        target: "whitenoise::nostr_client::subscriptions::handle_giftwrap",
                        "Received giftwrap of kind {:?}",
                        unwrapped.rumor.kind
                    );
                }
            }
        }
        Ok(())
    }

    fn handle_invite(&self, rumor: UnsignedEvent) -> Result<()> {
        // TODO: Remove the identifying info from the log
        tracing::info!(
            target: "whitenoise::nostr_client::handle_notifications",
            "Received invite: {:?}",
            rumor
        );
        Ok(())
    }

    fn handle_mls_message(&self, event: Event) -> Result<()> {
        // TODO: Remove the identifying info from the log
        tracing::info!(
            target: "whitenoise::nostr_client::handle_notifications",
            "Received MLS message: {:?}",
            event
        );

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

    fn handle_relay_status(&self, relay_url: RelayUrl, status: RelayStatus) -> Result<()> {
        tracing::debug!(
            target: "whitenoise::nostr_client::handle_notifications",
            "Relay {}: {:?}",
            relay_url,
            status
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

    fn handle_authenticated(&self, relay_url: RelayUrl) -> Result<()> {
        tracing::debug!(
            target: "whitenoise::nostr_client::handle_notifications",
            "Relay pool authenticated on {}",
            relay_url
        );
        Ok(())
    }
}
