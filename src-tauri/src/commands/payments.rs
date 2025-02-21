use crate::accounts::Account;
use crate::commands::groups::send_mls_message;
use crate::groups::Group;
use crate::payments::{self, PaymentError};
use crate::whitenoise::Whitenoise;
use nostr_sdk::prelude::*;
use serde::Serialize;

#[derive(Debug, thiserror::Error, Serialize)]
pub enum CommandError {
    #[error("Account error: No active account found")]
    NoActiveAccount,
    #[error("Account error: No NWC URI configured")]
    NoNWCUri,
    #[error("Message error: Can not build message")]
    MessageError,
    #[error("Payment error: {0}")]
    PaymentError(String),
}

impl From<PaymentError> for CommandError {
    fn from(err: PaymentError) -> Self {
        CommandError::PaymentError(err.to_string())
    }
}

#[tauri::command]
pub async fn pay_invoice(
    group: Group,
    tags: Option<Vec<Tag>>,
    bolt11: String,
    wn: tauri::State<'_, Whitenoise>,
    app_handle: tauri::AppHandle,
) -> Result<UnsignedEvent, CommandError> {
    let active_account = Account::get_active(wn.clone())
        .await
        .map_err(|_| CommandError::NoActiveAccount)?;

    let nwc_uri = active_account
        .get_nostr_wallet_connect_uri(wn.clone())
        .map_err(|_| CommandError::NoNWCUri)?
        .ok_or(CommandError::NoNWCUri)?;

    let preimage = payments::pay_bolt11_invoice(&bolt11, &nwc_uri)
        .await
        .map_err(CommandError::from)?;
    let message = "⚡".to_string();
    let kind = 7;
    let mut final_tags = tags.unwrap_or_default();
    final_tags.push(Tag::custom(
        TagKind::Custom("preimage".into()),
        vec![preimage.to_string()],
    ));
    let final_tags = Some(final_tags);
    let unsigned_message = send_mls_message(group, message, kind, final_tags, wn, app_handle)
        .await
        .map_err(|_| CommandError::MessageError)?;
    Ok(unsigned_message)
}
