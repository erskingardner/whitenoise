use crate::commands::key_packages::publish_key_package;
use crate::group_manager::{Invite, InviteState};
use crate::key_packages::delete_key_package_from_relays;
use crate::whitenoise::Whitenoise;
use nostr_sdk::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct InvitesWithFailures {
    invites: Vec<Invite>,
    failures: Vec<(EventId, String)>,
}

#[tauri::command]
pub async fn get_invites(wn: tauri::State<'_, Whitenoise>) -> Result<InvitesWithFailures, String> {
    let active_account = wn
        .account_manager
        .get_active_account()
        .map_err(|e| e.to_string())?;

    let mut invites: Vec<Invite> = Vec::new();

    let stored_invites = wn
        .group_manager
        .get_invites(active_account.mls_group_ids)
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
            tracing::debug!(target: "nostr_mls::invites::fetch_invites_for_user", "Invite {:?} already processed", event.id.unwrap());
            continue;
        }

        let processed_invite =
            match wn
                .nostr_mls
                .process_welcome(match hex::decode(&event.content) {
                    Ok(content) => content,
                    Err(e) => {
                        tracing::error!(target: "nostr_mls::invites::fetch_invites_for_user", "Error decoding welcome event {:?}: {}", event.id.unwrap(), e);
                        failed_invites.push((event.id.unwrap(), e.to_string()));
                        continue;
                    }
                }) {
                Ok(invite) => invite,
                Err(e) => {
                    tracing::error!(target: "nostr_mls::invites::fetch_invites_for_user", "Error processing welcome event {:?}: {}", event.id.unwrap(), e);
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
            mls_group_id: processed_invite.mls_group_id,
            nostr_group_id: processed_invite.nostr_group_data.nostr_group_id(),
            group_name: processed_invite.nostr_group_data.name(),
            group_description: processed_invite.nostr_group_data.description(),
            group_admin_pubkeys: processed_invite.nostr_group_data.admin_pubkeys(),
            group_relays: processed_invite.nostr_group_data.relays(),
            inviter: event.pubkey.to_hex(),
            member_count: processed_invite.member_count,
            state: InviteState::Pending,
        };

        invites.push(invite);
        if let Some(key_package_event_id) = key_package_event_id {
            used_key_package_ids.push(key_package_event_id.to_string());
        }
    }

    // Remove used key package ids from relays and from MLS storage
    // We do this in bulk after we've processed all welcome events to avoid deleting
    // the key package material while we're still processing events that might need it.
    used_key_package_ids.sort();
    used_key_package_ids.dedup();
    for key_package_id in &used_key_package_ids {
        tracing::debug!(target: "nostr_mls::invites::fetch_invites_for_user", "Deleting used key package {:?}", key_package_id);
        delete_key_package_from_relays(
            &EventId::parse(key_package_id).unwrap(),
            &active_account.key_package_relays,
            true,
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

#[tauri::command]
pub fn get_invite(invite_id: String, wn: tauri::State<'_, Whitenoise>) -> Result<Invite, String> {
    wn.group_manager
        .get_invite(invite_id)
        .map_err(|e| e.to_string())
}
