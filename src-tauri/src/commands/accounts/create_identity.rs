use crate::accounts::Account;
use crate::commands::accounts::update_account_onboarding;
use crate::commands::nostr::publish_relay_list;
use crate::key_packages::publish_key_package;
use crate::relays::RelayMeta;
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
    name: String,
    wn: tauri::State<'_, Whitenoise>,
    app_handle: tauri::AppHandle,
) -> Result<Account, String> {
    let account = Account::new(name, wn.clone())
        .await
        .map_err(|e| format!("Error creating account: {}", e))?;
    account
        .set_active(wn.clone(), &app_handle)
        .await
        .map_err(|e| format!("Error setting active account: {}", e))?;

    // Auto-onboard the account
    let default_inbox_relays = vec![("wss://auth.nostr1.com".to_string(), RelayMeta::ReadWrite)];

    let default_relays = vec![
        ("wss://relay.damus.io".to_string(), RelayMeta::ReadWrite),
        ("wss://relay.primal.net".to_string(), RelayMeta::ReadWrite),
        ("wss://nos.lol".to_string(), RelayMeta::ReadWrite),
    ];

    // Publish an inbox relay list
    publish_relay_list(default_inbox_relays, 10050, wn.clone()).await?;

    // Publish a key package relay list
    publish_relay_list(default_relays.clone(), 10051, wn.clone()).await?;

    // Publish a nostr relay list
    publish_relay_list(default_relays, 10050, wn.clone()).await?;

    // Publish a key package
    publish_key_package(wn.clone())
        .await
        .map_err(|e| format!("Error publishing key package: {}", e))?;

    // Update the account onboarding status
    update_account_onboarding(account.pubkey.to_hex(), true, true, true, wn.clone()).await?;

    Ok(account)
}
