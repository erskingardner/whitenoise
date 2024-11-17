use crate::account_manager::{Account, AccountError, AccountManagerState};
use crate::secrets_store;
use crate::whitenoise::Whitenoise;
use nostr_sdk::Keys;
use openmls_nostr::NostrMls;
use tauri::Emitter;
use tokio::spawn;

/// Retrieves the currently active account.
///
/// # Arguments
///
/// * `wn` - A reference to the Whitenoise state.
///
/// # Returns
///
/// * `Ok(Account)` - The active account if it exists.
/// * `Err(String)` - An error message if there was an issue fetching the active account.
#[tauri::command]
pub fn get_active_account(wn: tauri::State<'_, Whitenoise>) -> Result<Account, String> {
    wn.account_manager
        .get_active_account()
        .map_err(|e| format!("Error fetching active account: {}", e))
}

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
pub fn get_accounts_state(wn: tauri::State<'_, Whitenoise>) -> Result<AccountManagerState, String> {
    wn.account_manager
        .get_accounts_state()
        .map_err(|e| format!("Error listing accounts: {}", e))
}

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
) -> Result<AccountManagerState, String> {
    let keys = Keys::parse(nsec_or_hex_privkey).map_err(|e| e.to_string())?;

    if wn
        .account_manager
        .account_exists(&keys.public_key().to_hex())
        .map_err(|e| format!("Error checking if account exists: {}", e))?
    {
        tracing::debug!(
            target: "whitenoise::commands::accounts::login",
            "Account already exists: {:?}",
            keys.public_key().to_hex()
        );
        set_active_account(keys.public_key().to_hex(), wn.clone(), app_handle).await?;
    } else {
        // Add the account to the account manager
        let account = wn
            .account_manager
            .add_account(keys.clone(), true, &wn.nostr)
            .await
            .map_err(|e| format!("Error logging in: {}", e))?;

        tracing::debug!(
            target: "whitenoise::commands::accounts::login",
            "Added account: {:?}",
            account
        );

        // Store private key in secrets store
        secrets_store::store_private_key(&keys, &wn.data_dir)
            .map_err(|e| format!("Failed to store private key: {}", e))?;

        tracing::debug!(
            target: "whitenoise::account_manager::add_account",
            "Saved private key to secrets store"
        );

        wn.group_manager
            .set_active_account(keys.clone().public_key().to_hex())
            .map_err(|e| format!("Error setting active account on group manager: {}", e))?;

        app_handle
            .emit("account_changed", ())
            .map_err(|e| e.to_string())?;

        // Update Nostr identity and connect relays
        wn.nostr
            .update_nostr_identity(keys.clone())
            .await
            .map_err(|e| e.to_string())?;

        let pubkey = keys.public_key();
        let last_synced = account.last_synced;
        let group_ids = account.nostr_group_ids;
        let nostr = wn.nostr.clone();

        // Spawn two tasks in parallel:
        // 1. Negentropy sync for past events
        // 2. Setup subscriptions to catch future events
        spawn(async move {
            match nostr
                .sync_for_user(pubkey, last_synced, group_ids.clone())
                .await
            {
                Ok(_) => {
                    tracing::debug!(
                        target: "whitenoise::commands::accounts::login",
                        "Negentropy event sync completed"
                    );
                }
                Err(e) => {
                    tracing::error!(
                        target: "whitenoise::commands::accounts::login",
                        "Error in negentropy sync: {}",
                        e
                    );
                }
            }

            if let Err(e) = nostr.setup_subscriptions(pubkey, group_ids.clone()).await {
                tracing::error!(
                    target: "whitenoise::commands::accounts::login",
                    "Error subscribing to events: {}",
                    e
                );
            }
        });

        app_handle
            .emit("nostr_ready", ())
            .map_err(|e| e.to_string())?;

        // Update Nostr MLS instance
        {
            let mut nostr_mls = wn
                .nostr_mls
                .lock()
                .map_err(|e| format!("Error locking Nostr MLS: {}", e))?;
            *nostr_mls = NostrMls::new(wn.data_dir.clone(), Some(keys.public_key().to_hex()));
        }
    }
    Ok(wn.account_manager.get_accounts_state().unwrap())
}

/// Creates a new identity by generating a new keypair and logging in with it.
///
/// # Arguments
///
/// * `wn` - A reference to the Whitenoise state.
///
/// # Returns
///
/// * `Ok(Accounts)` - The newly created and logged in account.
/// * `Err(String)` - An error message if there was an issue creating the identity.
#[tauri::command]
pub async fn create_identity(
    wn: tauri::State<'_, Whitenoise>,
    app_handle: tauri::AppHandle,
) -> Result<AccountManagerState, String> {
    let keys = Keys::generate();
    let account_state = login(keys.secret_key().to_secret_hex(), wn, app_handle).await?;
    Ok(account_state)
}

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
) -> Result<AccountManagerState, String> {
    wn.account_manager
        .set_active_account(Some(hex_pubkey.clone()))
        .map_err(|e| format!("Error switching account: {}", e))?;

    let keys = secrets_store::get_nostr_keys_for_pubkey(&hex_pubkey, &wn.data_dir)
        .map_err(|e| format!("Error fetching keys: {}", e))?;

    let active_account = wn
        .account_manager
        .get_active_account()
        .map_err(|e| format!("Error fetching active account: {}", e))?;

    wn.group_manager
        .set_active_account(hex_pubkey.clone())
        .map_err(|e| format!("Error setting active account on group manager: {}", e))?;

    app_handle
        .emit("account_changed", ())
        .map_err(|e| e.to_string())?;

    // Update Nostr identity and connect relays
    wn.nostr
        .update_nostr_identity(keys.clone())
        .await
        .map_err(|e| e.to_string())?;

    let pubkey = keys.public_key();
    let last_synced = active_account.last_synced;
    let group_ids = active_account.nostr_group_ids.clone();
    let nostr = wn.nostr.clone();

    spawn(async move {
        if let Err(e) = nostr.sync_for_user(pubkey, last_synced, group_ids).await {
            tracing::error!("Error during background sync: {}", e);
        }
    });

    // TODO: Create subscriptions

    app_handle
        .emit("nostr_ready", ())
        .map_err(|e| e.to_string())?;

    // Update Nostr MLS instance
    {
        let mut nostr_mls = wn
            .nostr_mls
            .lock()
            .map_err(|e| format!("Error locking Nostr MLS: {}", e))?;
        *nostr_mls = NostrMls::new(wn.data_dir.clone(), Some(hex_pubkey.clone()));
    }

    Ok(wn.account_manager.get_accounts_state().unwrap())
}

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
) -> Result<AccountManagerState, String> {
    // Remove the account from the account manager
    wn.account_manager
        .remove_account(hex_pubkey.clone())
        .map_err(|e| format!("Error removing account: {}", e))?;

    tracing::debug!(
        target: "whitenoise::commands::accounts::logout",
        "Removed account. Accounts now: {:?}",
        wn.account_manager.get_accounts_state().unwrap()
    );

    // Remove the private key from the secrets store
    secrets_store::remove_private_key_for_pubkey(&hex_pubkey, &wn.data_dir)
        .map_err(|e| format!("Error removing private key: {}", e))?;

    // Update Nostr identity to the new current user
    match wn.account_manager.get_active_account() {
        Ok(current_account) => {
            // If the current identity is not the same as the Nostr identity, update the Nostr identity
            if current_account.pubkey
                != wn
                    .nostr
                    .client
                    .signer()
                    .await
                    .map_err(|e| format!("Error fetching signer: {}", e))?
                    .get_public_key()
                    .await
                    .map_err(|e| format!("Error fetching public key: {}", e))?
                    .to_hex()
            {
                let keys =
                    secrets_store::get_nostr_keys_for_pubkey(&current_account.pubkey, &wn.data_dir)
                        .map_err(|e| format!("Error fetching keys: {}", e))?;

                // Update Nostr identity and connect relays
                wn.nostr
                    .update_nostr_identity(keys.clone())
                    .await
                    .map_err(|e| e.to_string())?;

                let pubkey = keys.public_key();
                let last_synced = current_account.last_synced;
                let group_ids = current_account.nostr_group_ids.clone();
                let nostr = wn.nostr.clone();

                spawn(async move {
                    if let Err(e) = nostr.sync_for_user(pubkey, last_synced, group_ids).await {
                        tracing::error!("Error during background sync: {}", e);
                    }
                });

                // TODO: Create subscriptions

                app_handle
                    .emit("nostr_ready", ())
                    .map_err(|e| e.to_string())?;
            }
        }
        Err(AccountError::NoAccountsExist) => return Err("No accounts exist".to_string()),
        Err(e) => return Err(format!("Error fetching active account: {}", e)),
    }

    app_handle
        .emit("account_changed", ())
        .map_err(|e| e.to_string())?;

    Ok(wn.account_manager.get_accounts_state().unwrap())
}

/// Updates the onboarding state of an account.
#[tauri::command]
pub fn update_account_onboarding(
    pubkey: String,
    inbox_relays: bool,
    key_package_relays: bool,
    publish_key_package: bool,
    wn: tauri::State<'_, Whitenoise>,
) -> Result<(), String> {
    wn.account_manager
        .update_account_onboarding(
            pubkey,
            inbox_relays,
            key_package_relays,
            publish_key_package,
        )
        .map_err(|e| format!("Error updating account onboarding: {}", e))
}
