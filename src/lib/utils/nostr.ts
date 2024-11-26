import { npubEncode } from "nostr-tools/nip19";
import type { NEvent, NMetadata } from "../types/nostr";

/**
 * Retrieves the display name from the given NMetadata object.
 *
 * @param metadata - The NMetadata object containing user information.
 * @returns The display name in the following priority order:
 *          1. display_name
 *          2. name
 *          3. truncated npub of the pubkey (if available)
 */
export function nameFromMetadata(metadata: NMetadata, pubkey?: string, truncate = true): string {
    return (
        metadata.display_name ||
        metadata.name ||
        (pubkey ? (truncate ? truncatedNpub(pubkey) : npubFromPubkey(pubkey)) : "")
    );
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
export function truncatedNpub(pubkey: string, length = 20): string {
    return `${npubFromPubkey(pubkey).slice(0, length)}...`;
}

/**
 * Checks if a Nostr event is considered insecure from a messaging standpoint.
 *
 * @param event - The Nostr event to check.
 * @returns True if the event is considered insecure, false otherwise.
 *
 * @remarks
 * This function considers events with kinds 4 and 14 as insecure.
 * Kind 4 typically represents encrypted direct messages, which leak metadata.
 * kind 14 is often used for encrypted and gift-wrapped direct messages, which have no
 * PCS or forward secrecy.
 */
export function isInsecure(event: NEvent): boolean {
    const insecureKinds = [4, 14];
    return insecureKinds.includes(event.kind);
}
