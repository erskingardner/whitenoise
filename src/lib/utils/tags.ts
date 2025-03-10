import type { NEvent } from "$lib/types/nostr";

export function findTargetId(event: NEvent): string | undefined{
    return event.tags.find((t) => t[0] === "e")?.[1];
} 

export function findBolt11Tag(event: NEvent): string[] | undefined {
    return event.tags.find((t) => t[0] === "bolt11");
}

export function findPreimage(event: NEvent): string | undefined {
  return event.tags.find((t) => t[0] === "preimage")?.[1];
}

export function findReplyToId(event: NEvent): string | undefined {
  return event.tags.find((t) => t[0] === "q")?.[1];
}
