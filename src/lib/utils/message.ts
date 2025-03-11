import type { NEvent } from '$lib/types/nostr';
import type { Message } from '$lib/types/chat';
import { findReplyToId } from './tags';
import { eventToLightningInvoice, eventToLightningPayment } from './lightning';

function isSingleEmoji(str: string) {
    const trimmed = str.trim();
    const emojiRegex =
        /^(?:\p{Emoji_Presentation}|\p{Emoji}\uFE0F)\p{Emoji_Modifier}*(?:\u200D(?:\p{Emoji_Presentation}|\p{Emoji}\uFE0F)\p{Emoji_Modifier}*)*$/u;
    return emojiRegex.test(trimmed);
}

function contentToShow(
    { content, invoice }:
    { content: string, invoice: string | undefined }
) {
    if (!invoice) return content;
    const firstPart = invoice.substring(0, 15);
    const lastPart = invoice.substring(invoice.length - 15);
    return content.replace(invoice, `${firstPart}...${lastPart}`);
}

export function eventToMessage(event: NEvent, currentPubkey: string | undefined): Message {
    const replyToId = findReplyToId(event);
    const isMine = currentPubkey === event.pubkey;
    const lightningInvoice = eventToLightningInvoice(event);
    const lightningPayment = eventToLightningPayment(event);
    const content = contentToShow({ content: event.content, invoice: lightningInvoice?.invoice });

    return {
        id: event.id,
        pubkey: event.pubkey,
        content,
        createdAt: event.created_at,
        replyToId,
        reactions: [],
        lightningInvoice,
        isSingleEmoji: isSingleEmoji(content),
        lightningPayment,
        isMine,
        event
    };
} 
