import { describe, it, expect } from "vitest";
import { messageHasDeletionTag } from "../message";
import type { NEvent } from "$lib/types/nostr";

describe("messageHasDeletionTag", () => {
    // Mock message data for testing
    const messageId = "mock-message-id";
    const otherMessageId = "other-message-id";

    const mockDeleteEvent: NEvent = {
        id: "delete-event-id",
        pubkey: "pubkey123",
        created_at: 1234567890,
        kind: 5, // Deletion event
        tags: [["e", messageId]], // References the message to delete
        content: "Delete this message",
        sig: "signature",
    };

    const regularMessages: NEvent[] = [
        {
            id: messageId,
            pubkey: "pubkey123",
            created_at: 1234567890,
            kind: 9,
            tags: [],
            content: "Hello world",
            sig: "signature",
        },
        {
            id: otherMessageId,
            pubkey: "pubkey456",
            created_at: 1234567891,
            kind: 9,
            tags: [],
            content: "Another message",
            sig: "signature",
        },
    ];

    it("should return true when a message has a deletion event", () => {
        const messages = [...regularMessages, mockDeleteEvent];
        const result = messageHasDeletionTag(messageId, messages);
        expect(result).toBe(true);
    });

    it("should return false when a message has no deletion event", () => {
        const messages = [...regularMessages]; // No deletion event included
        const result = messageHasDeletionTag(messageId, messages);
        expect(result).toBe(false);
    });

    it("should return false for a different message ID", () => {
        const messages = [...regularMessages, mockDeleteEvent];
        const result = messageHasDeletionTag(otherMessageId, messages);
        expect(result).toBe(false);
    });

    it("should handle empty message arrays", () => {
        const result = messageHasDeletionTag(messageId, []);
        expect(result).toBe(false);
    });

    it("should handle deletion events with multiple tags", () => {
        const complexDeleteEvent: NEvent = {
            id: "complex-delete-id",
            pubkey: "pubkey789",
            created_at: 1234567892,
            kind: 5,
            tags: [
                ["p", "pubkey123"],
                ["e", otherMessageId], // References a different message
                ["e", messageId], // References our target message
                ["other", "value"],
            ],
            content: "Delete multiple messages",
            sig: "signature",
        };

        const messages = [...regularMessages, complexDeleteEvent];
        const result = messageHasDeletionTag(messageId, messages);
        expect(result).toBe(true);
    });

    it("should handle events with 'e' tags that are not deletion events", () => {
        const nonDeletionEvent: NEvent = {
            id: "non-deletion-id",
            pubkey: "pubkey789",
            created_at: 1234567893,
            kind: 1, // Not a deletion event
            tags: [["e", messageId]], // Has the reference but wrong kind
            content: "This references but doesn't delete",
            sig: "signature",
        };

        const messages = [...regularMessages, nonDeletionEvent];
        const result = messageHasDeletionTag(messageId, messages);
        expect(result).toBe(false);
    });
});
