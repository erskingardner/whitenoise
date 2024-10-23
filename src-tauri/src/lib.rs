mod accounts;
mod app_settings;
mod database;
mod nostr;
mod nostr_mls;
mod secrets_store;
mod whitenoise;

// --- Imports ---
use crate::accounts::{create_identity, get_accounts, login, logout, set_current_identity};
use crate::nostr::{
    decrypt_content, get_contact, get_contacts, get_legacy_chats, get_metadata_for_pubkey,
    send_message,
};
use crate::nostr_mls::groups::{
    create_group, fetch_and_process_mls_messages, get_group_member_pubkeys, get_groups,
    send_mls_message,
};
use crate::nostr_mls::invites::{accept_invite, decline_invite, fetch_invites_for_user};
use crate::nostr_mls::key_packages::{
    delete_all_key_packages_from_relays, generate_and_publish_key_package, parse_key_package,
};
use crate::whitenoise::{delete_data, Whitenoise};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            tauri::async_runtime::block_on(async move {
                let data_dir = app
                    .handle()
                    .path()
                    .app_data_dir()
                    .expect("Failed to get app data dir");

                // Initialize Whitenoise
                let whitenoise = Whitenoise::new(data_dir.clone()).await?;

                // Update Nostr signer with keys for the current identity if they exist
                let nostr_keys = whitenoise
                    .accounts
                    .lock()
                    .unwrap()
                    .get_nostr_keys_for_current_identity(&data_dir)?;
                if let Some(keys) = nostr_keys {
                    whitenoise.update_nostr_signer_with_keys(keys).await?;
                }

                app.manage(whitenoise);
                Ok(())
            })
        })
        .invoke_handler(tauri::generate_handler![
            accept_invite,
            create_group,
            create_identity,
            decline_invite,
            decrypt_content,
            delete_all_key_packages_from_relays,
            delete_data,
            fetch_and_process_mls_messages,
            fetch_invites_for_user,
            generate_and_publish_key_package,
            get_accounts,
            get_contact,
            get_contacts,
            get_group_member_pubkeys,
            get_groups,
            get_legacy_chats,
            get_metadata_for_pubkey,
            login,
            logout,
            parse_key_package,
            send_message,
            send_mls_message,
            set_current_identity,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
