use crate::Result;
use keyring::Entry;
use nostr_sdk::Keys;

pub fn store_private_key(keys: Keys) -> Result<()> {
    let entry = Entry::new("whitenoise", keys.public_key().to_hex().as_str())?;
    entry.set_password(
        keys.secret_key()
            .expect("Couldn't get secret key from keypair")
            .to_secret_hex()
            .as_str(),
    )?;
    Ok(())
}

pub fn get_nostr_keys_for_pubkey(pubkey: &str) -> Result<Keys> {
    let entry = Entry::new("whitenoise", pubkey)?;
    let private_key = entry.get_password()?;
    Ok(Keys::parse(private_key)?)
}

pub fn remove_private_key_for_pubkey(pubkey: &str) -> Result<()> {
    let entry = Entry::new("whitenoise", pubkey)?;
    entry.delete_credential()?;
    Ok(())
}
