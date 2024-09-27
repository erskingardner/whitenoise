// These are types that map to the rust-nostr types from the rust backend
export type HexPubkey = string & { readonly __brand: unique symbol };

export function isValidHexPubkey(value: string): value is HexPubkey {
    return /^[a-fA-F0-9]{64}$/.test(value);
}

export type Npub = string & { readonly __brand: unique symbol };

export function isValidNpub(value: string): value is Npub {
    return /^npub1[a-zA-Z0-9]{59}$/.test(value);
}

export type NUsers = {
    [keys: string]: {
        metadata: NMetadata;
        nip17: boolean;
        nip104: boolean;
        inbox_relays: string[];
    };
};

export type NMetadata = {
    name?: string;
    display_name?: string;
    about?: string;
    picture?: string;
    banner?: string;
    website?: string;
    nip05?: string;
    lud06?: string;
    lud16?: string;
};

export type NChat = {
    [key: string]: {
        latest: number;
        metadata: NMetadata;
        events: NEvent[];
    };
};

export type NEvent = {
    id: string;
    pubkey: string;
    created_at: number;
    kind: number;
    tags: string[][];
    content: string;
    sig: string;
};
