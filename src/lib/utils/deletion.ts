import type { NEvent } from "$lib/types/nostr";
import type { Deletion } from '$lib/types/chat';
import { findTargetId } from './tags';

export function eventToDeletion(event: NEvent): Deletion | null {
    const targetId = findTargetId(event);
    if (!targetId) return null;

    return {
        id: event.id,
        pubkey: event.pubkey,
        targetId,
        event
    };
} 
