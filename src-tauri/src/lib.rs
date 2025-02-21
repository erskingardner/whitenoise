mod accounts;
mod commands;
mod database;
mod groups;
mod invites;
mod key_packages;
mod messages;
mod nostr_manager;
mod relays;
mod secrets_store;
mod types;
mod utils;
mod whitenoise;

use crate::commands::accounts::*;
use crate::commands::delete_all_data;
use crate::commands::groups::*;
use crate::commands::invites::*;
use crate::commands::key_packages::*;
use crate::commands::messages::*;
use crate::commands::nostr::*;
use crate::whitenoise::Whitenoise;
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{filter::EnvFilter, fmt::Layer, prelude::*, registry::Registry};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            // Get the port from environment variable or use default
            let port = std::env::var("TAURI_DEV_PORT").unwrap_or_else(|_| "1420".to_string());

            let data_dir = app
                .handle()
                .path()
                .app_data_dir()
                .expect("Failed to get data dir");

            let logs_dir = app.handle().path().app_log_dir().unwrap();

            let formatted_data_dir = if cfg!(dev) {
                PathBuf::from(format!(
                    "{}/dev/instance_{}",
                    data_dir.to_string_lossy(),
                    port
                ))
            } else {
                PathBuf::from(format!("{}/release", data_dir.to_string_lossy()))
            };
            std::fs::create_dir_all(&formatted_data_dir)?;

            let formatted_logs_dir = if cfg!(dev) {
                PathBuf::from(format!(
                    "{}/dev/instance_{}",
                    logs_dir.to_string_lossy(),
                    port
                ))
            } else {
                PathBuf::from(format!("{}/release", logs_dir.to_string_lossy()))
            };
            std::fs::create_dir_all(&formatted_logs_dir)?;

            setup_logging(formatted_logs_dir.clone())?;

            #[cfg(debug_assertions)]
            {
                // Open devtools on debug builds
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
                window.close_devtools();
                // Customize the window title based on port
                window.set_title(&format!("White Noise (port {})", &port))?;
            }

            tauri::async_runtime::block_on(async move {
                let whitenoise =
                    Whitenoise::new(formatted_data_dir, formatted_logs_dir, app.handle().clone())
                        .await;
                app.manage(whitenoise);
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            create_identity,
            get_accounts,
            set_active_account,
            login,
            logout,
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
            publish_new_key_package,
            delete_all_key_packages,
            valid_key_package_exists_for_user,
            publish_relay_list,
            update_account_onboarding,
            get_group,
            get_group_and_messages,
            get_group_members,
            get_group_admins,
            rotate_key_in_group,
            get_invite,
            accept_invite,
            decline_invite,
            send_mls_message,
            delete_all_data,
            search_for_enriched_contacts,
            invite_to_white_noise,
            query_message,
            export_nsec
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

    // Create non-blocking writers for both stdout and file
    let (non_blocking_file, file_guard) = tracing_appender::non_blocking(file_appender);
    let (non_blocking_stdout, stdout_guard) = tracing_appender::non_blocking(std::io::stdout());

    static GUARDS: Lazy<Mutex<Option<(WorkerGuard, WorkerGuard)>>> = Lazy::new(|| Mutex::new(None));
    *GUARDS.lock().unwrap() = Some((file_guard, stdout_guard));

    Registry::default()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug")))
        .with(Layer::new().with_writer(non_blocking_stdout))
        .with(Layer::new().with_writer(non_blocking_file))
        .init();

    Ok(())
}
