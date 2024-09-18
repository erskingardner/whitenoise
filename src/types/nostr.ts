// These are types that map to the rust-nostr types from the rust backend

export type NUsers = {
    [keys: string]: {
        metadata: NMetadata;
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
