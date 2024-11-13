use crate::fetch_enriched_contact;
use crate::groups::{validate_group_members, Group, GroupType};
use crate::key_packages::fetch_key_packages_for_members;
use crate::whitenoise::Whitenoise;
use nostr_sdk::prelude::*;
use std::ops::Add;
use tauri::Emitter;

// This is scoped so that we can return only the groups that the user is a member of.
#[tauri::command]
pub fn get_groups(wn: tauri::State<'_, Whitenoise>) -> Result<Vec<Group>, String> {
    let active_account = wn
        .account_manager
        .get_active_account()
        .map_err(|e| e.to_string())?;

    wn.group_manager
        .get_groups(active_account.mls_group_ids)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_group(group_id: String, wn: tauri::State<'_, Whitenoise>) -> Result<Group, String> {
    wn.group_manager
        .get_group(group_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_group(
    creator_pubkey: String,
    member_pubkeys: Vec<String>,
    admin_pubkeys: Vec<String>,
    group_name: String,
    description: String,
    wn: tauri::State<'_, Whitenoise>,
    app_handle: tauri::AppHandle,
) -> Result<Group, String> {
    let active_account = wn
        .account_manager
        .get_active_account()
        .map_err(|e| e.to_string())?;

    let signer = wn.nostr.client.signer().await.map_err(|e| e.to_string())?;

    // Check that active account is the creator and signer
    if active_account.pubkey != creator_pubkey
        || active_account.pubkey
            != signer
                .get_public_key()
                .await
                .map_err(|e| e.to_string())?
                .to_hex()
    {
        return Err("You must be the creator to create a group".to_string());
    }

    validate_group_members(&creator_pubkey, &member_pubkeys, &admin_pubkeys)
        .map_err(|e| e.to_string())?;

    let member_key_packages = fetch_key_packages_for_members(&member_pubkeys, &wn)
        .await
        .map_err(|e| e.to_string())?;

    tracing::debug!(
        target: "whitenoise::groups::create_group",
        "Member key packages: {:?}",
        member_key_packages
    );

    // TODO: We'll need to set these based on the group creator?
    let group_relays = vec!["ws://localhost:8080".to_string()];

    let create_group_result = wn
        .nostr_mls
        .create_group(
            group_name,
            description,
            member_key_packages,
            admin_pubkeys,
            creator_pubkey,
            group_relays,
        )
        .map_err(|e| e.to_string())?;

    let mls_group = create_group_result.mls_group;
    let serialized_welcome_message = create_group_result.serialized_welcome_message;
    let group_data = create_group_result.nostr_group_data;

    // Fan out the welcome message to all members
    for member in &member_pubkeys {
        let member_pubkey = PublicKey::from_hex(member).map_err(|e| e.to_string())?;
        let contact = fetch_enriched_contact(member.clone(), false, wn.clone(), app_handle.clone())
            .await
            .map_err(|e| e.to_string())?;

        // If we're in dev mode, use the local relay, otherwise use the relays from the contact
        let relay_urls = if tauri::is_dev() {
            vec!["ws://localhost:8080".to_string()]
        } else {
            contact.inbox_relays
        };

        let welcome_rumor = EventBuilder::new(
            Kind::MlsWelcome,
            hex::encode(&serialized_welcome_message),
            vec![Tag::from_standardized_without_cell(TagStandard::Relays(
                relay_urls
                    .iter()
                    .filter_map(|r| Url::parse(r).ok())
                    .collect(),
            ))],
        )
        .build(signer.get_public_key().await.map_err(|e| e.to_string())?);

        tracing::debug!(
            target: "whitenoise::groups::create_group",
            "Welcome rumor: {:?}",
            welcome_rumor
        );

        // Create a timestamp 1 month in the future
        let one_month_future = Timestamp::now().add(30 * 24 * 60 * 60);

        // TODO: We'll probably want to refactor this to be async eventually.
        let wrapped_event = EventBuilder::gift_wrap(
            &signer,
            &member_pubkey,
            welcome_rumor,
            vec![Tag::expiration(one_month_future)],
        )
        .await
        .map_err(|e| e.to_string())?;

        let max_retries = 5;
        let mut retry_count = 0;
        let mut last_error = None;

        while retry_count < max_retries {
            match wn
                .nostr
                .client
                .send_event_to(relay_urls.clone(), wrapped_event.clone())
                .await
            {
                Ok(_) => {
                    // Successfully sent, break the loop
                    break;
                }
                Err(e) => {
                    last_error = Some(e);
                    retry_count += 1;
                    if retry_count < max_retries {
                        // Wait for a short time before retrying
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                }
            }
        }

        if retry_count == max_retries {
            return Err(format!(
                "Failed to send event after {} attempts. Last error: {:?}",
                max_retries, last_error
            ));
        }

        tracing::debug!(
            target: "whitenoise::groups::create_group",
            "Published welcome message to {:?}",
            &member_pubkey
        );
    }

    let group_type = if mls_group.members().count() == 2 {
        GroupType::DirectMessage
    } else {
        GroupType::Group
    };

    let group_id = mls_group.group_id().to_vec();

    let nostr_group = wn
        .group_manager
        .add_group(group_id.clone(), group_type, group_data)
        .map_err(|e| e.to_string())?;

    tracing::debug!(
        target: "whitenoise::groups::create_group",
        "Added group to database: {:?}",
        nostr_group
    );

    app_handle
        .emit("group_added", nostr_group.clone())
        .map_err(|e| e.to_string())?;

    wn.account_manager
        .add_mls_group_id(active_account.pubkey, group_id.clone())
        .map_err(|e| e.to_string())?;

    tracing::debug!(
        target: "whitenoise::groups::create_group",
        "Added MLS group id to account manager: {:?}",
        group_id
    );

    Ok(nostr_group)
}
