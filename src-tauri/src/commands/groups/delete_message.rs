use crate::accounts::Account;
use crate::groups::Group;
use crate::send_mls_message;
use crate::whitenoise::Whitenoise;
use nostr_sdk::prelude::*;

/// Deletes a message from an MLS group by creating and sending a deletion event
///
/// Creates a kind 5 (deletion) event with an "e" tag referencing the message
/// to be deleted, as specified in NIP-09.
///
/// # Arguments
/// * `group` - The MLS group containing the message
/// * `message_id` - ID of the message to delete (hex-encoded string)
/// * `wn` - Whitenoise state handle
/// * `app_handle` - Tauri app handle
///
/// # Returns
/// * `Ok(UnsignedEvent)` - The deletion event if successful
/// * `Err(String)` - Error message if deletion fails
///
/// # Errors
/// Returns error if:
/// * Message ID cannot be parsed as a valid EventId
/// * No active account is found
/// * Message cannot be found in the group
/// * User is not the owner of the message
/// * Sending the deletion event fails
#[tauri::command]
pub async fn delete_message(
    group: Group,
    message_id: String,
    wn: tauri::State<'_, Whitenoise>,
    app_handle: tauri::AppHandle,
) -> Result<UnsignedEvent, String> {
    tracing::debug!(
        target: "whitenoise::commands::groups::delete_message",
        "Attempting to delete message with ID: {} from group: {}",
        message_id,
        hex::encode(&group.mls_group_id)
    );

    // Validate inputs and permissions
    let (message_event_id, active_account) =
        validate_deletion_request(&message_id, &group, wn.clone()).await?;

    // Create deletion event with "e" tag (NIP-09)
    let deletion_tags = vec![Tag::event(message_event_id)];
    let deletion_reason = "Message deleted by user";

    tracing::debug!(
        target: "whitenoise::commands::groups::delete_message",
        "Creating deletion event for message ID: {}, from user: {}",
        message_id,
        active_account.pubkey.to_hex()
    );

    // Send the deletion event
    let result = send_mls_message(
        group,
        deletion_reason.to_string(),
        5, // Kind 5 for deletion events as per NIP-09
        Some(deletion_tags),
        wn,
        app_handle,
    )
    .await
    .map_err(|e| format!("Failed to send deletion event: {}", e));

    match &result {
        Ok(event) => {
            let id_str = match &event.id {
                Some(id) => id.to_hex(),
                None => "unknown".to_string(),
            };
            tracing::debug!(
                target: "whitenoise::commands::groups::delete_message",
                "Successfully created deletion event with ID: {}",
                id_str
            )
        }
        Err(e) => tracing::error!(
            target: "whitenoise::commands::groups::delete_message",
            "Failed to delete message: {}",
            e
        ),
    }

    result
}

/// Validates a message deletion request
///
/// # Arguments
/// * `message_id` - Hex-encoded message ID
/// * `group` - Group containing the message
/// * `wn` - Whitenoise state
///
/// # Returns
/// * `Ok((EventId, Account))` - Validated message ID and active account
/// * `Err(String)` - Error message if validation fails
async fn validate_deletion_request(
    message_id: &str,
    group: &Group,
    wn: tauri::State<'_, Whitenoise>,
) -> Result<(EventId, Account), String> {
    tracing::debug!(
        target: "whitenoise::commands::groups::validate_deletion_request",
        "Validating deletion request for message ID: {} in group: {}",
        message_id,
        hex::encode(&group.mls_group_id)
    );

    // Parse and validate message ID
    let message_event_id =
        EventId::from_hex(message_id).map_err(|e| format!("Invalid message ID format: {}", e))?;

    // Get and validate active account
    let active_account = Account::get_active(wn.clone())
        .await
        .map_err(|e| format!("Failed to get active account: {}", e))?;

    tracing::debug!(
        target: "whitenoise::commands::groups::validate_deletion_request",
        "Active account: {}, attempting to delete message: {}",
        active_account.pubkey.to_hex(),
        message_id
    );

    // Fetch messages and verify message exists in this group
    let messages = group
        .messages(wn.clone())
        .await
        .map_err(|e| format!("Failed to fetch messages: {}", e))?;

    // Find the target message
    let message = messages
        .iter()
        .find(|m| m.id == Some(message_event_id))
        .ok_or_else(|| format!("Message with ID {} not found in this group", message_id))?;

    // Verify ownership
    if message.pubkey != active_account.pubkey {
        tracing::warn!(
            target: "whitenoise::commands::groups::validate_deletion_request",
            "Permission denied: User {} attempted to delete message {} created by {}",
            active_account.pubkey.to_hex(),
            message_id,
            message.pubkey.to_hex()
        );
        return Err(format!(
            "Permission denied: Cannot delete message {}. Only the message creator can delete it.",
            message_id
        ));
    }

    tracing::debug!(
        target: "whitenoise::commands::groups::validate_deletion_request",
        "Validation successful for message: {}",
        message_id
    );

    Ok((message_event_id, active_account))
}
