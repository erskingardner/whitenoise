use crate::accounts::Account;
use crate::whitenoise::Whitenoise;

use nostr_sdk::prelude::*;

/// Logs in with the given public key. Will set the active account if successful.
///
/// # Arguments
///
/// * `wn` - A reference to the Whitenoise state.
/// * `hex_pubkey` - The public key in hexadecimal format.
///
/// # Returns
///
/// * `Ok(Account)` - The account if login was successful.
/// * `Err(String)` - An error message if there was an issue logging in.
#[tauri::command]
pub async fn login(
    nsec_or_hex_privkey: String,
    wn: tauri::State<'_, Whitenoise>,
    app_handle: tauri::AppHandle,
) -> Result<Account, String> {
    let keys = Keys::parse(&nsec_or_hex_privkey).map_err(|e| e.to_string())?;

    match Account::find_by_pubkey(&keys.public_key, wn.clone()).await {
        Ok(account) => {
            tracing::debug!("Account found, setting active");
            account
                .set_active(wn.clone(), &app_handle)
                .await
                .map_err(|e| format!("Error logging in: {}", e))
        }
        _ => {
            tracing::debug!(target: "whitenoise::commands::accounts","Account not found, adding from keys");
            Account::add_from_keys(&keys, true, wn.clone(), &app_handle)
                .await
                .map_err(|e| format!("Error logging in: {}", e))
        }
    }
}
