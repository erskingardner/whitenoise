pub fn is_valid_hex_pubkey(pubkey: &str) -> bool {
    pubkey.len() == 64 && pubkey.chars().all(|c| c.is_ascii_hexdigit())
}
