mod accounts;
mod app_settings;
mod database;
mod nostr;
mod nostr_mls;
mod secrets_store;
mod whitenoise;

// --- Imports ---
use crate::accounts::{create_identity, get_accounts, login, logout, set_current_identity};
use crate::database::delete_app_data;
use crate::nostr::{
    fetch_dev_events, get_contacts, get_legacy_chats, get_metadata_for_pubkey, send_message,
};
use crate::nostr_mls::key_packages::{generate_and_publish_key_package, parse_key_package};
use crate::whitenoise::Whitenoise;
use log::debug;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .setup(|app| {
            tauri::async_runtime::block_on(async move {
                let data_dir = app
                    .handle()
                    .path()
                    .app_data_dir()
                    .expect("Failed to get app data dir");

                // let cache_dir = app
                //     .handle()
                //     .path()
                //     .cache_dir()
                //     .expect("Failed to get cache dir");

                debug!("Whitenoise data dir: {:?}", &data_dir.as_path());

                // Initialize Whitenoise
                let whitenoise = Whitenoise::new(data_dir.as_path()).await?;

                // Update Nostr signer with keys for the current identity if they exist
                let nostr_keys = whitenoise
                    .accounts
                    .lock()
                    .unwrap()
                    .get_nostr_keys_for_current_identity()?;
                if let Some(keys) = nostr_keys {
                    whitenoise.update_nostr_signer_with_keys(keys).await?;
                }

                app.manage(whitenoise);
                Ok(())
            })
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            delete_app_data,
            get_accounts,
            set_current_identity,
            login,
            logout,
            get_contacts,
            get_metadata_for_pubkey,
            create_identity,
            get_legacy_chats,
            fetch_dev_events,
            send_message,
            generate_and_publish_key_package,
            parse_key_package,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
