import type { NEvent } from "$lib/types/nostr";

/**
 * Check if a message has been deleted by looking for a kind 5 deletion event
 * that references the message's ID in its 'e' tag.
 *
 * @param messageId - The ID of the message to check for deletion
 * @param messages - Array of messages to search for deletion events
 * @returns boolean indicating whether the message has been deleted
 */
export function messageHasDeletionTag(messageId: string, messages: NEvent[]): boolean {
    return messages.some(
        (m) => m.kind === 5 && m.tags.some((t) => t[0] === "e" && t[1] === messageId)
    );
}
