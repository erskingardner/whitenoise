use crate::messages::Message;
use crate::whitenoise::Whitenoise;
use nostr_sdk::prelude::*;

#[tauri::command]
pub async fn query_message(
    message_id: &str,
    wn: tauri::State<'_, Whitenoise>,
) -> Result<UnsignedEvent, String> {
    let message = Message::find_by_event_id(
        EventId::parse(message_id).map_err(|e| e.to_string())?,
        wn.clone(),
    )
    .await
    .map_err(|e| format!("Error fetching message: {}", e))?;

    Ok(message.event)
}
