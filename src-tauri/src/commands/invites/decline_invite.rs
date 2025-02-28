use crate::invites::{Invite, InviteState};
use crate::whitenoise::Whitenoise;
use nostr_sdk::prelude::*;
use tauri::Emitter;

/// Declines a group invite.
///
/// # Arguments
/// * `invite` - The invite to decline
/// * `wn` - The Whitenoise state
/// * `app_handle` - The Tauri app handle
///
/// # Returns
/// * `Ok(())` if the invite was successfully declined
/// * `Err(String)` if there was an error declining the invite
///
/// # Events Emitted
/// * `invite_declined` - Emitted with the updated invite after it is declined
#[tauri::command]
pub async fn decline_invite(
    mut invite: Invite,
    wn: tauri::State<'_, Whitenoise>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    tracing::debug!(target: "whitenoise::invites::decline_invite", "Declining invite {:?}", invite.event.id.unwrap());

    invite.state = InviteState::Declined;
    invite.save(wn.clone()).await.map_err(|e| e.to_string())?;

    app_handle
        .emit("invite_declined", invite)
        .map_err(|e| e.to_string())?;

    Ok(())
}
