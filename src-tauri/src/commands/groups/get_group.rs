use crate::groups::{Group, GroupWithRelays};
use crate::whitenoise::Whitenoise;
use nostr_sdk::prelude::*;

/// Gets a single MLS group by its group ID
///
/// # Arguments
/// * `group_id` - Hex encoded MLS group ID
/// * `wn` - Whitenoise state
///
/// # Returns
/// * `Ok(Group)` - The requested group if found
/// * `Err(String)` - Error message if group not found or other error occurs
///
/// # Errors
/// Returns error if:
/// - Group ID is not valid hex
/// - Group not found in database
/// - Database error occurs
#[tauri::command]
pub async fn get_group(
    group_id: &str,
    wn: tauri::State<'_, Whitenoise>,
) -> Result<GroupWithRelays, String> {
    let mls_group_id =
        hex::decode(group_id).map_err(|e| format!("Error decoding group id: {}", e))?;
    let group = Group::find_by_mls_group_id(&mls_group_id, wn.clone())
        .await
        .map_err(|e| format!("Error fetching group: {}", e))?;
    let relays = group.relays(wn.clone()).await.map_err(|e| e.to_string())?;
    tracing::debug!(
        target: "whitenoise::commands::groups::get_group",
        "Group Relays: {:?}",
        relays
    );
    Ok(GroupWithRelays { group, relays })
}
