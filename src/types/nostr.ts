// These are types that map to the rust-nostr types from the rust backend

export type NContact = {
    public_key: string;
    relay_url: string;
    alias: string;
};

export type NChat = {
    [key: string]: {
        latest: number; 
        events: NEvent[];
    }
};

export type NEvent = {
    id: string;
    pubkey: string;
    created_at: number;
    kind: number;
    tags: string[][];
    content: string;
    sig: string;
}