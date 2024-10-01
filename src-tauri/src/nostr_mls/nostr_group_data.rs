use nostr_sdk::util::hex;
use openmls::prelude::*;
use rand::Rng;
use tls_codec::{TlsDeserialize, TlsDeserializeBytes, TlsSerialize, TlsSerializeBytes, TlsSize};

/// # Nostr Group Extension
/// This is an MLS Group Context extension used to store the group's name,
/// description, ID, and admin identities.

#[derive(
    PartialEq,
    Eq,
    Clone,
    Debug,
    TlsSerialize,
    TlsDeserialize,
    TlsDeserializeBytes,
    TlsSerializeBytes,
    TlsSize,
)]
pub struct NostrGroupDataExtension {
    pub id: [u8; 32],
    pub name: Vec<u8>,
    pub description: Vec<u8>,
    pub admin_identities: Vec<Vec<u8>>,
}

impl NostrGroupDataExtension {
    pub fn extension_type(&self) -> u16 {
        0xFF69
    }

    pub fn new(name: String, description: String, admin_identities: Vec<String>) -> Self {
        // Generate a random 32-byte group ID
        let mut rng = rand::thread_rng();
        let random_bytes: [u8; 32] = rng.gen();

        Self {
            id: random_bytes,
            name: name.into(),
            description: description.into(),
            admin_identities: admin_identities
                .into_iter()
                .map(|identity| identity.into())
                .collect(),
        }
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

    pub fn get_id(&self) -> String {
        hex::encode(self.id)
    }

    pub fn get_name(&self) -> String {
        String::from_utf8_lossy(&self.name).to_string()
    }

    pub fn get_description(&self) -> String {
        String::from_utf8_lossy(&self.description).to_string()
    }

    pub fn get_admin_identities(&self) -> Vec<String> {
        self.admin_identities
            .iter()
            .map(|identity| String::from_utf8_lossy(identity).to_string())
            .collect()
    }
}
