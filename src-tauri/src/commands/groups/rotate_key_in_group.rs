use crate::groups::Group;
use crate::whitenoise::Whitenoise;
use nostr_sdk::prelude::*;

#[tauri::command]
pub async fn rotate_key_in_group(
    group_id: &str,
    wn: tauri::State<'_, Whitenoise>,
) -> Result<(), String> {
    let mls_group_id =
        hex::decode(group_id).map_err(|e| format!("Error decoding group id: {}", e))?;
    let group = Group::find_by_mls_group_id(&mls_group_id, wn.clone())
        .await
        .map_err(|e| format!("Error fetching group: {}", e))?;
    group
        .self_update_keys(wn.clone())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
