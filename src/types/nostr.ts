// These are types that map to the rust-nostr types from the rust backend

export type NContact = {
    public_key: string;
    relay_url: string;
    alias: string;
};
