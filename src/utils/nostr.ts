import { npubEncode } from "nostr-tools/nip19";

export function npubFromPubkey(pubkey: string): string {
    return npubEncode(pubkey);
}

export function truncatedNpub(pubkey: string, length: number = 20): string {
    return npubFromPubkey(pubkey).slice(0, length);
}
