use nostr_sdk::util::hex;
use openmls::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use tls_codec::{TlsDeserialize, TlsDeserializeBytes, TlsSerialize, TlsSerializeBytes, TlsSize};

/// # Nostr Group Extension
/// This is an MLS Group Context extension used to store the group's name,
/// description, ID, and admin identities.

#[derive(
    PartialEq,
    Eq,
    Clone,
    Debug,
    Serialize,
    Deserialize,
    TlsSerialize,
    TlsDeserialize,
    TlsDeserializeBytes,
    TlsSerializeBytes,
    TlsSize,
)]
pub struct NostrGroupDataExtension {
    pub nostr_group_id: [u8; 32],
    pub name: Vec<u8>,
    pub description: Vec<u8>,
    pub admin_identities: Vec<Vec<u8>>,
    pub relays: Vec<Vec<u8>>,
}

impl NostrGroupDataExtension {
    pub fn extension_type(&self) -> u16 {
        0xFF69
    }

    pub fn new(
        name: String,
        description: String,
        admin_identities: Vec<String>,
        relays: Vec<String>,
    ) -> Self {
        // Generate a random 32-byte group ID
        let mut rng = rand::thread_rng();
        let random_bytes: [u8; 32] = rng.gen();

        Self {
            nostr_group_id: random_bytes,
            name: name.into_bytes(),
            description: description.into_bytes(),
            admin_identities: admin_identities
                .into_iter()
                .map(|identity| identity.into_bytes())
                .collect(),
            relays: relays.into_iter().map(|relay| relay.into_bytes()).collect(),
        }
    }

    pub fn from_group_context(group_context: &GroupContext) -> Result<Self, anyhow::Error> {
        let group_data_extension = match group_context
            .extensions()
            .iter()
            .find(|ext| ext.extension_type() == ExtensionType::Unknown(0xFF69))
        {
            Some(Extension::Unknown(_, ext)) => ext,
            Some(_) => return Err(anyhow::anyhow!("Unexpected extension type")),
            None => return Err(anyhow::anyhow!("Nostr group data extension not found")),
        };

        let (deserialized, _) = Self::tls_deserialize_bytes(&group_data_extension.0)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize extension: {}", e))?;

        Ok(deserialized)
    }

    pub fn from_group(group: &MlsGroup) -> Result<Self, anyhow::Error> {
        let group_data_extension = match group
            .extensions()
            .iter()
            .find(|ext| ext.extension_type() == ExtensionType::Unknown(0xFF69))
        {
            Some(Extension::Unknown(_, ext)) => ext,
            Some(_) => return Err(anyhow::anyhow!("Unexpected extension type")),
            None => return Err(anyhow::anyhow!("Nostr group data extension not found")),
        };

        let (deserialized, _) = Self::tls_deserialize_bytes(&group_data_extension.0)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize extension: {}", e))?;

        Ok(deserialized)
    }

    pub fn nostr_group_id(&self) -> String {
        hex::encode(self.nostr_group_id)
    }

    pub fn name(&self) -> String {
        String::from_utf8_lossy(&self.name).to_string()
    }

    pub fn description(&self) -> String {
        String::from_utf8_lossy(&self.description).to_string()
    }

    pub fn admin_identities(&self) -> Vec<String> {
        self.admin_identities
            .iter()
            .map(|identity| String::from_utf8_lossy(identity).to_string())
            .collect()
    }

    pub fn relays(&self) -> Vec<String> {
        self.relays
            .iter()
            .map(|relay| String::from_utf8_lossy(relay).to_string())
            .collect()
    }
}
