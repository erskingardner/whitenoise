use crate::invites::Invite;
use crate::whitenoise::Whitenoise;
use nostr_sdk::prelude::*;

/// Gets a specific invite by its ID.
///
/// # Arguments
/// * `invite_id` - The ID of the invite to retrieve
/// * `wn` - The Whitenoise state
///
/// # Returns
/// * `Ok(Invite)` if the invite was found
/// * `Err(String)` if there was an error retrieving the invite or it wasn't found
#[tauri::command]
pub async fn get_invite(
    active_account: String,
    invite_id: String,
    wn: tauri::State<'_, Whitenoise>,
) -> Result<Invite, String> {
    Invite::find_by_id(&active_account, &invite_id, wn.clone())
        .await
        .map_err(|e| e.to_string())
}
