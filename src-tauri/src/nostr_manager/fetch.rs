use crate::nostr_manager::{NostrManager, NostrManagerError, Result};
use nostr_sdk::prelude::*;

impl NostrManager {
    pub async fn fetch_user_metadata(&self, pubkey: PublicKey) -> Result<Option<Metadata>> {
        match self
            .client
            .fetch_metadata(pubkey, Some(self.timeout()?))
            .await
        {
            Ok(metadata) => Ok(Some(metadata)),
            Err(nostr_sdk::client::Error::MetadataNotFound) => Ok(None),
            Err(e) => Err(NostrManagerError::from(e)),
        }
    }

    pub async fn fetch_user_relays(&self, pubkey: PublicKey) -> Result<Vec<String>> {
        let filter = Filter::new().author(pubkey).kind(Kind::RelayList).limit(1);

        let events = self
            .client
            .fetch_events(vec![filter], Some(self.timeout()?))
            .await
            .map_err(NostrManagerError::from)?;

        Ok(Self::relay_urls_from_events(events))
    }

    pub async fn fetch_user_inbox_relays(&self, pubkey: PublicKey) -> Result<Vec<String>> {
        let filter = Filter::new()
            .author(pubkey)
            .kind(Kind::InboxRelays)
            .limit(1);
        let events = self
            .client
            .fetch_events(vec![filter], Some(self.timeout()?))
            .await
            .map_err(NostrManagerError::from)?;

        Ok(Self::relay_urls_from_events(events))
    }

    pub async fn fetch_user_key_package_relays(&self, pubkey: PublicKey) -> Result<Vec<String>> {
        let filter = Filter::new()
            .author(pubkey)
            .kind(Kind::MlsKeyPackageRelays)
            .limit(1);
        let events = self
            .client
            .fetch_events(vec![filter], Some(self.timeout()?))
            .await
            .map_err(NostrManagerError::from)?;

        Ok(Self::relay_urls_from_events(events))
    }

    pub async fn fetch_user_key_packages(&self, pubkey: PublicKey) -> Result<Events> {
        let filter = Filter::new()
            .author(pubkey)
            .kind(Kind::MlsKeyPackage);
        let events = self
            .client
            .fetch_events(vec![filter], Some(self.timeout()?))
            .await
            .map_err(NostrManagerError::from)?;

        Ok(events)
    }

    pub async fn fetch_contacts(&self) -> Result<Vec<Event>> {
        tracing::debug!(
            target: "whitenoise::nostr_client::fetch_contacts",
            "Fetching contacts for: {:?}",
            self.client.signer().await?.get_public_key().await.unwrap().to_hex()
        );
        let contacts_pubkeys = self
            .client
            .get_contact_list_public_keys(Some(self.timeout()?))
            .await?;

        // If there are no contacts, return an empty vector
        if contacts_pubkeys.is_empty() {
            return Ok(vec![]);
        }

        let filter = Filter::new().kind(Kind::Metadata).authors(contacts_pubkeys);
        let database_contacts = self.client.database().query(vec![filter.clone()]).await?;
        let fetched_contacts = self
            .client
            .fetch_events(vec![filter], Some(self.timeout()?))
            .await?;

        let contacts = database_contacts.merge(fetched_contacts);
        Ok(contacts.into_iter().collect())
    }

    #[allow(dead_code)]
    pub async fn fetch_user_welcomes(&self, pubkey: PublicKey) -> Result<Vec<UnsignedEvent>> {
        let gw_events = self.fetch_user_giftwrapped_events(pubkey).await?;
        let invites = self.extract_invite_events(gw_events).await;
        Ok(invites)
    }

    #[allow(dead_code)]
    async fn fetch_user_giftwrapped_events(&self, pubkey: PublicKey) -> Result<Vec<Event>> {
        let filter = Filter::new().kind(Kind::GiftWrap).pubkeys(vec![pubkey]);

        let stored_events = self.client.database().query(vec![filter.clone()]).await?;
        let fetched_events = self
            .client
            .fetch_events(
                vec![Filter::new().kind(Kind::GiftWrap).pubkeys(vec![pubkey])],
                Some(self.timeout()?),
            )
            .await?;

        let events = stored_events.merge(fetched_events);
        Ok(events.into_iter().collect())
    }
}
