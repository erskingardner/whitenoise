use crate::accounts::Account;
use crate::whitenoise::Whitenoise;
use nostr_openmls::NostrMls;
use nostr_sdk::prelude::*;

#[tauri::command]
pub async fn init_nostr_for_current_user(
    wn: tauri::State<'_, Whitenoise>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let current_account = Account::get_active(wn.clone())
        .await
        .map_err(|e| e.to_string())?;

    // Update Nostr identity and connect relays
    wn.nostr
        .set_nostr_identity(&current_account, wn.clone(), &app_handle)
        .await
        .map_err(|e| e.to_string())?;

    // Then update Nostr MLS instance
    {
        let mut nostr_mls = wn.nostr_mls.lock().await;
        *nostr_mls = NostrMls::new(wn.data_dir.clone(), Some(current_account.pubkey.to_hex()));
    }

    tracing::debug!(
        target: "whitenoise::commands::nostr::init_nostr_for_current_user",
        "Nostr initialized for current user"
    );
    Ok(())
}
