use crate::accounts::Account;
use crate::whitenoise::Whitenoise;

use nostr_sdk::prelude::*;

/// Lists all accounts.
///
/// # Arguments
///
/// * `wn` - A reference to the Whitenoise state.
///
/// # Returns
///
/// * `Ok(Vec<Account>)` - A vector of accounts if successful.
/// * `Err(String)` - An error message if there was an issue listing the accounts.
#[tauri::command]
pub async fn get_accounts(wn: tauri::State<'_, Whitenoise>) -> Result<Vec<Account>, String> {
    Account::all(wn.clone())
        .await
        .map_err(|e| format!("Error fetching accounts: {}", e))
}
