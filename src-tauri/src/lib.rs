mod account_manager;
mod commands;
mod database;
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
use crate::commands::delete_all_data;
use crate::commands::groups::*;
use crate::commands::invites::*;
use crate::commands::key_packages::*;
use crate::commands::nostr::*;
use crate::whitenoise::Whitenoise;
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;
use tracing_subscriber::{filter::EnvFilter, fmt::Layer, prelude::*, registry::Registry};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let data_dir = app
                .handle()
                .path()
                .app_data_dir()
                .expect("Failed to get data dir");

            let logs_dir = app.handle().path().app_log_dir().unwrap();

            let formatted_data_dir = if cfg!(dev) {
                PathBuf::from(format!("{}/dev", data_dir.to_string_lossy()))
            } else {
                PathBuf::from(format!("{}/release", data_dir.to_string_lossy()))
            };

            let formatted_logs_dir = if cfg!(dev) {
                PathBuf::from(format!("{}/dev", logs_dir.to_string_lossy()))
            } else {
                PathBuf::from(format!("{}/release", logs_dir.to_string_lossy()))
            };

            setup_logging(formatted_logs_dir.clone())?;

            // Open devtools on debug builds
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
                window.close_devtools();
            }

            tauri::async_runtime::block_on(async move {
                let whitenoise =
                    Whitenoise::new(formatted_data_dir, formatted_logs_dir, app.handle()).await;
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
            query_contacts_with_metadata,
            fetch_enriched_contact,
            query_enriched_contact,
            fetch_enriched_contacts,
            query_enriched_contacts,
            fetch_relays,
            encrypt_content,
            decrypt_content,
            create_group,
            get_groups,
            get_invites,
            publish_key_package,
            delete_all_key_packages,
            valid_key_package_exists_for_user,
            publish_relay_list,
            update_account_onboarding,
            get_group,
            get_invite,
            accept_invite,
            decline_invite,
            send_mls_message,
            fetch_mls_messages,
            delete_all_data
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_logging(logs_dir: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let file_appender = tracing_appender::rolling::RollingFileAppender::builder()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .filename_prefix("whitenoise")
        .filename_suffix("log")
        .build(logs_dir)?;

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    static GUARD: Lazy<Mutex<Option<tracing_appender::non_blocking::WorkerGuard>>> =
        Lazy::new(|| Mutex::new(None));
    *GUARD.lock().unwrap() = Some(guard);

    Registry::default()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(Layer::new().with_writer(std::io::stdout))
        .with(Layer::new().with_writer(non_blocking))
        .init();

    Ok(())
}
