use crate::database::Database;
use crate::nostr_mls::{groups::NostrMlsGroup, invites::Invite};
use crate::whitenoise::Whitenoise;
use anyhow::{anyhow, Result};
use log::{debug, error};
use nostr_sdk::prelude::*;
use openmls::prelude::*;
use tauri::Manager;

pub struct EventProcessor {
    wn: Whitenoise,
    app_handle: tauri::AppHandle,
}

impl NostrEventProcessor {
    pub fn new(wn: Whitenoise, app_handle: tauri::AppHandle) -> Self {
        Self { wn, app_handle }
    }

    pub async fn process_event(&self, event: Event) -> Result<()> {
        match event.kind {
            Kind::GiftWrap => self.process_gift_wrap(event).await,
            Kind::Custom(443) => self.process_key_package(event).await,
            _ => Ok(()),
        }
    }

    async fn process_gift_wrap(&self, event: Event) -> Result<()> {
        debug!(target: "nostr_event_processor::process_gift_wrap", "Processing gift wrap: {:?}", event.id);
        // Unwrap the gift-wrappeed event and then process the rumor
        Ok(())
    }

    async fn process_welcome_message(&self, event: Event) -> Result<()> {
        debug!(target: "nostr_event_processor::process_welcome_message", "Processing welcome message: {:?}", event.id);

        // Parse the welcome message
        let invite = Invite::from_event(&event, self.wn.clone())?;

        // Save the invite to the database
        invite.save(self.wn.clone())?;

        // Emit a Tauri event
        self.app_handle.emit_all("new_invite", invite)?;

        Ok(())
    }

    async fn process_mls_message(&self, event: Event) -> Result<()> {
        debug!(target: "nostr_event_processor::process_mls_message", "Processing MLS message: {:?}", event.id);

        // Extract the group ID from the event tags
        let group_id = event
            .tags
            .iter()
            .find(|tag| {
                tag.kind() == TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::H))
            })
            .ok_or_else(|| anyhow!("No group ID found in event tags"))?
            .content()
            .ok_or_else(|| anyhow!("Invalid group ID tag"))?;

        // Fetch the NostrMlsGroup from the database
        let mut nostr_group = NostrMlsGroup::get_by_id(&group_id, self.wn.clone())?;

        // Process the MLS message
        let processed_message =
            nostr_group.process_message(event.content.clone(), &self.wn.nostr_mls.provider)?;

        // Update the group state in the database
        nostr_group.save(self.wn.clone())?;

        // Emit a Tauri event with the processed message
        self.app_handle
            .emit_all("mls_message_processed", processed_message)?;

        Ok(())
    }

    async fn process_key_package(&self, event: Event) -> Result<()> {
        debug!(target: "nostr_event_processor::process_key_package", "Processing key package: {:?}", event.id);

        // Parse and validate the key package
        let key_package =
            crate::nostr_mls::key_packages::parse_key_package(event.content, self.wn.clone())?;

        // Store the key package in the MLS provider's storage
        self.wn.nostr_mls.provider.storage().store_key_package(
            &key_package.hash_ref(self.wn.nostr_mls.provider.crypto())?,
            &key_package,
        )?;

        // Emit a Tauri event
        self.app_handle
            .emit_all("new_key_package", event.pubkey.to_string())?;

        Ok(())
    }
}
