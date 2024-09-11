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
use crate::nostr::init_nostr_with_pubkey;
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::Manager;

// --- Structs ---
#[allow(dead_code)]
struct AppState {
    db: Arc<Database>,
    settings: Arc<Mutex<AppSettings>>,
    app_handle: tauri::AppHandle,
}

// --- Setup App State ---
/// Initialize the app state
/// This will create or reuse a local embedded database
/// AppSettings will be loaded from the database or defaulted if none exists
fn init_app_state(databse_path: PathBuf, app_handle: tauri::AppHandle) -> Result<AppState> {
    let db = Database::new(databse_path)?;
    let settings =
        AppSettings::from_database(&db).context("Couldn't load settings from database")?;
    Ok(AppState {
        db: Arc::new(db),
        settings: Arc::new(Mutex::new(settings)),
        app_handle,
    })
}

// --- Run ---
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle();
            let app_data_dir = app_handle
                .clone()
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir");
            println!("App data dir: {:?}", app_data_dir);
            let app_state = init_app_state(app_data_dir, app_handle.clone())?;
            // TODO: We should handle errors here and retry or send an event to the frontend for display.
            app.manage(app_state);
            Ok(())
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
            nip04_decrypt,
            delete_app_data,
            init_nostr_with_pubkey
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
