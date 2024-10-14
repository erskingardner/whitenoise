use anyhow::Result;
use keyring::Entry;
use nostr_sdk::Keys;

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
    let entry = Entry::new("whitenoise", keys.public_key().to_hex().as_str())?;
    entry.set_password(
        keys.secret_key()
            .expect("Couldn't get secret key from keypair")
            .to_secret_hex()
            .as_str(),
    )?;
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
    let entry = Entry::new("whitenoise", pubkey)?;
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
    let entry = Entry::new("whitenoise", pubkey);
    if let Ok(entry) = entry {
        let _ = entry.delete_credential();
    }
    Ok(())
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
}
