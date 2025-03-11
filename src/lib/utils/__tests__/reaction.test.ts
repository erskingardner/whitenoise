import { describe, it, expect } from "vitest";
import { eventToReaction } from "../reaction";
import type { NEvent } from "$lib/types/nostr";
import type { Reaction } from "$lib/types/chat";

describe("eventToReaction", () => {
  describe('with valid target id', () => {
    const event: NEvent = {
      id: "test-id",
      pubkey: "test-pubkey",
      created_at: 1234567890,
      kind: 7,
      tags: [
        ["p", "author-pubkey"],
        ["e", "target-event-id"],
        ["other", "value"]
      ],
      content: "👍",
      sig: "signature"
    };
    
    it('returns a Reaction object', () => {
      const result = eventToReaction(event);
      expect(result).toEqual({
        id: "test-id",
        pubkey: "test-pubkey",
        content: "👍",
        createdAt: 1234567890,
        targetId: "target-event-id",
        event
      });
    });
  });

  describe('without a target id', () => {
    const event: NEvent = {
      id: "test-id",
      pubkey: "test-pubkey",
      created_at: 1234567890,
      kind: 7,
      tags: [
        ["p", "author-pubkey"],
        ["other", "value"]
      ],
      content: "👍",
      sig: "signature"
    };

    it('returns null', () => {
      expect(eventToReaction(event)).toBeNull();
    });
  });

  describe('with empty e tag', () => {
    const event: NEvent = {
      id: "test-id",
      pubkey: "test-pubkey",
      created_at: 1234567890,
      kind: 7,
      tags: [
        ["p", "author-pubkey"],
        ["e"],
        ["other", "value"]
      ],
      content: "👍",
      sig: "signature"
    };

    it('returns null', () => {
      expect(eventToReaction(event)).toBeNull();
    });
  });
});
