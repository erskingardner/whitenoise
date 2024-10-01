import { npubEncode } from "nostr-tools/nip19";
import type { NMetadata, NEvent } from "../types/nostr";
import type { NDKUserProfile } from "@nostr-dev-kit/ndk";

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

/**
 * Converts an NDKUserProfile object to an NMetadata object.
 *
 * @param profile - The NDKUserProfile object to convert. Can be undefined.
 * @returns An NMetadata object containing the user's profile information.
 *
 * @remarks
 * This function maps the properties from NDKUserProfile to NMetadata.
 * If the input profile is undefined, an empty object is returned.
 *
 * The mapping is as follows:
 * - name -> name
 * - displayName -> display_name
 * - about -> about
 * - image -> picture
 * - banner -> banner
 * - website -> website
 * - nip05 -> nip05
 * - lud06 -> lud06
 * - lud16 -> lud16
 */
export function ndkUserProfileToNMetadata(profile: NDKUserProfile | undefined): NMetadata {
    if (!profile) return {};
    return {
        name: profile.name,
        display_name: profile.displayName,
        about: profile.about,
        picture: profile.image,
        banner: profile.banner,
        website: profile.website,
        nip05: profile.nip05,
        lud06: profile.lud06,
        lud16: profile.lud16,
    };
}
