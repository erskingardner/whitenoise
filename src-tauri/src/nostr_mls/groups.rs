use anyhow::Result;
use nostr_group_data::NostrGroupData;
use nostr_sdk::prelude::*;
use openmls::prelude::*;

pub fn create_group(
    group_name: String,
    description: String,
    credential_with_key: CredentialWithKey,
    signer: SignatureKeyPair,
) -> Result<MlsGroup> {
    // Define the required capabilities, including our nostr group data extension
    let mut required_capabilities = RequiredCapabilities::default();
    required_capabilities.add_extension_type(ExtensionType::Custom(0xFF69));

    // Create a new group with the given name
    let group_create_config: MlsGroupCreateConfig = MlsGroupCreateConfig::builder()
        .ciphersuite(DEFAULT_CIPHERSUITE)
        .required_capabilities(required_capabilities)
        .capabilities(DEFAULT_CAPABILITIES.clone())
        .build();

    let group_data: NostrGroupData = NostrGroupData::new(group_name, description);
    let mut group = MlsGroup::new(
        provider,
        &alice_signer,
        &group_create_config,
        alice_credential_with_keypair.clone(),
    )
    .expect("An error occurred while creating the group");

    group.extensions_mut().add(group_data.clone());

    Ok(())
}
