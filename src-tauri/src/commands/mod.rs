use crate::whitenoise::Whitenoise;
use tauri::Manager;

pub mod accounts;
pub mod groups;
pub mod invites;
pub mod key_packages;
pub mod nostr;

#[tauri::command]
pub async fn delete_all_data(app_handle: tauri::AppHandle) -> Result<(), String> {
    // Unmanage the current Whitenoise instance
    let old_wn = app_handle.unmanage::<Whitenoise>();

    // Delete all data and reset the Whitenoise state instance
    if let Some(wn) = old_wn {
        wn.delete_all_data().await.map_err(|e| e.to_string())?;
        app_handle.manage(Whitenoise::new(wn.data_dir, wn.logs_dir, &app_handle).await);
    }

    Ok(())
}
