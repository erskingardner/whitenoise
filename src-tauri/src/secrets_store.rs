use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use keyring::Entry;
use nostr_sdk::Keys;
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use tauri::is_dev;

fn get_service_name() -> String {
    match is_dev() {
        true => "White Noise Dev".to_string(),
        false => "White Noise".to_string(),
    }
}

fn get_device_key() -> Vec<u8> {
    // In a real-world scenario, you'd want a more secure way to generate and store this key
    "device_specific_key".as_bytes().to_vec()
}

fn obfuscate(data: &str) -> String {
    let device_key = get_device_key();
    let xored: Vec<u8> = data
        .as_bytes()
        .iter()
        .zip(device_key.iter().cycle())
        .map(|(&x1, &x2)| x1 ^ x2)
        .collect();
    general_purpose::STANDARD_NO_PAD.encode(xored)
}

fn deobfuscate(data: &str) -> Result<String> {
    let device_key = get_device_key();
    let decoded = general_purpose::STANDARD_NO_PAD.decode(data)?;
    let xored: Vec<u8> = decoded
        .iter()
        .zip(device_key.iter().cycle())
        .map(|(&x1, &x2)| x1 ^ x2)
        .collect();
    Ok(String::from_utf8(xored)?)
}

fn read_secrets_file(file_path: &PathBuf) -> Result<Value> {
    let content = fs::read_to_string(file_path)?;
    Ok(serde_json::from_str(&content)?)
}

fn write_secrets_file(file_path: &PathBuf, secrets: &Value) -> Result<()> {
    let content = serde_json::to_string_pretty(secrets)?;
    fs::write(file_path, content)?;
    Ok(())
}

/// Stores the private key associated with the given Keys in the system's keyring.
///
/// This function takes a reference to a `Keys` object and stores the private key
/// in the system's keyring, using the public key as an identifier.
///
/// # Arguments
///
/// * `keys` - A reference to a `Keys` object containing the keypair to store.
/// * `file_path` - The path to the secrets file.
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
pub fn store_private_key(keys: &Keys, file_path: &PathBuf) -> Result<()> {
    let service = get_service_name();

    if cfg!(target_os = "android") {
        let mut secrets = read_secrets_file(file_path).unwrap_or(json!({}));
        let obfuscated_key = obfuscate(keys.secret_key().to_secret_hex().as_str());
        secrets[keys.public_key().to_hex()] = json!(obfuscated_key);
        write_secrets_file(file_path, &secrets)?;
    } else {
        let entry = Entry::new(service.as_str(), keys.public_key().to_hex().as_str())?;
        entry.set_password(keys.secret_key().to_secret_hex().as_str())?;
    }

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
/// * `file_path` - The path to the secrets file.
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
pub fn get_nostr_keys_for_pubkey(pubkey: &str, file_path: &PathBuf) -> Result<Keys> {
    let service = get_service_name();

    if cfg!(target_os = "android") {
        let secrets = read_secrets_file(file_path)?;
        let obfuscated_key = secrets[pubkey]
            .as_str()
            .ok_or(anyhow::anyhow!("Key not found"))?;
        let private_key = deobfuscate(obfuscated_key)?;
        Ok(Keys::parse(private_key)?)
    } else {
        let entry = Entry::new(service.as_str(), pubkey)?;
        let private_key = entry.get_password()?;
        Ok(Keys::parse(private_key)?)
    }
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
/// * `file_path` - The path to the secrets file.
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
pub fn remove_private_key_for_pubkey(pubkey: &str, file_path: &PathBuf) -> Result<()> {
    let service = get_service_name();

    if cfg!(target_os = "android") {
        let mut secrets = read_secrets_file(file_path)?;
        secrets.as_object_mut().map(|obj| obj.remove(pubkey));
        write_secrets_file(file_path, &secrets)?;
    } else {
        let entry = Entry::new(service.as_str(), pubkey);
        if let Ok(entry) = entry {
            let _ = entry.delete_credential();
        }
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
/// * `file_path` - The path to the secrets file.
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
pub fn store_mls_export_secret(
    group_id: &str,
    epoch: u64,
    secret: &str,
    file_path: &PathBuf,
) -> Result<()> {
    let key = format!("{group_id}:{epoch}");
    let service = get_service_name();

    if cfg!(target_os = "android") {
        let mut secrets = read_secrets_file(file_path).unwrap_or(json!({}));
        let obfuscated_secret = obfuscate(secret);
        secrets[key] = json!(obfuscated_secret);
        write_secrets_file(file_path, &secrets)?;
    } else {
        let entry = Entry::new(service.as_str(), key.as_str())?;
        entry.set_password(secret)?;
    }
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
/// * `file_path` - The path to the secrets file.
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
pub fn get_export_secret_keys_for_group(
    group_id: &str,
    epoch: u64,
    file_path: &PathBuf,
) -> Result<Keys> {
    let key = format!("{group_id}:{epoch}");
    let service = get_service_name();

    if cfg!(target_os = "android") {
        let secrets = read_secrets_file(file_path)?;
        let obfuscated_secret = secrets[key]
            .as_str()
            .ok_or(anyhow::anyhow!("Secret not found"))?;
        let secret = deobfuscate(obfuscated_secret)?;
        let keys = Keys::parse(secret)?;
        Ok(keys)
    } else {
        let entry = Entry::new(service.as_str(), key.as_str())?;
        let secret = entry.get_password()?;
        let keys = Keys::parse(secret)?;
        Ok(keys)
    }
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
        store_private_key(&keys, &PathBuf::from("secrets.json"))?;

        // Retrieve the keys
        let retrieved_keys = get_nostr_keys_for_pubkey(&pubkey, &PathBuf::from("secrets.json"))?;

        assert_eq!(keys.public_key(), retrieved_keys.public_key());
        assert_eq!(keys.secret_key(), retrieved_keys.secret_key());

        // Clean up
        remove_private_key_for_pubkey(&pubkey, &PathBuf::from("secrets.json"))?;

        Ok(())
    }

    #[test]
    fn test_remove_private_key() -> Result<()> {
        let keys = Keys::generate();
        let pubkey = keys.public_key().to_hex();

        // Store the private key
        store_private_key(&keys, &PathBuf::from("secrets.json"))?;

        // Remove the private key
        remove_private_key_for_pubkey(&pubkey, &PathBuf::from("secrets.json"))?;

        // Attempt to retrieve the removed key
        let result = get_nostr_keys_for_pubkey(&pubkey, &PathBuf::from("secrets.json"));

        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_nonexistent_key() {
        let nonexistent_pubkey = "nonexistent_pubkey";
        let result = get_nostr_keys_for_pubkey(nonexistent_pubkey, &PathBuf::from("secrets.json"));

        assert!(result.is_err());
    }

    #[test]
    fn test_store_and_retrieve_mls_export_secret() -> Result<()> {
        let group_id = "test_group";
        let epoch = 42;
        let secret = "9b9da9c6ee9a62016ab2db1a3397d267a575c02266c6ca9b5ec8e015db67c30e";

        // Store the MLS export secret
        store_mls_export_secret(group_id, epoch, secret, &PathBuf::from("secrets.json"))?;

        // Retrieve the keys
        let retrieved_keys =
            get_export_secret_keys_for_group(group_id, epoch, &PathBuf::from("secrets.json"))?;

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

        let result = get_export_secret_keys_for_group(
            nonexistent_group_id,
            nonexistent_epoch,
            &PathBuf::from("secrets.json"),
        );

        assert!(result.is_err());
    }
}
