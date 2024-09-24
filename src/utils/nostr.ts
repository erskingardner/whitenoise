import { npubEncode } from "nostr-tools/nip19";
import type { NMetadata } from "../types/nostr";

/**
 * Retrieves the display name from the given NMetadata object.
 *
 * @param metadata - The NMetadata object containing user information.
 * @returns The display name in the following priority order:
 *          1. display_name
 *          2. name
 *          3. truncated npub of the pubkey (if available)
 */
export function nameFromMetadata(metadata: NMetadata, pubkey?: string): string {
    return metadata.display_name || metadata.name || (pubkey ? npubFromPubkey(pubkey) : "");
}

/**
 * Converts a public key to its npub (Nostr public key) representation.
 * @param pubkey - The public key to convert.
 * @returns The npub representation of the public key.
 */
export function npubFromPubkey(pubkey: string): string {
    return npubEncode(pubkey);
}

/**
 * Generates a truncated npub from a public key.
 * @param pubkey - The public key to convert and truncate.
 * @param length - The desired length of the truncated npub. Defaults to 20.
 * @returns A truncated npub representation of the public key.
 */
export function truncatedNpub(pubkey: string, length: number = 20): string {
    return npubFromPubkey(pubkey).slice(0, length);
}
