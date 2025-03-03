use crate::accounts::Account;
use crate::whitenoise::Whitenoise;
use nostr_sdk::prelude::*;

/// Logs out the specified account.
///
/// This function:
/// 1. Removes the account from the account manager
/// 2. Removes the private key from the secrets store
/// 3. Updates the Nostr identity to the new active account if needed
///
/// # Arguments
///
/// * `wn` - A reference to the Whitenoise state
/// * `hex_pubkey` - The public key in hexadecimal format of the account to log out
///
/// # Returns
///
/// * `Ok(())` - If the logout was successful
/// * `Err(String)` - An error message if there was an issue during logout
#[tauri::command]
pub async fn logout(
    hex_pubkey: String,
    wn: tauri::State<'_, Whitenoise>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let pubkey =
        PublicKey::parse(&hex_pubkey).map_err(|e| format!("Error parsing public key: {}", e))?;
    let account = Account::find_by_pubkey(&pubkey, wn.clone())
        .await
        .map_err(|e| format!("Error fetching account: {}", e))?;
    account
        .remove(wn.clone(), app_handle)
        .await
        .map_err(|e| format!("Error logging out: {}", e))
}
