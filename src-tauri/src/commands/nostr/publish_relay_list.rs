use crate::accounts::Account;
use crate::relays::{RelayMeta, RelayType};
use crate::whitenoise::Whitenoise;
use nostr_sdk::prelude::*;

#[tauri::command]
pub async fn publish_relay_list(
    relay_entries: Vec<(String, RelayMeta)>,
    kind: u64,
    wn: tauri::State<'_, Whitenoise>,
) -> Result<(), String> {
    let event_kind = Kind::from_u16(kind.try_into().expect("Invalid kind"));

    let relay_iter = relay_entries.iter().filter_map(|(url, meta)| {
        RelayUrl::parse(url)
            .ok()
            .map(|relay_url| (relay_url, meta.to_relay_metadata()))
    });

    let event_builder = match event_kind {
        Kind::RelayList => EventBuilder::relay_list(relay_iter),
        Kind::InboxRelays => {
            let mut tags: Vec<Tag> = Vec::new();
            for (relay, meta) in relay_entries.clone() {
                tags.push(Tag::custom(TagKind::Relay, [relay, meta.into()]));
            }
            EventBuilder::new(Kind::InboxRelays, "").tags(tags)
        }
        Kind::MlsKeyPackageRelays => {
            let mut tags: Vec<Tag> = Vec::new();
            for (relay, meta) in relay_entries.clone() {
                tags.push(Tag::custom(TagKind::Relay, [relay, meta.into()]));
            }
            EventBuilder::new(Kind::MlsKeyPackageRelays, "").tags(tags)
        }
        _ => return Err("Invalid relay list kind".to_string()),
    };

    wn.nostr
        .client
        .send_event_builder(event_builder)
        .await
        .map_err(|e| e.to_string())?;

    let active_account = Account::get_active(wn.clone())
        .await
        .map_err(|e| e.to_string())?;

    match event_kind {
        Kind::RelayList => {
            active_account
                .update_relays(RelayType::Nostr, &relay_entries, wn.clone())
                .await
                .map_err(|e| format!("Failed to update nostr relays: {}", e))?;
        }
        Kind::InboxRelays => {
            active_account
                .update_relays(RelayType::Inbox, &relay_entries, wn.clone())
                .await
                .map_err(|e| format!("Failed to update inbox relays: {}", e))?;
        }
        Kind::MlsKeyPackageRelays => {
            active_account
                .update_relays(RelayType::KeyPackage, &relay_entries, wn.clone())
                .await
                .map_err(|e| format!("Failed to update key package relays: {}", e))?;
        }
        _ => return Err("Invalid relay list kind".to_string()),
    }
    Ok(())
}
