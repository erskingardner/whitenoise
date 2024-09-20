use openmls::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use tls_codec::{TlsDeserialize, TlsDeserializeBytes, TlsSerialize, TlsSize};

/// # Nostr Group Extension
/// This is an MLS Group Context extension used to store the group's name,
/// description, ID, and other metadata.

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
    TlsSize,
)]
pub struct NostrGroupDataExtension {
    id: VLBytes,
    name: VLBytes,
    description: VLBytes,
}

impl NostrGroupDataExtension {
    fn extension_type(&self) -> ExtensionType {
        ExtensionType::Unknown(0xFF69)
    }
    pub fn new(name: String, description: String) -> Self {
        // Generate a random 32-byte group ID
        let mut rng = rand::thread_rng();
        let random_bytes: [u8; 32] = rng.gen();
        let id = VLBytes::from(random_bytes.to_vec());

        Self {
            id,
            name: VLBytes::from(name.as_bytes().to_vec()),
            description: VLBytes::from(description.as_bytes().to_vec()),
        }
    }
}
