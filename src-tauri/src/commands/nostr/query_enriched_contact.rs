use crate::accounts::Account;
use crate::relays::RelayType;
use crate::types::EnrichedContact;
use crate::whitenoise::Whitenoise;
use nostr_sdk::prelude::*;
use tauri::Emitter;

#[tauri::command]
pub async fn query_enriched_contact(
    pubkey: String,
    update_account: bool,
    wn: tauri::State<'_, Whitenoise>,
    app_handle: tauri::AppHandle,
) -> Result<EnrichedContact, String> {
    let pubkey = PublicKey::from_hex(&pubkey).map_err(|_| "Invalid pubkey".to_string())?;

    let metadata = wn
        .nostr
        .query_user_metadata(pubkey)
        .await
        .map_err(|_| "Failed to get metadata".to_string())?;
    let nostr_relays = wn
        .nostr
        .query_user_relays(pubkey)
        .await
        .map_err(|_| "Failed to get user relays".to_string())?;
    let inbox_relays = wn
        .nostr
        .query_user_inbox_relays(pubkey)
        .await
        .map_err(|_| "Failed to get inbox relays".to_string())?;
    let key_package_relays = wn
        .nostr
        .query_user_key_package_relays(pubkey)
        .await
        .map_err(|_| "Failed to get key package relays".to_string())?;
    let key_packages = wn
        .nostr
        .query_user_key_packages(pubkey)
        .await
        .map_err(|_| "Failed to get key packages".to_string())?;

    let enriched_contact = EnrichedContact {
        metadata: metadata.unwrap_or_default(),
        nip17: !inbox_relays.is_empty(),
        nip104: !key_packages.is_empty(),
        nostr_relays: nostr_relays.iter().map(|(url, _)| url.clone()).collect(),
        inbox_relays: inbox_relays.iter().map(|(url, _)| url.clone()).collect(),
        key_package_relays: key_package_relays
            .iter()
            .map(|(url, _)| url.clone())
            .collect(),
    };

    if update_account {
        let mut account = Account::find_by_pubkey(&pubkey, wn.clone())
            .await
            .map_err(|e| format!("Failed to find account: {}", e))?;

        account.metadata = enriched_contact.metadata.clone();
        account
            .update_relays(RelayType::Nostr, &nostr_relays, wn.clone())
            .await
            .map_err(|e| format!("Failed to update relays: {}", e))?;
        account
            .update_relays(RelayType::Inbox, &inbox_relays, wn.clone())
            .await
            .map_err(|e| format!("Failed to update relays: {}", e))?;
        account
            .update_relays(RelayType::KeyPackage, &key_package_relays, wn.clone())
            .await
            .map_err(|e| format!("Failed to update relays: {}", e))?;

        account
            .save(wn.clone())
            .await
            .map_err(|e| format!("Failed to save account: {}", e))?;
        app_handle
            .emit("account_changed", ())
            .map_err(|e| e.to_string())?;
    }

    Ok(enriched_contact)
}
