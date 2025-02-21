use crate::accounts::Account;
use crate::fetch_enriched_contact;
use crate::groups::{Group, GroupType, GroupWithRelays};
use crate::key_packages::fetch_key_packages_for_members;
use crate::secrets_store;
use crate::whitenoise::Whitenoise;
use lightning_invoice::SignedRawBolt11Invoice;
use nostr_sdk::prelude::*;
use nostr_sdk::NostrSigner;
use std::ops::Add;
use std::str::FromStr;
use std::sync::Arc;
use tauri::Emitter;

/// Gets all MLS groups that the active account is a member of
/// This is scoped so that we can return only the groups that the user is a member of.
///
/// # Arguments
/// * `wn` - Whitenoise state containing account and group managers
///
/// # Returns
/// * `Ok(Vec<Group>)` - List of groups the active account belongs to
/// * `Err(String)` - Error message if retrieval fails
///
/// # Errors
/// Returns error if:
/// - No active account found
/// - Database error occurs retrieving groups
#[tauri::command]
pub async fn get_groups(wn: tauri::State<'_, Whitenoise>) -> Result<Vec<Group>, String> {
    Group::get_all_groups(wn.clone())
        .await
        .map_err(|e| format!("Error fetching groups for account: {}", e))
}

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

/// Gets a single MLS group and its messages by group ID
///
/// # Arguments
/// * `group_id` - Hex encoded MLS group ID
/// * `wn` - Whitenoise state
///
/// # Returns
/// * `Ok((Group, Vec<UnsignedEvent>))` - Tuple containing:
///   - The requested group if found
///   - Vector of unsigned message events for the group
/// * `Err(String)` - Error message if operation fails
///
/// # Errors
/// Returns error if:
/// - No active account found
/// - Group ID is not valid hex
/// - Group not found in database
/// - Error fetching messages
#[tauri::command]
pub async fn get_group_and_messages(
    group_id: &str,
    wn: tauri::State<'_, Whitenoise>,
) -> Result<(Group, Vec<UnsignedEvent>), String> {
    let mls_group_id =
        hex::decode(group_id).map_err(|e| format!("Error decoding group id: {}", e))?;
    tracing::debug!(
        target: "whitenoise::commands::groups::get_group_and_messages",
        "Getting group and messages for group ID: {:?}",
        mls_group_id
    );
    let group = Group::find_by_mls_group_id(&mls_group_id, wn.clone())
        .await
        .map_err(|e| format!("Error fetching group: {}", e))?;
    tracing::debug!(
        target: "whitenoise::commands::groups::get_group_and_messages",
        "Group: {:?}",
        group
    );
    let messages = group
        .messages(wn.clone())
        .await
        .map_err(|e| format!("Error fetching messages: {}", e))?;
    tracing::debug!(
        target: "whitenoise::commands::groups::get_group_and_messages",
        "Messages: {:?}",
        messages
    );
    Ok((group, messages))
}

/// Creates a new MLS group with the specified members and settings
///
/// # Arguments
/// * `creator_pubkey` - Public key of the group creator (must be the active account)
/// * `member_pubkeys` - List of public keys for group members
/// * `admin_pubkeys` - List of public keys for group admins
/// * `group_name` - Name of the group
/// * `description` - Description of the group
/// * `wn` - Whitenoise state
/// * `app_handle` - Tauri app handle
///
/// # Returns
/// * `Ok(Group)` - The newly created group
/// * `Err(String)` - Error message if group creation fails
///
/// # Flow
/// 1. Validates that active account is the creator and signer
/// 2. Validates member and admin lists
/// 3. Fetches key packages for all members
/// 4. Creates MLS group with NostrMls
/// 5. Sends welcome messages to all members via Nostr
/// 6. Adds group to GroupManager database
/// 7. Updates account with new group ID
/// 8. Emits group_added event
///
/// # Errors
/// Returns error if:
/// - Active account is not the creator
/// - Member/admin validation fails
/// - Key package fetching fails
/// - MLS group creation fails
/// - Welcome message sending fails
/// - Database operations fail
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
    let active_account = Account::get_active(wn.clone())
        .await
        .map_err(|e| e.to_string())?;
    let signer = wn.nostr.client.signer().await.map_err(|e| e.to_string())?;

    // Check that active account is the creator and signer
    if active_account.pubkey.to_hex() != creator_pubkey
        || active_account.pubkey.to_hex()
            != signer
                .get_public_key()
                .await
                .map_err(|e| e.to_string())?
                .to_hex()
    {
        return Err("You cannot create a group for another account".to_string());
    }

    // Run various checks on the group members
    Group::validate_group_members(&creator_pubkey, &member_pubkeys, &admin_pubkeys)
        .map_err(|e| e.to_string())?;

    // Fetch key packages for all members
    let member_key_packages = fetch_key_packages_for_members(&member_pubkeys, wn.clone())
        .await
        .map_err(|e| e.to_string())?;

    tracing::debug!(
        target: "whitenoise::groups::create_group",
        "Member key packages: {:?}",
        member_key_packages
    );

    // TODO: Add ability to specify relays for the group
    let group_relays = wn.nostr.relays().await.unwrap();

    let create_group_result;
    {
        let nostr_mls = wn.nostr_mls.lock().await;

        create_group_result = nostr_mls
            .create_group(
                group_name,
                description,
                member_key_packages
                    .iter()
                    .map(|kp| kp.key_package.clone())
                    .collect(),
                admin_pubkeys,
                creator_pubkey,
                group_relays,
            )
            .map_err(|e| e.to_string())?;
    }

    let mls_group = create_group_result.mls_group;
    let serialized_welcome_message = create_group_result.serialized_welcome_message;
    let group_data = create_group_result.nostr_group_data;

    // Fan out the welcome message to all members
    for member in member_key_packages {
        let member_pubkey = PublicKey::from_hex(&member.pubkey).map_err(|e| e.to_string())?;
        let contact =
            fetch_enriched_contact(member.pubkey.clone(), false, wn.clone(), app_handle.clone())
                .await
                .map_err(|e| e.to_string())?;

        // We only want to connect to user relays in release mode
        let relay_urls: Vec<String> = if cfg!(dev) {
            vec!["ws://localhost:8080".to_string()]
        } else if !contact.inbox_relays.is_empty() {
            contact.inbox_relays
        } else if !contact.nostr_relays.is_empty() {
            contact.nostr_relays
        } else {
            // Get default relays from the client
            wn.nostr
                .client
                .relays()
                .await
                .keys()
                .map(|url| url.to_string())
                .collect()
        };

        let welcome_rumor =
            EventBuilder::new(Kind::MlsWelcome, hex::encode(&serialized_welcome_message)).tags(
                vec![
                    Tag::from_standardized(TagStandard::Relays(
                        relay_urls
                            .iter()
                            .filter_map(|r| Url::parse(r).ok())
                            .collect(),
                    )),
                    Tag::event(member.event_id),
                ],
            );

        tracing::debug!(
            target: "whitenoise::groups::create_group",
            "Welcome rumor: {:?}",
            welcome_rumor
        );

        // Create a timestamp 1 month in the future
        let one_month_future = Timestamp::now().add(30 * 24 * 60 * 60);

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

        let mut relays_to_remove: Vec<String> = Vec::new();

        for url in relay_urls.clone() {
            let to_remove = wn
                .nostr
                .client
                .add_relay(url.clone())
                .await
                .map_err(|e| e.to_string())?;
            if to_remove {
                relays_to_remove.push(url);
            }
        }

        while retry_count < max_retries {
            match wn
                .nostr
                .client
                .send_event_to(relay_urls.clone(), wrapped_event.clone())
                .await
            {
                Ok(result) => {
                    // Successfully sent, break the loop
                    // TODO: Remove the identifying info from the log
                    tracing::info!(
                        target: "whitenoise::groups::create_group",
                        "Sent welcome message RESULT: {:?}",
                        result
                    );
                    tracing::info!(
                        target: "whitenoise::groups::create_group",
                        "Successfully sent welcome message {:?} to {:?} on {:?}",
                        wrapped_event,
                        &member_pubkey,
                        &relay_urls
                    );
                    break;
                }
                Err(e) => {
                    tracing::error!(
                        target: "whitenoise::groups::create_group",
                        "Failed to send welcome message to {:?} on {:?}: {:?}",
                        &member_pubkey,
                        &relay_urls,
                        e
                    );
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
                "Failed to send welcome message to {:?} on {:?} after {} attempts. Last error: {:?}",
                &member_pubkey, &relay_urls, max_retries, last_error
            ));
        }

        tracing::debug!(
            target: "whitenoise::groups::create_group",
            "Published welcome message to {:?} on {:?}: ID: {:?}",
            &member_pubkey,
            &relay_urls,
            wrapped_event.id
        );

        for url in relays_to_remove {
            wn.nostr
                .client
                .remove_relay(url)
                .await
                .map_err(|e| e.to_string())?;
        }
    }

    let group_type = if mls_group.members().count() == 2 {
        GroupType::DirectMessage
    } else {
        GroupType::Group
    };

    let group_id = mls_group.group_id().to_vec();

    // Create the group and save it to the database
    let nostr_group = Group::new(
        group_id.clone(),
        mls_group.epoch().as_u64(),
        group_type,
        group_data,
        wn.clone(),
        &app_handle,
    )
    .await
    .map_err(|e| e.to_string())?;

    tracing::debug!(
        target: "whitenoise::groups::create_group",
        "Added group to database: {:?}",
        nostr_group
    );

    // Update the subscription for MLS group messages to include the new group
    let group_ids = active_account
        .groups(wn.clone())
        .await
        .map_err(|e| format!("Failed to get groups: {}", e))?
        .into_iter()
        .map(|group| group.nostr_group_id.clone())
        .collect::<Vec<_>>();

    wn.nostr
        .subscribe_mls_group_messages(group_ids.clone())
        .await
        .map_err(|e| format!("Failed to update MLS group subscription: {}", e))?;

    app_handle
        .emit("group_added", nostr_group.clone())
        .map_err(|e| e.to_string())?;

    Ok(nostr_group)
}

#[tauri::command]
pub async fn send_mls_message(
    group: Group,
    message: String,
    kind: u16,
    tags: Option<Vec<Tag>>,
    wn: tauri::State<'_, Whitenoise>,
    app_handle: tauri::AppHandle,
) -> Result<UnsignedEvent, String> {
    let nostr_keys = wn.nostr.client.signer().await.map_err(|e| e.to_string())?;

    let inner_event = create_unsigned_nostr_event(&nostr_keys, message, kind, tags)
        .await
        .map_err(|e| e.to_string())?;

    let json_event_string = serde_json::to_string(&inner_event).map_err(|e| e.to_string())?;

    let serialized_message;
    let export_secret_hex;
    let epoch;
    {
        let nostr_mls = wn.nostr_mls.lock().await;
        serialized_message = nostr_mls
            .create_message_for_group(group.mls_group_id.clone(), json_event_string)
            .map_err(|e| e.to_string())?;

        (export_secret_hex, epoch) = nostr_mls
            .export_secret_as_hex_secret_key_and_epoch(group.mls_group_id.clone())
            .map_err(|e| e.to_string())?;
    }

    // Store the export secret key in the secrets store
    secrets_store::store_mls_export_secret(
        group.mls_group_id.clone(),
        epoch,
        export_secret_hex.clone(),
        wn.data_dir.as_path(),
    )
    .map_err(|e| e.to_string())?;

    let export_nostr_keys = Keys::parse(&export_secret_hex).map_err(|e| e.to_string())?;

    let encrypted_content = nip44::encrypt(
        export_nostr_keys.secret_key(),
        &export_nostr_keys.public_key(),
        &serialized_message,
        nip44::Version::V2,
    )
    .map_err(|e| e.to_string())?;

    let ephemeral_nostr_keys = Keys::generate();

    let published_message_event = EventBuilder::new(Kind::MlsGroupMessage, encrypted_content)
        .tags(vec![Tag::custom(
            TagKind::h(),
            vec![group.nostr_group_id.clone()],
        )])
        .sign(&ephemeral_nostr_keys)
        .await
        .map_err(|e| e.to_string())?;

    tracing::debug!(
        target: "whitenoise::commands::groups::send_mls_message",
        "Publishing MLSMessage event to group relays"
    );

    let relays = group.relays(wn.clone()).await.map_err(|e| e.to_string())?;
    let outer_event_id = wn
        .nostr
        .client
        .send_event_to(relays, published_message_event)
        .await
        .map_err(|e| e.to_string())?;

    group
        .add_message(
            outer_event_id.id().to_string(),
            inner_event.clone(),
            wn.clone(),
            app_handle.clone(),
        )
        .await
        .map_err(|e| e.to_string())?;

    app_handle
        .emit("mls_message_sent", (group.clone(), inner_event.clone()))
        .expect("Couldn't emit event");

    Ok(inner_event)
}

/// Creates an unsigned nostr event with the given parameters
async fn create_unsigned_nostr_event(
    nostr_keys: &Arc<dyn NostrSigner>,
    message: String,
    kind: u16,
    tags: Option<Vec<Tag>>,
) -> Result<UnsignedEvent, Error> {
    let mut final_tags = tags.unwrap_or_default();
    final_tags.extend(bolt11_invoice_tags(&message));

    let mut inner_event = UnsignedEvent::new(
        nostr_keys.get_public_key().await?,
        Timestamp::now(),
        kind.into(),
        final_tags,
        message,
    );
    inner_event.ensure_id();
    Ok(inner_event)
}

/// Parses a message for BOLT11 invoices and returns corresponding tags
fn bolt11_invoice_tags(message: &str) -> Vec<Tag> {
    let mut tags = Vec::new();

    // Bitcoin network prefixes according to BOLT-11 spec
    const NETWORK_PREFIXES: [&str; 4] = ["lnbc", "lntb", "lntbs", "lnbcrt"];

    // Check if message contains what looks like a bolt11 invoice
    if let Some(word) = message
        .split_whitespace()
        .find(|w| {
            let w_lower = w.to_lowercase();
            NETWORK_PREFIXES.iter().any(|prefix| w_lower.starts_with(prefix))
        })
    {
        // Try to parse as BOLT11 invoice
        if let Ok(invoice) = SignedRawBolt11Invoice::from_str(word) {
            let amount_msats = invoice
                .raw_invoice()
                .amount_pico_btc()
                .map(|pico_btc| (pico_btc as f64 * 0.1) as u64);

            if let Some(msats) = amount_msats {
                tags.push(Tag::custom(
                    TagKind::from("bolt11"),
                    vec![word.to_string(), msats.to_string()],
                ));
            }
        }
    }

    tags
}

/// Gets the list of members in an MLS group
///
/// # Arguments
/// * `group_id` - Hex-encoded MLS group ID
/// * `wn` - Whitenoise state handle
///
/// # Returns
/// * `Ok(Vec<String>)` - List of member public keys if successful
/// * `Err(String)` - Error message if operation fails
///
/// # Errors
/// * If no active account is found
/// * If group ID cannot be decoded from hex
/// * If group cannot be found
/// * If members cannot be retrieved
#[tauri::command]
pub async fn get_group_members(
    group_id: &str,
    wn: tauri::State<'_, Whitenoise>,
) -> Result<Vec<PublicKey>, String> {
    let mls_group_id =
        hex::decode(group_id).map_err(|e| format!("Error decoding group id: {}", e))?;
    let group = Group::find_by_mls_group_id(&mls_group_id, wn.clone())
        .await
        .map_err(|e| format!("Error fetching group: {}", e))?;
    let members = group.members(wn.clone()).await.map_err(|e| e.to_string())?;
    Ok(members)
}

/// Gets the list of admin members in an MLS group
///
/// # Arguments
/// * `group_id` - Hex-encoded MLS group ID
/// * `wn` - Whitenoise state handle
///
/// # Returns
/// * `Ok(Vec<String>)` - List of admin public keys if successful
/// * `Err(String)` - Error message if operation fails
///
/// # Errors
/// * If no active account is found
/// * If group ID cannot be decoded from hex
/// * If group cannot be found
/// * If admin list cannot be retrieved
#[tauri::command]
pub async fn get_group_admins(
    group_id: &str,
    wn: tauri::State<'_, Whitenoise>,
) -> Result<Vec<PublicKey>, String> {
    let mls_group_id =
        hex::decode(group_id).map_err(|e| format!("Error decoding group id: {}", e))?;
    let group = Group::find_by_mls_group_id(&mls_group_id, wn.clone())
        .await
        .map_err(|e| format!("Error fetching group: {}", e))?;
    let admins = group.admins().map_err(|e| e.to_string())?;
    Ok(admins)
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_unsigned_nostr_event_basic() {
        let keys =
            Keys::from_str("nsec1d4ed5x49d7p24xn63flj4985dc4gpfngdhtqcxpth0ywhm6czxcs5l2exj")
                .unwrap();
        let signer: Arc<dyn NostrSigner> = Arc::new(keys.clone());
        let message = "Stay humble & stack sats!".to_string();
        let kind = 1;
        let tags = None;

        let result = create_unsigned_nostr_event(&signer, message.clone(), kind, tags).await;

        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.content, message);
        assert!(event.tags.is_empty());
        assert_eq!(event.kind, kind.into());
        assert_eq!(event.pubkey, keys.public_key());
    }

    #[tokio::test]
    async fn test_create_unsigned_nostr_event_with_tags() {
        let keys =
            Keys::from_str("nsec1d4ed5x49d7p24xn63flj4985dc4gpfngdhtqcxpth0ywhm6czxcs5l2exj")
                .unwrap();
        let signer: Arc<dyn NostrSigner> = Arc::new(keys.clone());
        let message = "Stay humble & stack sats!".to_string();
        let kind = 1;
        let tags = Some(vec![Tag::reference("test_id")]);

        let result =
            create_unsigned_nostr_event(&signer, message.clone(), kind, tags.clone()).await;

        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.content, message);
        assert_eq!(event.tags.to_vec(), tags.unwrap());
        assert_eq!(event.kind, kind.into());
        assert_eq!(event.pubkey, keys.public_key());
    }

    #[tokio::test]
    async fn test_create_unsigned_nostr_event_with_bolt11() {
        let keys =
            Keys::from_str("nsec1d4ed5x49d7p24xn63flj4985dc4gpfngdhtqcxpth0ywhm6czxcs5l2exj")
                .unwrap();
        let signer: Arc<dyn NostrSigner> = Arc::new(keys.clone());

        // Test case 1: Message with invoice and existing tags
        let invoice = "lnbc15u1p3xnhl2pp5jptserfk3zk4qy42tlucycrfwxhydvlemu9pqr93tuzlv9cc7g3sdqsvfhkcap3xyhx7un8cqzpgxqzjcsp5f8c52y2stc300gl6s4xswtjpc37hrnnr3c9wvtgjfuvqmpm35evq9qyyssqy4lgd8tj637qcjp05rdpxxykjenthxftej7a2zzmwrmrl70fyj9hvj0rewhzj7jfyuwkwcg9g2jpwtk3wkjtwnkdks84hsnu8xps5vsq4gj5hs";
        let message: String = "Please pay me here: ".to_string() + &invoice;
        let existing_tag = Tag::reference("test_id");
        let result = create_unsigned_nostr_event(
            &signer,
            message,
            1,
            Some(vec![existing_tag.clone()]),
        )
        .await;

        assert!(result.is_ok());
        let event = result.unwrap();
        let tags_vec = event.tags.to_vec();

        // Check that original tag is preserved
        assert!(tags_vec.contains(&existing_tag));

        // Check bolt11 tag content
        let bolt11_tags: Vec<_> = tags_vec
            .iter()
            .filter(|tag| *tag != &existing_tag)
            .collect();
        assert_eq!(bolt11_tags.len(), 1);

        let tag = &bolt11_tags[0];
        let content = (*tag).clone().to_vec();
        assert_eq!(content[0], "bolt11");
        assert_eq!(content[1], invoice);
        assert!(!content[2].is_empty());

        // Test case 2: Regular message with tags
        let result = create_unsigned_nostr_event(
            &signer,
            "Just a regular message".to_string(),
            1,
            Some(vec![existing_tag.clone()]),
        )
        .await;

        assert!(result.is_ok());
        let event = result.unwrap();
        let tags_vec = event.tags.to_vec();
        assert!(tags_vec.contains(&existing_tag));
        assert_eq!(tags_vec.len(), 1); // Only the existing tag, no bolt11 tag

        // Test case 3: Invalid invoice
        let result = create_unsigned_nostr_event(
            &signer,
            "lnbc1invalid".to_string(),
            1,
            Some(vec![existing_tag.clone()]),
        )
        .await;

        assert!(result.is_ok());
        let event = result.unwrap();
        let tags_vec = event.tags.to_vec();
        assert!(tags_vec.contains(&existing_tag));
        assert_eq!(tags_vec.len(), 1); // Only the existing tag, no bolt11 tag
    }

    #[tokio::test]
    async fn test_create_unsigned_nostr_event_with_bolt11_networks() {
        let keys = Keys::from_str("nsec1d4ed5x49d7p24xn63flj4985dc4gpfngdhtqcxpth0ywhm6czxcs5l2exj").unwrap();
        let signer: Arc<dyn NostrSigner> = Arc::new(keys.clone());
        let existing_tag = Tag::reference("test_id");

        // Test cases for different network prefixes
        let test_cases = vec![
            // Mainnet invoice (lnbc)
            "lnbc15u1p3xnhl2pp5jptserfk3zk4qy42tlucycrfwxhydvlemu9pqr93tuzlv9cc7g3sdqsvfhkcap3xyhx7un8cqzpgxqzjcsp5f8c52y2stc300gl6s4xswtjpc37hrnnr3c9wvtgjfuvqmpm35evq9qyyssqy4lgd8tj637qcjp05rdpxxykjenthxftej7a2zzmwrmrl70fyj9hvj0rewhzj7jfyuwkwcg9g2jpwtk3wkjtwnkdks84hsnu8xps5vsq4gj5hs",
            // Testnet invoice (lntb)
            "lntb20m1pvjluezsp5zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zygshp58yjmdan79s6qqdhdzgynm4zwqd5d7xmw5fk98klysy043l2ahrqspp5qqqsyqcyq5rqwzqfqqqsyqcyq5rqwzqfqqqsyqcyq5rqwzqfqypqfpp3x9et2e20v6pu37c5d9vax37wxq72un989qrsgqdj545axuxtnfemtpwkc45hx9d2ft7x04mt8q7y6t0k2dge9e7h8kpy9p34ytyslj3yu569aalz2xdk8xkd7ltxqld94u8h2esmsmacgpghe9k8",
            // Signet invoice (lntbs)
            "lntbs4320n1pnm35s8dqqnp4qg62h96f9rsq0fwq0wff6q2444j8ylp7984srtvxtdth8mmw008qgpp5uad7pp9cjtvde5l67dtakznj9x3fd4qggmeg4z6j5za6zxz0areqsp5dgdv4ugpfsgqmp7vuxpq5s06jxaesg9e7hu32ffjdc2va6cwpt4s9qyysgqcqpcxqyz5vqn94eujdlwdtjxqzu9tycyujzgwsq6xnjw3ycpqfvzk6dl3pk2wrjyja4645xftw7x4m4h9jl3wugczsdn9jeyhv75g63nk83y2848zqpsdqdx7",
            // Regtest invoice (lnbcrt)
            "lnbcrt12340n1pnm35h8pp5dz8c9ytfv0s6h97vp0mwdhmxm4c9jn5wjnyeez9th06t5lag6q4qdqqcqzzsxqyz5vqsp5v6jg8wrl37s6ggf0sc2jd0g6a2axnemyet227ckfwlxgrykclw8s9qxpqysgqy6966qlpgc2frw5307wy2a9f966ksv2f8zx6tatcmdcqpwxn9vp3m9s6eg4cewuprn0wljs3vkfs5cny5nq3n8slme2lvfxf70pzdlsqztw8hc",
        ];

        for invoice in test_cases {
            let message = format!("Please pay me here: {}", invoice);
            let result = create_unsigned_nostr_event(
                &signer,
                message,
                1,
                Some(vec![existing_tag.clone()]),
            )
            .await;

            assert!(result.is_ok());
            let event = result.unwrap();
            let tags_vec = event.tags.to_vec();

            // Check that original tag is preserved
            assert!(tags_vec.contains(&existing_tag));

            // Check bolt11 tag content
            let bolt11_tags: Vec<_> = tags_vec
                .iter()
                .filter(|tag| *tag != &existing_tag)
                .collect();
            assert_eq!(bolt11_tags.len(), 1);

            let tag = &bolt11_tags[0];
            let content = (*tag).clone().to_vec();
            assert_eq!(content[0], "bolt11");
            assert_eq!(content[1], invoice);
            assert!(!content[2].is_empty());
        }
    }
}
