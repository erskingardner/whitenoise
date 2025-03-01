use crate::whitenoise::Whitenoise;
use bitcoin_hashes::sha256::Hash as Sha256Hash;
use nostr_sdk::prelude::*;
use std::str::FromStr;

/// Generates a NIP-98 auth token for a given URL and method.
///
/// # Arguments
///
/// * `url` - The URL to generate the auth token for.
/// * `method` - The HTTP method to use for the auth token.
/// * `payload` - The json stringified body of the request to use for the auth token.
///
/// # Returns
///
/// * `Ok(String)` - The auth token.
/// * `Err(String)` - An error message if there was an issue generating the auth token.
#[tauri::command]
pub async fn generate_nip98_auth_token(
    url: String,
    method: String,
    payload: Option<String>,
    wn: tauri::State<'_, Whitenoise>,
) -> Result<String, String> {
    let signer = wn.nostr.client.signer().await.map_err(|e| e.to_string())?;

    let url = Url::parse(&url).map_err(|e| e.to_string())?;
    let method = HttpMethod::from_str(&method.to_uppercase()).map_err(|e| e.to_string())?;
    let mut http_data = HttpData::new(url.clone(), method.clone());
    if let Some(payload) = payload {
        let hash: Sha256Hash = Sha256Hash::from_str(&payload).map_err(|e| e.to_string())?;
        http_data = http_data.payload(hash);
    }
    let event = EventBuilder::http_auth(http_data)
        .sign(&signer)
        .await
        .map_err(|e| e.to_string())?;

    tracing::debug!(
        target: "whitenoise::commands::nostr::generate_nip98_auth_token",
        "Generated auth token for URL: {}, method: {}, event: {:?}",
        url,
        method,
        event
    );

    Ok(event.as_json())
}
