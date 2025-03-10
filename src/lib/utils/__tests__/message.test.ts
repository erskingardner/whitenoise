import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { messageHasDeletionTag, eventToMessage } from "../message";
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
        expect(result).toEqual(false);
    });
});
describe("eventToMessage", () => {    
    const defaultEvent: NEvent = {
        id: "event123",
        pubkey: "pubkey456",
        created_at: 1622548800,
        kind: 1,
        tags: [],
        content: "Hello world",
        sig: "signature"
    };

    it('converts event to message', () => {                        
        const result = eventToMessage(defaultEvent, 'some-other-pubkey');
        expect(result).toEqual({
            id: "event123",
            pubkey: "pubkey456",
            content: "Hello world",
            createdAt: 1622548800,
            replyToId: undefined,
            reactions: [],
            lightningInvoice: undefined,
            lightningPayment: undefined,
            isSingleEmoji: false,
            isMine: false,
            event: defaultEvent
        });
    });

    describe('with emojis', () => {
        it('returns isSingleEmoji true for a single basic emoji', () => {
             const event = { ...defaultEvent, content :"😊" };
            const message = eventToMessage(event, "some-pubkey")
            expect(message.isSingleEmoji).toEqual(true);
        });
        
        it('returns isSingleEmoji true for a compound emoji', () => {
            const event = { ...defaultEvent, content :"👨‍👩‍👧‍👦" };
            const message = eventToMessage(event, "some-pubkey")
            expect(message.isSingleEmoji).toEqual(true);
        });
        
        it('returns isSingleEmoji true for an emoji with skin tone modifier', () => {
            const event = { ...defaultEvent, content :"👍🏽" };
            const message = eventToMessage(event, "some-pubkey")
            expect(message.isSingleEmoji).toEqual(true);
        });
        
        it('returns isSingleEmoji true for emoji with whitespace', () => {
             const event = { ...defaultEvent, content :" 🎉 "};
            const message = eventToMessage(event, "some-pubkey")
            expect(message.isSingleEmoji).toEqual(true);
        });
        
        it('returns isSingleEmoji false for text with emoji', () => {
            const event = { ...defaultEvent, content :"Hello 👋"};
            const message = eventToMessage(event, "some-pubkey")
            expect(message.isSingleEmoji).toEqual(false);
        });
        
        it('returns isSingleEmoji false for multiple emojis', () => {
             const event = { ...defaultEvent, content :"😊😎"};
            const message = eventToMessage(event, "some-pubkey")
            expect(message.isSingleEmoji).toEqual(false);
        });
    });

    describe('with same pubkey', () => {
        it('returns isMine true', () => {
            const event = { ...defaultEvent, pubkey: "pubkey456" };
            const message = eventToMessage(event, "pubkey456");
            expect(message.isMine).toEqual(true);
        });
    });

    describe('with reply q tag', () => {
        it('returns replyToId from q tag', () => {
            const event = { ...defaultEvent, tags: [["q", "original-event-id"]] };
            const message = eventToMessage(event, "some-pubkey");
            expect(message.replyToId).toEqual("original-event-id");
        });
    });

    describe('with bolt11 tag', () => {
        const invoice = "lntbs330n1pnuu4msdqqnp4qg094pqgshvyfsltrck5lkdw5negkn3zwe36ukdf8zhwfc2h5ay6spp5hhsamdzupvqvygycgk5a37fx94m5qctzhz37sf0tensuje9phxgssp50qpmchgkh5z94gffsq3u9sgyr4l778wzj7x4g2wvwtyghdxmt23s9qyysgqcqpcxqyz5vqfuj3u2u2lcs7wdu6k8jh2vur9l3zmffwfup2k8ea7fgeg2puc6xs9cssqcl0xhzngg8z5ye62h3vcgfve56zd9rum2sygndh66qdehgqm4ajkej";
        const event: NEvent = {
            id: "event123",
            pubkey: "pubkey456",
            created_at: 1622548800,
            kind: 9734,
            tags: [["bolt11", invoice, "1000000", "Invoice description"]],
            content: `Please pay this invoice: ${invoice}`,
            sig: "signature"
        };
        
    
        it("sets lightning invoice", () => {
            const message = eventToMessage(event, "some-pubkey");
            expect(message.lightningInvoice).toMatchObject({
                invoice: 'lntbs330n1pnuu4msdqqnp4qg094pqgshvyfsltrck5lkdw5negkn3zwe36ukdf8zhwfc2h5ay6spp5hhsamdzupvqvygycgk5a37fx94m5qctzhz37sf0tensuje9phxgssp50qpmchgkh5z94gffsq3u9sgyr4l778wzj7x4g2wvwtyghdxmt23s9qyysgqcqpcxqyz5vqfuj3u2u2lcs7wdu6k8jh2vur9l3zmffwfup2k8ea7fgeg2puc6xs9cssqcl0xhzngg8z5ye62h3vcgfve56zd9rum2sygndh66qdehgqm4ajkej',
                amount: 1000,
                description: "Invoice description",
                isPaid: false
            });
        });

        it('shortens lightning invoice in content', () => {
            const message = eventToMessage(event, "some-pubkey");
            expect(message.content).toEqual(
                'Please pay this invoice: lntbs330n1pnuu4...66qdehgqm4ajkej'
            );
        });
    });
    
    describe('with preimage tag', () => {
        const event: NEvent = {
            id: "event123",
            pubkey: "pubkey456",
            created_at: 1622548800,
            kind: 9,
            tags: [["preimage", "preimage123"]],
            content: "Payment sent",
            sig: "signature"
        };
        it("saves lightning payment", () => {
            const message = eventToMessage(event, "some-pubkey");
            
            expect(message.lightningPayment).toEqual({
                preimage: 'preimage123',
                isPaid: false
            });
        });
    });
});
