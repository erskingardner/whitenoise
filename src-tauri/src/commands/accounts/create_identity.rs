use crate::accounts::Account;
use crate::whitenoise::Whitenoise;
use nostr_sdk::prelude::*;

/// Creates a new identity by generating a new keypair and logging in with it.
///
/// # Arguments
///
/// * `wn` - A reference to the Whitenoise state.
/// * `app_handle` - The app handle.
///
/// # Returns
///
/// * `Ok(Account)` - The newly created account.
/// * `Err(String)` - An error message if there was an issue creating the identity.
#[tauri::command]
pub async fn create_identity(
    wn: tauri::State<'_, Whitenoise>,
    app_handle: tauri::AppHandle,
) -> Result<Account, String> {
    let account = Account::new(wn.clone())
        .await
        .map_err(|e| format!("Error creating account: {}", e))?;
    account
        .set_active(wn.clone(), &app_handle)
        .await
        .map_err(|e| format!("Error setting active account: {}", e))
}
