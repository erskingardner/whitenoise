use crate::whitenoise::Whitenoise;
use nostr_sdk::prelude::*;

#[tauri::command]
pub async fn publish_metadata(
    metadata: Metadata,
    wn: tauri::State<'_, Whitenoise>,
) -> Result<(), String> {
    let event = EventBuilder::metadata(&metadata);

    tracing::debug!(
        target: "whitenoise::commands::nostr::publish_metadata",
        "Sending event: {:?}",
        event
    );
    wn.nostr
        .client
        .send_event_builder(event)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
