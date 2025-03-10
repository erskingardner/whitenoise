import { describe, it, expect } from "vitest";
import { findTargetId, findBolt11Tag, findPreimage, findReplyToId } from "../tags";
import type { NEvent } from "$lib/types/nostr";

describe("findTargetId", () => {
  describe('with e tag', () => {
    const event: NEvent = {
      id: "test-id",
      pubkey: "test-pubkey",
      created_at: 1234567890,
      kind: 1,
      tags: [
        ["p", "some-pubkey"],
        ["e", "target-event-id"],
        ["other", "value"]
      ],
      content: "Test content",
      sig: "signature"
    };
    
    it('returns the value of the first "e" tag', () => {  
      expect(findTargetId(event)).toBe("target-event-id");
    });
  });

  describe('without e tag', () => {
    const event: NEvent = {
      id: "test-id",
      pubkey: "test-pubkey",
      created_at: 1234567890,
      kind: 1,
      tags: [
        ["p", "some-pubkey"],
        ["other", "value"]
      ],
      content: "Test content",
      sig: "signature"
    };

    it('returns undefined', () => {
      expect(findTargetId(event)).toBeUndefined();
    });
  });

  describe('without e tag value', () => {
    const event: NEvent = {
      id: "test-id",
      pubkey: "test-pubkey",
      created_at: 1234567890,
      kind: 1,
      tags: [
        ["p", "some-pubkey"],
        ["e"],
        ["other", "value"]
      ],
      content: "Test content",
      sig: "signature"
    };
    it('returns undefined', () => {
      expect(findTargetId(event)).toBeUndefined();
    });
  });
});

describe("findBolt11Tag", () => {
  describe('with bolt11 tag', () => {
    const event: NEvent = {
      id: "test-id",
      pubkey: "test-pubkey",
      created_at: 1234567890,
      kind: 1,
      tags: [
        ["p", "some-pubkey"],
        ["bolt11", "invoice-data", "additional-data"],
        ["other", "value"]
      ],
      content: "Test content",
      sig: "signature"
    };
    it("returns bolt11 tag", () => {
      expect(findBolt11Tag(event)).toEqual(["bolt11", "invoice-data", "additional-data"]);
    });
  });

  describe('without bolt11 tag', () => {
    const event: NEvent = {
      id: "test-id",
      pubkey: "test-pubkey",
      created_at: 1234567890,
      kind: 1,
      tags: [
        ["p", "some-pubkey"],
        ["other", "value"]
      ],
      content: "Test content",
      sig: "signature"
    };
    it('returns undefined', () => {
      expect(findBolt11Tag(event)).toBeUndefined();
    });
  });
});

describe("findPreimage", () => {
  describe('with preimage tag', () => {
    const event: NEvent = {
      id: "test-id",
      pubkey: "test-pubkey",
      created_at: 1234567890,
      kind: 1,
      tags: [
        ["p", "some-pubkey"],
        ["preimage", "preimage-hash-value"],
        ["other", "value"]
      ],
      content: "Test content",
      sig: "signature"
    };
    
    it('returns the preimage value', () => {
      expect(findPreimage(event)).toBe("preimage-hash-value");
    });
  });

  describe('without preimage tag', () => {
    const event: NEvent = {
      id: "test-id",
      pubkey: "test-pubkey",
      created_at: 1234567890,
      kind: 1,
      tags: [
        ["p", "some-pubkey"],
        ["other", "value"]
      ],
      content: "Test content",
      sig: "signature"
    };

    it('returns undefined', () => {
      expect(findTargetId(event)).toBeUndefined();
    });
  });

  describe('without preimage tag value', () => {
    const event: NEvent = {
      id: "test-id",
      pubkey: "test-pubkey",
      created_at: 1234567890,
      kind: 1,
      tags: [
        ["p", "some-pubkey"],
        ["preimage"],
        ["other", "value"]
      ],
      content: "Test content",
      sig: "signature"
    };
    it('returns undefined', () => {
      expect(findTargetId(event)).toBeUndefined();
    });
  });
});

describe("findReplyToId", () => {
  describe('with q tag', () => {
    const event: NEvent = {
      id: "test-id",
      pubkey: "test-pubkey",
      created_at: 1234567890,
      kind: 1,
      tags: [
        ["p", "some-pubkey"],
        ["q", "reply-to-id"],
        ["other", "value"]
      ],
      content: "Test content",
      sig: "signature"
    };
    it('returns value of q tag', () => {

      expect(findReplyToId(event)).toBe("reply-to-id");
    });
  });

  describe('without q tag', () => {
    const event: NEvent = {
      id: "test-id",
      pubkey: "test-pubkey",
      created_at: 1234567890,
      kind: 1,
      tags: [
        ["p", "some-pubkey"],
        ["other", "value"]
      ],
      content: "Test content",
      sig: "signature"
    };

    it('returns undefined', () => {
      expect(findReplyToId(event)).toBeUndefined();
    });
  });

  describe('with q tag but no value', () => {
    const event: NEvent = {
      id: "test-id",
      pubkey: "test-pubkey",
      created_at: 1234567890,
      kind: 1,
      tags: [
        ["p", "some-pubkey"],
        ["q"],
        ["other", "value"]
      ],
      content: "Test content",
      sig: "signature"
    };

    it('returns undefined', () => {
      expect(findReplyToId(event)).toBeUndefined();
    });
  });
}); 
