#[cfg_attr(mobile, tauri::mobile_entry_point)]
// --- Sub-Modules ---
mod app_settings;
mod database;
mod identities;
mod nostr;
mod secrets_store;

// --- Imports ---
use crate::app_settings::AppSettings;
use crate::database::{delete_app_data, Database};
use crate::identities::{
    create_identity, get_current_identity, get_identities, login, logout, set_current_identity,
    nip04_decrypt,
};
use crate::nostr::{get_contacts, init_nostr_for_pubkey};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::Manager;

// --- Structs ---
struct AppState {
    db: Arc<Database>,
    settings: Arc<Mutex<AppSettings>>,
}

// --- Setup App State ---
/// Initialize the app state
/// This will create or reuse a local embedded database
/// AppSettings will be loaded from the database or defaulted if none exists
async fn init_app_state(databse_path: PathBuf) -> Result<AppState> {
    let db = Database::new(databse_path)?;
    let settings =
        AppSettings::from_database(&db).context("Couldn't load settings from database")?;

    if settings.clone().current_identity.is_some() {
        init_nostr_for_pubkey(settings.clone().current_identity.unwrap()).await?;
    }

    Ok(AppState {
        db: Arc::new(db),
        settings: Arc::new(Mutex::new(settings)),
    })
}

// --- Run ---
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            tauri::async_runtime::block_on(async move {
                let app_handle = app.handle();
                let app_data_dir = app_handle
                    .clone()
                    .path()
                    .app_data_dir()
                    .expect("Failed to get app data dir");

                // TODO: Remove this later - just for debugging
                // println!("App data dir: {:?}", app_data_dir);

                let app_state = init_app_state(app_data_dir).await?;
                app.manage(app_state);
                Ok(())
            })
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            login,
            logout,
            get_identities,
            get_current_identity,
            set_current_identity,
            create_identity,
            get_contacts,
            nip04_decrypt,
            delete_app_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
