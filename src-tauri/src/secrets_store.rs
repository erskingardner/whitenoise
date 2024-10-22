use anyhow::Result;
use keyring::Entry;
use nostr_sdk::Keys;
use tauri::is_dev;

fn get_service_name() -> String {
    match is_dev() {
        true => "White Noise Dev".to_string(),
        false => "White Noise".to_string(),
    }
}

/// Stores the private key associated with the given Keys in the system's keyring.
///
/// This function takes a reference to a `Keys` object and stores the private key
/// in the system's keyring, using the public key as an identifier.
///
/// # Arguments
///
/// * `keys` - A reference to a `Keys` object containing the keypair to store.
///
/// # Returns
///
/// * `Result<()>` - Ok(()) if the operation was successful, or an error if it failed.
///
/// # Errors
///
/// This function will return an error if:
/// * The Entry creation fails
/// * Setting the password in the keyring fails
/// * The secret key cannot be retrieved from the keypair
pub fn store_private_key(keys: &Keys) -> Result<()> {
    let service = get_service_name();
    let entry = Entry::new(service.as_str(), keys.public_key().to_hex().as_str())?;
    entry.set_password(keys.secret_key().to_secret_hex().as_str())?;
    Ok(())
}

/// Retrieves the Nostr keys associated with a given public key from the system's keyring.
///
/// This function looks up the private key stored in the system's keyring using the provided
/// public key as an identifier, and then constructs a `Keys` object from the retrieved private key.
///
/// # Arguments
///
/// * `pubkey` - A string slice containing the public key to look up.
///
/// # Returns
///
/// * `Result<Keys>` - A Result containing the `Keys` object if successful, or an error if the operation fails.
///
/// # Errors
///
/// This function will return an error if:
/// * The Entry creation fails
/// * Retrieving the password from the keyring fails
/// * Parsing the private key into a `Keys` object fails
pub fn get_nostr_keys_for_pubkey(pubkey: &str) -> Result<Keys> {
    let service = get_service_name();
    let entry = Entry::new(service.as_str(), pubkey)?;
    let private_key = entry.get_password()?;
    Ok(Keys::parse(private_key)?)
}

/// Removes the private key associated with a given public key from the system's keyring.
///
/// This function attempts to delete the credential entry for the specified public key
/// from the system's keyring. If the entry doesn't exist or the deletion fails, the
/// function will still return Ok(()) to maintain idempotency.
///
/// # Arguments
///
/// * `pubkey` - A string slice containing the public key for which to remove the associated private key.
///
/// # Returns
///
/// * `Result<()>` - Ok(()) if the operation was successful or if the key didn't exist,
///                  or an error if the Entry creation fails.
///
/// # Errors
///
/// This function will return an error if:
/// * The Entry creation fails
pub fn remove_private_key_for_pubkey(pubkey: &str) -> Result<()> {
    let service = get_service_name();
    let entry = Entry::new(service.as_str(), pubkey);
    if let Ok(entry) = entry {
        let _ = entry.delete_credential();
    }
    Ok(())
}

/// Stores the MLS export secret for a specific group and epoch in the system's keyring.
///
/// This function creates a unique key by combining the group ID and epoch, then stores
/// the provided secret in the system's keyring using this key.
///
/// # Arguments
///
/// * `group_id` - A string slice containing the ID of the MLS group.
/// * `epoch` - The epoch number as a u64.
/// * `secret` - A string slice containing the export secret to be stored.
///
/// # Returns
///
/// * `Result<()>` - Ok(()) if the operation was successful, or an error if it fails.
///
/// # Errors
///
/// This function will return an error if:
/// * The Entry creation fails
/// * Setting the password in the keyring fails
pub fn store_mls_export_secret(group_id: &str, epoch: u64, secret: &str) -> Result<()> {
    let key = format!("{group_id}:{epoch}");
    let service = get_service_name();
    let entry = Entry::new(service.as_str(), key.as_str())?;
    entry.set_password(secret)?;
    Ok(())
}

/// Retrieves the export secret keys for a specific MLS group and epoch from the system's keyring.
///
/// This function constructs a unique key by combining the group ID and epoch, then retrieves
/// the corresponding secret from the system's keyring. It then parses this secret into Keys.
///
/// # Arguments
///
/// * `group_id` - A string slice containing the ID of the MLS group.
/// * `epoch` - The epoch number as a u64.
///
/// # Returns
///
/// * `Result<Keys>` - Ok(Keys) if the operation was successful, or an error if it fails.
///
/// # Errors
///
/// This function will return an error if:
/// * The Entry creation fails
/// * Retrieving the password from the keyring fails
/// * Parsing the secret into Keys fails
pub fn get_export_secret_keys_for_group(group_id: &str, epoch: u64) -> Result<Keys> {
    let key = format!("{group_id}:{epoch}");
    let service = get_service_name();
    let entry = Entry::new(service.as_str(), key.as_str())?;
    let secret = entry.get_password()?;
    let keys = Keys::parse(secret)?;
    Ok(keys)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_store_and_retrieve_private_key() -> Result<()> {
        let keys = Keys::generate();
        let pubkey = keys.public_key().to_hex();

        // Store the private key
        store_private_key(&keys)?;

        // Retrieve the keys
        let retrieved_keys = get_nostr_keys_for_pubkey(&pubkey)?;

        assert_eq!(keys.public_key(), retrieved_keys.public_key());
        assert_eq!(keys.secret_key(), retrieved_keys.secret_key());

        // Clean up
        remove_private_key_for_pubkey(&pubkey)?;

        Ok(())
    }

    #[test]
    fn test_remove_private_key() -> Result<()> {
        let keys = Keys::generate();
        let pubkey = keys.public_key().to_hex();

        // Store the private key
        store_private_key(&keys)?;

        // Remove the private key
        remove_private_key_for_pubkey(&pubkey)?;

        // Attempt to retrieve the removed key
        let result = get_nostr_keys_for_pubkey(&pubkey);

        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_nonexistent_key() {
        let nonexistent_pubkey = "nonexistent_pubkey";
        let result = get_nostr_keys_for_pubkey(nonexistent_pubkey);

        assert!(result.is_err());
    }

    #[test]
    fn test_store_and_retrieve_mls_export_secret() -> Result<()> {
        let group_id = "test_group";
        let epoch = 42;
        let secret = "9b9da9c6ee9a62016ab2db1a3397d267a575c02266c6ca9b5ec8e015db67c30e";

        // Store the MLS export secret
        store_mls_export_secret(group_id, epoch, secret)?;

        // Retrieve the keys
        let retrieved_keys = get_export_secret_keys_for_group(group_id, epoch)?;

        log::debug!(
            target: "secrets_store::test_store_and_retrieve_mls_export_secret",
            "Retrieved keys: {:?}",
            retrieved_keys
        );
        // Verify that the retrieved keys match the original secret
        assert_eq!(retrieved_keys.secret_key().to_secret_hex(), secret);

        // Clean up
        let key = format!("{group_id}:{epoch}");
        let service = get_service_name();
        let entry = Entry::new(service.as_str(), key.as_str())?;
        let _ = entry.delete_credential();

        Ok(())
    }

    #[test]
    fn test_get_nonexistent_mls_export_secret() {
        let nonexistent_group_id = "nonexistent_group";
        let nonexistent_epoch = 999;

        let result = get_export_secret_keys_for_group(nonexistent_group_id, nonexistent_epoch);

        assert!(result.is_err());
    }
}
