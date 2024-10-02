pub mod groups;
pub mod key_packages;
pub mod nostr_group_data;
use openmls::prelude::*;
use openmls_rust_crypto::OpenMlsRustCrypto;

const DEFAULT_CIPHERSUITE: Ciphersuite = Ciphersuite::MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519;
const DEFAULT_EXTENSIONS: &[ExtensionType] = &[
    ExtensionType::RequiredCapabilities,
    ExtensionType::Unknown(0xFF69),
];

#[allow(dead_code)]
const GREASE: &[u16] = &[
    0x0A0A, 0x1A1A, 0x2A2A, 0x3A3A, 0x4A4A, 0x5A5A, 0x6A6A, 0x7A7A, 0x8A8A, 0x9A9A, 0xAAAA, 0xBABA,
    0xCACA, 0xDADA, 0xEAEA,
];

pub struct NostrMls {
    ciphersuite: Ciphersuite,
    extensions: Vec<ExtensionType>,
    crypto_provider: OpenMlsRustCrypto,
}

impl NostrMls {
    pub fn new() -> Self {
        let crypto_provider = OpenMlsRustCrypto::default();
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

// TODO: Add grease values to the ciphersuite and extensions
