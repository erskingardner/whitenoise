import { describe, it, expect } from "vitest";
import { eventToDeletion } from "../deletion";
import type { NEvent } from "$lib/types/nostr";

describe("eventToDeletion", () => {
  describe('with valid target id in e tag', () => {
    const event: NEvent = {
      id: "deletion-event-id",
      pubkey: "author-pubkey",
      created_at: 1234567890,
      kind: 5,
      tags: [
        ["p", "some-pubkey"],
        ["e", "target-event-id"],
        ["other", "value"]
      ],
      content: "Delete this event",
      sig: "signature"
    };
    
    it('returns a valid Deletion object', () => {  
      const deletion = eventToDeletion(event);
      expect(deletion).toEqual({
        id: "deletion-event-id",
        pubkey: "author-pubkey",
        targetId: "target-event-id",
        event: event
      });
    });
  });

  describe('without e tag', () => {
    const event: NEvent = {
      id: "deletion-event-id",
      pubkey: "author-pubkey",
      created_at: 1234567890,
      kind: 5,
      tags: [
        ["p", "some-pubkey"],
        ["other", "value"]
      ],
      content: "Delete this event",
      sig: "signature"
    };

    it('returns null', () => {
      const deletion = eventToDeletion(event);
      expect(deletion).toBeNull();
    });
  });

  describe('with empty e tag', () => {
    const event: NEvent = {
      id: "deletion-event-id",
      pubkey: "author-pubkey",
      created_at: 1234567890,
      kind: 5,
      tags: [
        ["p", "some-pubkey"],
        ["e"],
        ["other", "value"]
      ],
      content: "Delete this event",
      sig: "signature"
    };
    
    it('returns null', () => {
      const deletion = eventToDeletion(event);
      expect(deletion).toBeNull();
    });
  });
}); 
