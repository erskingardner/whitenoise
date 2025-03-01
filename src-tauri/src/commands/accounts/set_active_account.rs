use crate::accounts::Account;
use crate::whitenoise::Whitenoise;

use nostr_sdk::prelude::*;

/// Sets the active account.
///
/// # Arguments
///
/// * `wn` - A reference to the Whitenoise state.
/// * `hex_pubkey` - The public key in hexadecimal format.
///
/// # Returns
///
/// * `Ok(())` - If the active account was set successfully.
/// * `Err(String)` - An error message if there was an issue setting the active account.
#[tauri::command]
pub async fn set_active_account(
    hex_pubkey: String,
    wn: tauri::State<'_, Whitenoise>,
    app_handle: tauri::AppHandle,
) -> Result<Account, String> {
    tracing::debug!(target: "whitenoise::commands::accounts", "Setting active account: {}", hex_pubkey);

    let pubkey =
        PublicKey::parse(&hex_pubkey).map_err(|e| format!("Error parsing public key: {}", e))?;

    let mut account = Account::find_by_pubkey(&pubkey, wn.clone())
        .await
        .map_err(|e| format!("Error fetching account: {}", e))?;

    account.active = true;

    account
        .set_active(wn.clone(), &app_handle)
        .await
        .map_err(|e| format!("Error setting active account: {}", e))
}
