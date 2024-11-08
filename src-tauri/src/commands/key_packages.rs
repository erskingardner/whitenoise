use crate::{key_packages::fetch_key_package_for_pubkey, Whitenoise};
use nostr_sdk::event::{EventBuilder, Kind, Tag, TagKind};
use openmls_nostr::key_packages::create_key_package_for_event;

#[tauri::command]
pub async fn valid_key_package_exists_for_user(
    pubkey: String,
    wn: tauri::State<'_, Whitenoise>,
) -> Result<bool, String> {
    let key_package = fetch_key_package_for_pubkey(pubkey, &wn)
        .await
        .map_err(|e| e.to_string())?;
    Ok(key_package.is_some())
}

#[tauri::command]
pub async fn publish_key_package(wn: tauri::State<'_, Whitenoise>) -> Result<(), String> {
    let pubkey = wn
        .nostr
        .client
        .signer()
        .await
        .map_err(|e| e.to_string())?
        .get_public_key()
        .await
        .map_err(|e| e.to_string())?;

    let serialized_key_package =
        create_key_package_for_event(pubkey.to_hex(), &wn.nostr_mls).map_err(|e| e.to_string())?;

    let key_package_relays = match wn
        .account_manager
        .get_active_account()
        .map_err(|e| e.to_string())?
        .key_package_relays
        .is_empty()
    {
        true => return Err("Key package relays not set".to_string()),
        false => wn
            .account_manager
            .get_active_account()
            .map_err(|e| e.to_string())?
            .key_package_relays
            .clone(),
    };

    let event = EventBuilder::new(
        Kind::MlsKeyPackage,
        serialized_key_package,
        [
            Tag::custom(TagKind::MlsProtocolVersion, ["1.0"]),
            Tag::custom(
                TagKind::MlsCiphersuite,
                [wn.nostr_mls.ciphersuite_value().to_string()],
            ),
            Tag::custom(TagKind::MlsExtensions, [wn.nostr_mls.extensions_value()]),
            Tag::custom(TagKind::Client, ["whitenoise"]),
            Tag::custom(TagKind::Relays, key_package_relays.clone()),
        ],
    );

    wn.nostr
        .client
        .send_event_builder_to(key_package_relays.clone(), event)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
