use crate::nostr_manager::{NostrManager, NostrManagerError, Result};
use nostr_sdk::prelude::*;

impl NostrManager {
    pub async fn query_user_metadata(&self, pubkey: PublicKey) -> Result<Metadata> {
        let events = self
            .client
            .database()
            .query(vec![Filter::new()
                .kind(Kind::Metadata)
                .authors(vec![pubkey])
                .limit(1)])
            .await?;

        let metadata = if let Some(event) = events.first() {
            Metadata::from_json(&event.content).map_err(NostrManagerError::Metadata)?
        } else {
            Metadata::default()
        };
        Ok(metadata)
    }

    pub async fn query_user_relays(&self, pubkey: PublicKey) -> Result<Vec<String>> {
        let filter = Filter::new().author(pubkey).kind(Kind::RelayList).limit(1);
        let events = self.client.database().query(vec![filter]).await?;
        Ok(Self::relay_urls_from_events(events))
    }

    pub async fn query_user_inbox_relays(&self, pubkey: PublicKey) -> Result<Vec<String>> {
        let filter = Filter::new()
            .author(pubkey)
            .kind(Kind::InboxRelays)
            .limit(1);
        let events = self.client.database().query(vec![filter]).await?;

        Ok(Self::relay_urls_from_events(events))
    }

    pub async fn query_user_key_package_relays(&self, pubkey: PublicKey) -> Result<Vec<String>> {
        let filter = Filter::new()
            .author(pubkey)
            .kind(Kind::MlsKeyPackageRelays)
            .limit(1);
        let events = self.client.database().query(vec![filter]).await?;

        Ok(Self::relay_urls_from_events(events))
    }

    pub async fn query_user_key_packages(&self, pubkey: PublicKey) -> Result<Events> {
        let filter = Filter::new()
            .author(pubkey)
            .kind(Kind::MlsKeyPackage)
            .limit(1);
        let events = self.client.database().query(vec![filter]).await?;
        Ok(events)
    }

    pub async fn query_contact_list_pubkeys(&self) -> Result<Vec<PublicKey>> {
        let pubkey = self.client.signer().await?.get_public_key().await.unwrap();

        let filter = Filter::new()
            .kind(Kind::ContactList)
            .author(pubkey)
            .limit(1);
        let events = self.client.database().query(vec![filter]).await?;

        let contacts_pubkeys = if let Some(event) = events.first() {
            event
                .tags
                .iter()
                .filter(|tag| tag.kind() == TagKind::p())
                .filter_map(|tag| tag.content().map(|c| PublicKey::from_hex(c).unwrap()))
                .collect()
        } else {
            vec![]
        };

        Ok(contacts_pubkeys)
    }

    pub async fn query_contacts(&self) -> Result<Vec<Event>> {
        let contacts_pubkeys = self.query_contact_list_pubkeys().await?;
        // If there are no contacts, return an empty vector
        if contacts_pubkeys.is_empty() {
            return Ok(vec![]);
        }
        let filter = Filter::new().kind(Kind::Metadata).authors(contacts_pubkeys);
        let events = self.client.database().query(vec![filter]).await?;

        Ok(events.into_iter().collect())
    }

    pub async fn query_user_welcomes(&self, pubkey: PublicKey) -> Result<Vec<UnsignedEvent>> {
        let gw_events = self.query_user_giftwrapped_events(pubkey).await?;
        let invites = self.extract_invite_events(gw_events).await;
        Ok(invites)
    }

    async fn query_user_giftwrapped_events(&self, pubkey: PublicKey) -> Result<Vec<Event>> {
        let filter = Filter::new().kind(Kind::GiftWrap).pubkeys(vec![pubkey]);
        let events = self.client.database().query(vec![filter]).await?;
        Ok(events.into_iter().collect())
    }

    pub async fn query_mls_group_messages(&self, group_ids: Vec<String>) -> Result<Vec<Event>> {
        let filter = Filter::new()
            .kind(Kind::MlsGroupMessage)
            .custom_tag(SingleLetterTag::lowercase(Alphabet::H), group_ids);
        let events = self.client.database().query(vec![filter]).await?;
        Ok(events.into_iter().collect())
    }
}
