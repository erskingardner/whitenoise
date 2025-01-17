use crate::accounts::Account;
use crate::commands::key_packages::publish_key_package;
use crate::groups::{Group, GroupType};
use crate::invites::{Invite, InviteState};
use crate::key_packages::delete_key_package_from_relays;
use crate::whitenoise::Whitenoise;
use nostr_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use tauri::Emitter;

#[derive(Debug, Serialize, Deserialize)]
pub struct InvitesWithFailures {
    invites: Vec<Invite>,
    failures: Vec<(EventId, String)>,
}

/// Fetches and processes pending invites for the active user.
///
/// This command:
/// 1. Gets stored invites from the group manager
/// 2. Fetches new welcome messages from Nostr relays
/// 3. Processes new welcome messages into invites
/// 4. Cleans up used key packages
/// 5. Generates replacement key packages
///
/// # Arguments
/// * `wn` - The Whitenoise state
///
/// # Returns
/// * `Ok(InvitesWithFailures)` containing:
///   - `invites`: Vec of successfully processed Invite objects
///   - `failures`: Vec of (EventId, error message) for failed invite processing
///
/// # Errors
/// Returns `Err(String)` if:
/// - Cannot get active account
/// - Cannot access group manager
/// - Cannot fetch welcome messages
/// - Cannot process welcome messages
/// - Cannot delete used key packages
/// - Cannot generate new key packages
///
/// # Events
/// No events are emitted by this command.
#[tauri::command]
pub async fn get_invites(
    wn: tauri::State<'_, Whitenoise>,
    app_handle: tauri::AppHandle,
) -> Result<InvitesWithFailures, String> {
    let active_account = Account::get_active(wn.inner()).map_err(|e| e.to_string())?;

    let mut invites: Vec<Invite> = Vec::new();

    let stored_invites = active_account
        .invites(wn.inner())
        .map_err(|e| e.to_string())?;

    let stored_invite_ids: Vec<_> = stored_invites
        .iter()
        .map(|invite| invite.event.id.unwrap())
        .collect();

    for invite in stored_invites {
        if invite.state == InviteState::Pending {
            invites.push(invite);
        }
    }

    let fetched_invite_events = wn
        .nostr
        .fetch_user_welcomes(
            wn.nostr
                .client
                .signer()
                .await
                .unwrap()
                .get_public_key()
                .await
                .map_err(|e| e.to_string())?,
        )
        .await
        .map_err(|e| e.to_string())?;

    // We need to track the key_packages that were used in procesing this batch of invites
    // These key_packages need to be deleted from the relays and the private key material needs to be deleted from the MLS database
    let mut used_key_package_ids: Vec<String> = Vec::new();

    let mut failed_invites: Vec<(EventId, String)> = Vec::new();

    for event in fetched_invite_events {
        // Skip if we have already processed this invite
        if stored_invite_ids.contains(&event.id.unwrap()) {
            tracing::debug!(target: "whitenoise::commands::invites::get_invites", "Invite {:?} already processed", event.id.unwrap());
            continue;
        }
        let nostr_mls = wn.nostr_mls.lock().expect("Failed to lock nostr_mls");
        let welcome_preview =
            match nostr_mls.preview_welcome_event(match hex::decode(&event.content) {
                    Ok(content) => content,
                    Err(e) => {
                        tracing::error!(target: "whitenoise::commands::invites::get_invites", "Error decoding welcome event {:?}: {}", event.id.unwrap(), e);
                        failed_invites.push((event.id.unwrap(), e.to_string()));
                        continue;
                    }
                }) {
                Ok(invite) => invite,
                Err(e) => {
                    tracing::error!(target: "whitenoise::commands::invites::get_invites", "Error processing welcome event {:?}: {}", event.id.unwrap(), e);
                    failed_invites.push((event.id.unwrap(), e.to_string()));
                    continue;
                }
            };

        let key_package_event_id = event
            .tags
            .iter()
            .find(|tag| {
                tag.kind() == TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::E))
            })
            .and_then(|tag| tag.content());

        let invite = Invite {
            event: event.clone(),
            mls_group_id: welcome_preview
                .staged_welcome
                .group_context()
                .group_id()
                .to_vec(),
            nostr_group_id: welcome_preview.nostr_group_data.nostr_group_id(),
            group_name: welcome_preview.nostr_group_data.name(),
            group_description: welcome_preview.nostr_group_data.description(),
            group_admin_pubkeys: welcome_preview.nostr_group_data.admin_pubkeys(),
            group_relays: welcome_preview.nostr_group_data.relays(),
            inviter: event.pubkey.to_hex(),
            member_count: welcome_preview.staged_welcome.members().count(),
            state: InviteState::Pending,
        };

        invites.push(invite.clone());
        invite
            .save(&active_account.pubkey, &wn)
            .map_err(|e| e.to_string())?;

        if let Some(key_package_event_id) = key_package_event_id {
            used_key_package_ids.push(key_package_event_id.to_string());
        }

        app_handle
            .emit("invite_processed", invite)
            .map_err(|e| e.to_string())?;
    }

    let key_package_relays: Vec<String> = if cfg!(dev) {
        vec!["ws://localhost:8080".to_string()]
    } else {
        active_account.key_package_relays.clone()
    };

    // Remove used key package ids from relays and from MLS storage
    // We do this in bulk after we've processed all welcome events to avoid deleting
    // the key package material while we're still processing events that might need it.
    used_key_package_ids.sort();
    used_key_package_ids.dedup();

    // TODO: We need to handle cleaning up old key packages from MLS storage on a regular basis
    for key_package_id in &used_key_package_ids {
        tracing::debug!(target: "nostr_mls::invites::fetch_invites_for_user", "Deleting used key package {:?}", key_package_id);
        delete_key_package_from_relays(
            &EventId::parse(key_package_id).unwrap(),
            &key_package_relays,
            false, // For now we don't want to delete the key packages from MLS storage
            &wn,
        )
        .await
        .map_err(|e| format!("Couldn't delete key package {:?}: {}", key_package_id, e))?;
    }

    // Generate and publish new key packages to replace the used key packages
    for _ in used_key_package_ids.iter() {
        publish_key_package(wn.clone()).await?;
    }

    // TODO: We need to filter and only show the latest welcome message for a given group if there are duplicates
    Ok(InvitesWithFailures {
        invites,
        failures: failed_invites,
    })
}

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
pub fn get_invite(
    active_account: String,
    invite_id: String,
    wn: tauri::State<'_, Whitenoise>,
) -> Result<Invite, String> {
    Invite::find_by_id(&active_account, &invite_id, &wn).map_err(|e| e.to_string())
}

/// Accepts a group invite and joins the corresponding group.
///
/// # Arguments
/// * `invite` - The invite to accept
/// * `wn` - The Whitenoise state
/// * `app_handle` - The Tauri app handle
///
/// # Returns
/// * `Ok(())` if the invite was successfully accepted and the group was joined
/// * `Err(String)` if there was an error accepting the invite or joining the group
///
/// # Events Emitted
/// * `group_added` - Emitted with the newly joined group after successful join
/// * `invite_accepted` - Emitted with the updated invite after it is accepted
#[tauri::command]
pub async fn accept_invite(
    mut invite: Invite,
    wn: tauri::State<'_, Whitenoise>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    tracing::debug!(target: "whitenoise::invites::accept_invite", "Accepting invite {:?}", invite.event.id.unwrap());

    let active_account = Account::get_active(&wn).map_err(|e| e.to_string())?;

    // Scope the MutexGuard to drop it before the .await points
    let (mls_group, nostr_group_data) = {
        let nostr_mls = wn.nostr_mls.lock().expect("Failed to lock nostr_mls");
        let joined_group_result = nostr_mls
            .join_group_from_welcome(
                hex::decode(&invite.event.content)
                    .map_err(|e| format!("Error decoding welcome event: {}", e))?,
            )
            .map_err(|e| format!("Error joining group from welcome: {}", e))?;

        (
            joined_group_result.mls_group,
            joined_group_result.nostr_group_data,
        )
    };

    let group_type = match mls_group.members().count() {
        2 => GroupType::DirectMessage,
        _ => GroupType::Group,
    };

    let group = Group::new(
        &active_account.pubkey,
        mls_group.group_id().to_vec(),
        mls_group.epoch().as_u64(),
        group_type,
        nostr_group_data,
        &wn,
        &app_handle,
    )
    .map_err(|e| format!("Failed to add group: {}", e))?;

    // Update the subscription for MLS group messages to include the new group
    let group_ids = active_account
        .groups(&wn)
        .map_err(|e| format!("Failed to get groups: {}", e))?
        .into_iter()
        .map(|group| group.nostr_group_id.clone())
        .collect::<Vec<_>>();

    wn.nostr
        .subscribe_mls_group_messages(group_ids.clone())
        .await
        .map_err(|e| format!("Failed to update MLS group subscription: {}", e))?;

    // Manually fetch for MLS messages for the new group
    wn.nostr
        .fetch_group_messages(Timestamp::zero(), group_ids.clone())
        .await
        .map_err(|e| format!("Failed to fetch group messages: {}", e))?;

    app_handle
        .emit("group_added", group.clone())
        .map_err(|e| e.to_string())?;

    // Update the invite state to accepted
    invite.state = InviteState::Accepted;
    invite
        .save(&active_account.pubkey, &wn)
        .map_err(|e| e.to_string())?;

    app_handle
        .emit("invite_accepted", invite)
        .map_err(|e| e.to_string())?;

    tracing::debug!(target: "whitenoise::invites::accept_invite", "Accepted invite - Added group: {:?}", group);

    Ok(())
}

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
pub fn decline_invite(
    mut invite: Invite,
    wn: tauri::State<'_, Whitenoise>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    tracing::debug!(target: "whitenoise::invites::decline_invite", "Declining invite {:?}", invite.event.id.unwrap());

    let active_account = Account::get_active(&wn).map_err(|e| e.to_string())?;

    invite.state = InviteState::Declined;
    invite
        .save(&active_account.pubkey, &wn)
        .map_err(|e| e.to_string())?;

    app_handle
        .emit("invite_declined", invite)
        .map_err(|e| e.to_string())?;

    Ok(())
}
