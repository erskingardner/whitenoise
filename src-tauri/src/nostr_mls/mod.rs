pub mod groups;
pub mod key_packages;
pub mod nostr_group_data;

use openmls::prelude::*;

const DEFAULT_CIPHERSUITE: Ciphersuite = Ciphersuite::MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519;
const DEFAULT_EXTENSIONS: &[ExtensionType] = &[
    ExtensionType::RequiredCapabilities,
    ExtensionType::Unknown(0xFF69),
];

pub struct NostrMls {
    ciphersuite: Ciphersuite,
    extensions: Vec<ExtensionType>,
    crypto_provider: openmls_libcrux_crypto::Provider,
}

impl NostrMls {
    pub fn new() -> Self {
        let crypto_provider = openmls_libcrux_crypto::Provider::default();
        Self {
            ciphersuite: DEFAULT_CIPHERSUITE,
            extensions: DEFAULT_EXTENSIONS.to_vec(),
            crypto_provider,
        }
    }

    pub fn ciphersuite_value(&self) -> u16 {
        self.ciphersuite.into()
    }

    pub fn extensions_value(&self) -> String {
        self.extensions
            .iter()
            .map(|e| format!("{:?}", e))
            .collect::<Vec<String>>()
            .join(",")
    }
}

impl Default for NostrMls {
    fn default() -> Self {
        Self::new()
    }
}
