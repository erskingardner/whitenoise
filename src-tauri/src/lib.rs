mod account_manager;
mod commands;
mod group_manager;
mod groups;
mod invites;
mod key_packages;
mod nostr_manager;
mod secrets_store;
mod types;
mod utils;
mod whitenoise;

use crate::commands::accounts::*;
use crate::commands::groups::*;
use crate::commands::invites::*;
use crate::commands::key_packages::*;
use crate::commands::nostr::*;
use crate::whitenoise::Whitenoise;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            tracing_subscriber::fmt::init();

            tauri::async_runtime::block_on(async move {
                let data_dir = app
                    .handle()
                    .path()
                    .app_data_dir()
                    .expect("Failed to get data dir");

                let whitenoise = Whitenoise::new(data_dir).await;
                app.manage(whitenoise);
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_active_account,
            get_accounts_state,
            login,
            logout,
            create_identity,
            set_active_account,
            init_nostr_for_current_user,
            fetch_contacts_with_metadata,
            fetch_enriched_contact,
            fetch_enriched_contacts,
            fetch_metadata,
            fetch_relays,
            encrypt_content,
            decrypt_content,
            create_group,
            get_groups,
            get_invites,
            publish_key_package,
            valid_key_package_exists_for_user,
            publish_relay_list,
            update_account_onboarding,
            get_group,
            get_invite,
            accept_invite,
            decline_invite,
            send_mls_message,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
