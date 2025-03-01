use crate::types::NostrEncryptionMethod;
use crate::whitenoise::Whitenoise;
use nostr_sdk::prelude::*;

#[tauri::command]
pub async fn encrypt_content(
    content: String,
    pubkey: String,
    method: NostrEncryptionMethod,
    wn: tauri::State<'_, Whitenoise>,
) -> Result<String, String> {
    wn.nostr
        .encrypt_content(content, pubkey, method)
        .await
        .map_err(|e| e.to_string())
}
