use crate::nostr::DEFAULT_TIMEOUT;
use crate::nostr_mls::groups::NostrMlsGroup;
use crate::nostr_mls::key_packages::{
    delete_key_package_from_relays, generate_and_publish_key_package,
};
use crate::nostr_mls::nostr_group_data::NostrGroupDataExtension;
use crate::nostr_mls::NostrMlsProvider;
use crate::whitenoise::Whitenoise;
use anyhow::{anyhow, Result};
use bincode;
use log::debug;
use nostr_sdk::prelude::*;
use openmls::prelude::*;
use serde::{Deserialize, Serialize};
use tauri::State;
use tls_codec::Deserialize as TlsDeserialize;

/// Key used to store and retrieve welcome messages data in the database
const INVITES_KEY: &str = "invites";

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum InviteState {
    Pending,
    Accepted,
    Declined,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Invite {
    /// The event that contains the welcome message
    pub event: UnsignedEvent,
    /// MLS group id (from NostrGroupDataExtension)
    pub mls_group_id: String,
    /// Group name (from NostrGroupDataExtension)
    pub group_name: String,
    /// Group description (from NostrGroupDataExtension)
    pub group_description: String,
    /// Group admin pubkeys (from NostrGroupDataExtension)
    pub group_admin_pubkeys: Vec<String>,
    /// Group relays (from NostrGroupDataExtension)
    pub group_relays: Vec<String>,
    /// Pubkey of the user that sent the invite
    pub invitee: String,
    /// Member count of the group
    pub member_count: usize,
    /// The state of the invite
    pub state: InviteState,
}

impl Invite {
    fn invite_tree_key(wn: State<'_, Whitenoise>) -> Result<String> {
        let accounts = wn.accounts.lock().expect("Failed to lock accounts");
        let current_identity = match &accounts.current_identity {
            Some(identity) => identity,
            None => return Err(anyhow!("No current identity found")),
        };
        Ok(format!("{}{}", INVITES_KEY, current_identity))
    }

    pub fn save(&self, wn: State<'_, Whitenoise>) -> Result<()> {
        let key = match &self.event.id {
            Some(id) => id.to_string(),
            None => return Err(anyhow!("Event ID is None")),
        };
        let serialized = bincode::serialize(self)?;
        wn.wdb
            .insert_in_tree(Invite::invite_tree_key(wn.clone())?, key, &serialized)?;
        Ok(())
    }

    pub fn delete(&self, wn: State<'_, Whitenoise>) -> Result<()> {
        let key = match &self.event.id {
            Some(id) => id.to_string(),
            None => return Err(anyhow!("Event ID is None")),
        };
        wn.wdb
            .delete_from_tree(Invite::invite_tree_key(wn.clone())?, key)?;
        Ok(())
    }

    pub fn get_invites(wn: State<'_, Whitenoise>) -> Result<Vec<Invite>> {
        let invite_tree = wn.wdb.db.open_tree(Invite::invite_tree_key(wn.clone())?)?;
        let invites = invite_tree
            .iter()
            .filter_map(|result| {
                result
                    .ok()
                    .and_then(|(_, value)| bincode::deserialize::<Invite>(&value).ok())
            })
            .collect::<Vec<Invite>>();
        debug!(target: "nostr_mls::invites::get_invites", "Loaded {:?} invites from the database", invites.len());
        Ok(invites)
    }
}

pub fn parse_welcome_message(welcome_message_hex: String) -> Result<Welcome> {
    let welcome_message_bytes = hex::decode(welcome_message_hex)?;
    let welcome_message_in = MlsMessageIn::tls_deserialize(&mut welcome_message_bytes.as_slice())?;
    let welcome = match welcome_message_in.extract() {
        MlsMessageBodyIn::Welcome(welcome) => welcome,
        _ => unreachable!("Failed to parse welcome message"),
    };

    Ok(welcome)
}

#[tauri::command]
pub async fn fetch_invites_for_user(
    pubkey: String,
    wn: State<'_, Whitenoise>,
) -> Result<Vec<Invite>, String> {
    debug!(target: "nostr_mls::invites::fetch_invites_for_user", "Fetching invites for user {:?}", pubkey);

    let (keys, key_package_relays) = get_user_keys_and_relays(&wn)?;

    let gw_events = fetch_giftwrapped_events(&wn, pubkey.clone())
        .await
        .expect("Failed to fetch giftwrapped events");
    debug!(target: "nostr_mls::invites::fetch_invites_for_user", "Found {:?} Gift-wrapped messages for user {:?}", gw_events.len(), pubkey);

    let invite_events = extract_invite_events(&keys, gw_events);

    let all_invites = Invite::get_invites(wn.clone())
        .map_err(|e| format!("Failed to get invites from the database: {}", e))?;

    debug!(target: "nostr_mls::invites::fetch_invites_for_user", "Found {:?} invites in the database", all_invites.len());

    // Get the invites that we have already processed from the database
    let mut pending_invites: Vec<Invite> = all_invites
        .clone()
        .into_iter()
        .filter(|invite| invite.state == InviteState::Pending)
        .collect();

    debug!(target: "nostr_mls::invites::fetch_invites_for_user", "Found {:?} pending invites in the database", pending_invites.len());

    let mls_group_config = MlsGroupJoinConfig::builder()
        .use_ratchet_tree_extension(true)
        .build();

    // We need to track the key_packages that were used in procesing this batch of invites
    // These key_packages need to be deleted from the relays and the private key material needs to be deleted from the MLS database
    let mut used_key_package_ids: Vec<String> = Vec::new();

    for event in &invite_events {
        // Skip if we have already processed this invite
        if all_invites.iter().any(|invite| invite.event.id == event.id) {
            debug!(target: "nostr_mls::invites::fetch_invites_for_user", "Invite {:?} already processed", event.id.unwrap());
            continue;
        }
        if let (Some(invite), key_package_event_id) = process_invite_event(
            event,
            &wn,
            &mls_group_config,
            &wn.nostr_mls.provider.lock().unwrap(),
        )
        .expect("Failed to process invite event")
        {
            pending_invites.push(invite);
            if let Some(key_package_event_id) = key_package_event_id {
                used_key_package_ids.push(key_package_event_id);
            }
        }
    }
    debug!(target: "nostr_mls::invites::fetch_invites_for_user", "Found {} invites", pending_invites.len());

    // Remove used key package ids from relays and from MLS storage
    // We do this in bulk after we've processed all welcome events to avoid deleting
    // the key package material while we're still processing events that might need it.
    used_key_package_ids.sort();
    used_key_package_ids.dedup();
    for key_package_id in &used_key_package_ids {
        debug!(target: "nostr_mls::invites::fetch_invites_for_user", "Deleting used key package {:?}", key_package_id);
        delete_key_package_from_relays(
            &EventId::parse(key_package_id).unwrap(),
            &key_package_relays,
            true,
            &wn,
        )
        .await
        .map_err(|e| format!("Couldn't delete key package {:?}: {}", key_package_id, e))?;
    }

    // Generate and publish new key packages to replace the used key packages
    for _ in used_key_package_ids.iter() {
        generate_and_publish_key_package(pubkey.clone(), wn.clone()).await?;
    }

    // TODO: We need to filter and only show the latest welcome message for a given group if there are duplicates
    Ok(pending_invites)
}

#[tauri::command]
pub fn accept_invite(mut invite: Invite, wn: State<'_, Whitenoise>) -> Result<(), String> {
    debug!(target: "nostr_mls::invites::accept_invite", "Accepting invite {:?}", invite.event.id.unwrap());

    let mls_group_config = MlsGroupJoinConfig::builder()
        .use_ratchet_tree_extension(true)
        .build();

    // Parse the welcome message and process it
    let welcome = parse_welcome_message(invite.event.content.clone())
        .map_err(|e| format!("Failed to parse welcome message: {}", e))?;

    let staged_welcome = StagedWelcome::new_from_welcome(
        &*wn.nostr_mls.provider.lock().unwrap(),
        &mls_group_config,
        welcome,
        None,
    )
    .map_err(|e| format!("Failed to create staged welcome: {}", e))?;

    // Get the group_context and create Nostr group data from it
    let nostr_group_data =
        NostrGroupDataExtension::from_group_context(staged_welcome.group_context())
            .map_err(|e| format!("Failed to create Nostr group data: {}", e))?;

    let group = staged_welcome
        .into_group(&*wn.nostr_mls.provider.lock().unwrap())
        .map_err(|e| format!("Failed to convert staged welcome to group: {}", e))?;

    let nostr_mls_group = NostrMlsGroup::new(group.group_id().to_vec(), nostr_group_data);
    nostr_mls_group
        .save(wn.clone())
        .map_err(|e| format!("Failed to save nostr_mls_group to database: {}", e))?;

    invite.state = InviteState::Accepted;
    invite
        .delete(wn.clone())
        .map_err(|e| format!("Failed to delete invite from database: {}", e))?;
    debug!(target: "nostr_mls::invites::accept_invite", "Deleted invite from database: {:?}", invite);
    invite
        .save(wn.clone())
        .map_err(|e| format!("Failed to update invite state: {}", e))?;
    // wn.wdb
    //     .db
    //     .flush()
    //     .map_err(|e| format!("Failed to flush database: {}", e))?;
    debug!(target: "nostr_mls::invites::accept_invite", "Updated invite in database: {:?}", invite);
    Ok(())
}

#[tauri::command]
pub fn decline_invite(mut invite: Invite, wn: State<'_, Whitenoise>) -> Result<(), String> {
    invite.state = InviteState::Declined;
    invite
        .save(wn.clone())
        .map_err(|e| format!("Failed to save invite: {}", e))?;
    Ok(())
}

/// Retrieves the user's Nostr keys and key package relays for the current identity.
///
/// This function accesses the Whitenoise state to fetch the Nostr keys and key package relays
/// associated with the current user identity.
///
/// # Arguments
///
/// * `wn` - A reference to the Whitenoise state.
///
/// # Returns
///
/// * `Result<(Keys, Vec<String>), String>` - A Result containing a tuple of:
///   - `Keys`: The Nostr keys for the current identity.
///   - `Vec<String>`: A list of key package relay URLs.
///   - If an error occurs, it returns a String describing the error.
///
/// # Errors
///
/// This function may return an error if:
/// - It fails to acquire the lock on the accounts.
/// - It fails to retrieve the Nostr keys for the current identity.
/// - It fails to retrieve the key package relays for the current identity.
fn get_user_keys_and_relays(wn: &State<'_, Whitenoise>) -> Result<(Keys, Vec<String>), String> {
    let accounts = wn.accounts.lock().unwrap();
    let keys = accounts
        .get_nostr_keys_for_current_identity()
        .expect("Failed to get nostr keys for current identity")
        .ok_or_else(|| "Failed to get nostr keys for current identity".to_string())?;
    let key_package_relays = accounts
        .get_key_package_relays_for_current_identity()
        .expect("Failed to get key package relays for current identity");
    Ok((keys, key_package_relays))
}

/// Fetches giftwrapped events for a specific public key from Nostr relays.
///
/// This function retrieves all giftwrapped events (Kind::GiftWrap) that are addressed to the
/// specified public key. It uses the Nostr client to fetch events from both local storage and
/// connected relays.
///
/// # Arguments
///
/// * `wn` - A Tauri State containing a Whitenoise instance, which provides access to Nostr functionality.
/// * `pubkey` - A String representing the public key for which to fetch giftwrapped events.
///
/// # Returns
///
/// * `Result<Vec<Event>>` - A Result that is Ok with a vector of Event objects if the fetch is successful,
///   or an Err with a descriptive error message if the fetch fails.
async fn fetch_giftwrapped_events(
    wn: &State<'_, Whitenoise>,
    pubkey: String,
) -> Result<Vec<Event>> {
    let public_key = PublicKey::from_hex(&pubkey).expect("Failed to parse public key");
    match wn
        .nostr
        .fetch_events(
            vec![Filter::new().kind(Kind::GiftWrap).pubkeys(vec![public_key])],
            Some(DEFAULT_TIMEOUT),
        )
        .await
    {
        Ok(events) => Ok(events.into_iter().collect()),
        Err(e) => Err(anyhow::anyhow!("Failed to fetch giftwrapped events: {}", e)),
    }
}

/// Extracts welcome events from a list of giftwrapped events.
///
/// This function processes a list of giftwrapped events and extracts the welcome events
/// (events with Kind::Custom(444)) from them.
///
/// # Arguments
///
/// * `keys` - A reference to the Keys struct containing the necessary keys for decryption.
/// * `gw_events` - A vector of giftwrapped Event objects to process.
///
/// # Returns
///
/// A vector of UnsignedEvent objects representing the extracted welcome events.
fn extract_invite_events(keys: &Keys, gw_events: Vec<Event>) -> Vec<UnsignedEvent> {
    let mut invite_events: Vec<UnsignedEvent> = Vec::new();

    for event in gw_events {
        if let Ok(unwrapped) = extract_rumor(keys, &event) {
            if unwrapped.rumor.kind == Kind::Custom(444) {
                invite_events.push(unwrapped.rumor);
            }
        }
    }

    invite_events
}

/// Processes an invite event for an MLS group.
///
/// This function performs the following steps:
/// 1. Parses the MLS welcome message from the event content.
/// 2. Creates a StagedWelcome from the parsed message.
/// 3. Extracts Nostr group data from the StagedWelcome's group context.
/// 4. Checks for the key package event ID used in the welcome message.
/// 5. Creates and saves an Invite to the database.
///
/// # Arguments
///
/// * `event` - The UnsignedEvent containing the welcome message.
/// * `wn` - A Tauri State containing a Whitenoise instance.
/// * `mls_group_config` - Configuration for joining the MLS group.
///
/// # Returns
///
/// A Result containing a tuple with:
/// - An Option<Invite> representing the processed invite.
/// - An Option<String> representing the key package event ID, if present.
///
/// # Errors
///
/// This function may return an error if:
/// - Parsing the welcome message fails.
/// - Creating the staged welcome fails.
/// - Extracting Nostr group data fails.
/// - Saving the invite to the database fails.
fn process_invite_event(
    event: &UnsignedEvent,
    wn: &State<'_, Whitenoise>,
    mls_group_config: &MlsGroupJoinConfig,
    provider: &NostrMlsProvider,
) -> Result<(Option<Invite>, Option<String>)> {
    debug!(target: "nostr_mls::invites::process_invite_event", "Processing invite event: {:?}", event.id);
    // Parse the welcome message and process it
    let welcome = parse_welcome_message(event.content.clone())?;

    // Remove the lock here
    let staged_welcome =
        StagedWelcome::new_from_welcome(provider, mls_group_config, welcome, None)?;

    // Get the group_context and create Nostr group data from it
    let nostr_group_data =
        NostrGroupDataExtension::from_group_context(staged_welcome.group_context())?;

    // Check the key_package used, so we can delete it from the relays and replace with a new one
    let key_package_event_id: Option<&str> = event
        .tags
        .iter()
        .find(|tag| tag.kind() == TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::E)))
        .and_then(|tag| tag.content());

    // Create the invite and save it to the database
    let invite = Invite {
        event: event.clone(),
        mls_group_id: nostr_group_data.nostr_group_id(),
        group_name: nostr_group_data.name(),
        group_description: nostr_group_data.description(),
        group_admin_pubkeys: nostr_group_data.admin_identities(),
        group_relays: nostr_group_data.relays(),
        invitee: event.pubkey.to_string(),
        member_count: staged_welcome.members().count(),
        state: InviteState::Pending,
    };

    // Save the invite to the database
    invite.save(wn.clone())?;
    debug!(target: "nostr_mls::invites::process_invite_event", "Saved invite to database: {:?}", invite);

    Ok((Some(invite), key_package_event_id.map(|id| id.to_string())))
}
