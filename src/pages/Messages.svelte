<script lang="ts">
    import { f7, Navbar, Link, Page, Messages, Message, Messagebar, Icon } from "framework7-svelte";
    import Avatar from "../components/Avatar.svelte";
    import Name from "../components/Name.svelte";
    import type { NMetadata, NEvent } from "../types/nostr";
    import { currentIdentity } from "../stores/accounts";
    import { formatMessageTime } from "../utils/time";
    import { Warning } from "phosphor-svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { nameFromMetadata, isInsecure } from "../utils/nostr";

    type Chat = {
        latest: number | undefined;
        metadata: NMetadata;
        events: NEvent[];
    };

    let { pubkey, chat }: { pubkey: string; chat: Chat } = $props();

    let messageText = $state("");

    async function sendMessage(e: MouseEvent) {
        e.preventDefault();
        if (messageText.length === 0) return;
        const event_id = await invoke("send_message", {
            pubkey: pubkey,
            message: messageText,
        });

        // Clear the message input
        messageText = "";
    }

    let warningTooltip: any;

    function onPageInit() {
        warningTooltip = f7.tooltip.create({
            targetEl: ".warning-tooltip",
            text: "This is a NIP-04 encrypted message.<br /><em>All metadata is publicly visible.</em>",
        });
    }

    function onPageBeforeRemove() {
        if (warningTooltip) f7.tooltip.destroy(warningTooltip);
    }

    function messageTypeForEvent(event: NEvent) {
        return $currentIdentity === event.pubkey ? "sent" : "received";
    }

    const messageTime = (event: NEvent) =>
        Intl.DateTimeFormat("en", { hour: "numeric", minute: "numeric" }).format(event.created_at);
    const isMessageFirst = (event: NEvent) => {
        const messageIndex = chat.events.indexOf(event);
        const previousMessage = chat.events[messageIndex - 1];
        return (
            !previousMessage || messageTypeForEvent(previousMessage) !== messageTypeForEvent(event)
        );
    };
    const isMessageLast = (event: NEvent) => {
        const messageIndex = chat.events.indexOf(event);
        const nextMessage = chat.events[messageIndex + 1];
        return !nextMessage || messageTypeForEvent(nextMessage) !== messageTypeForEvent(event);
    };

    // TODO: Add method for creating a new 104 chat
    // 1. Create group using current user details
    // 2. Fetch and parse prekey of other user - display error if there isn't a prekey
    // 3. Create group, invite the other party, send welcome message
    //      - This group needs to be saved to the database
    //      - We need to have a method for fetching groups from database/backend
    // 4. Create chat in the UI, move the view to that chat.
</script>

<Page class="messages-page bg-gray-900" noToolbar messagesContent {onPageInit} {onPageBeforeRemove}>
    <Navbar class="messages-navbar justify-start py-8" backLink backLinkShowText={false}>
        <Link
            href={`/profile/${pubkey}/`}
            slot="title"
            class="title-profile-link flex flex-row gap-2 items-center"
        >
            <Avatar picture={chat.metadata.picture} {pubkey} pxSize={32} />
            <div class="flex flex-col">
                <Name {pubkey} metadata={chat.metadata} />
                <div class="subtitle">online</div>
            </div>
        </Link>
        <!-- TODO: Add button to create 104 chat -->
    </Navbar>

    <Messagebar
        placeholder="Type a message&hellip;"
        bind:value={messageText}
        textareaId="messageTextarea"
        resizable={false}
    >
        <a href="/" class="link icon-only" slot="inner-end" onclick={sendMessage}>
            <Icon ios="f7:arrow_up_circle_fill" md="material:send" size={36} />
        </a>
    </Messagebar>

    <Messages class="pt-8">
        {#if chat.events.length === 0}
            <div
                class="text-center text-gray-400 mt-40 items-center flex flex-col gap-2 justify-center"
            >
                <span>GM {nameFromMetadata(chat.metadata)}!</span>
            </div>
        {/if}
        {#each chat.events as event, index}
            <Message
                data-key={index}
                first={isMessageFirst(event)}
                last={isMessageLast(event)}
                tail={isMessageLast(event)}
                type={messageTypeForEvent(event)}
                class="message-appear-from-bottom"
            >
                <span slot="text">
                    {#if event.content.length > 0}
                        {event.content}
                    {:else}
                        <span class="italic text-gray-300">No content</span>
                    {/if}
                </span>
                <span
                    slot="text-footer"
                    class="mt-1 mb-0 text-sm p-0 flex flex-row gap-2 justify-end"
                >
                    {#if event.kind === 4}
                        <Warning weight="fill" size={18} class="warning-tooltip text-yellow-400" />
                    {/if}
                    {formatMessageTime(event.created_at)}
                </span>
            </Message>
        {/each}
    </Messages>
</Page>
