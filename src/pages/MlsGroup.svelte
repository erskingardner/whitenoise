<script lang="ts">
    import { f7, Navbar, Link, Page, Messages, Message, Messagebar } from "framework7-svelte";
    import GroupAvatar from "../components/GroupAvatar.svelte";
    import type { NEvent, EnrichedContact, NostrMlsGroup } from "../types/nostr";
    import { NostrMlsGroupType } from "../types/nostr";
    import { currentIdentity } from "../stores/accounts";
    import { formatMessageTime } from "../utils/time";
    import { Warning, ArrowCircleUp } from "phosphor-svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { nameFromMetadata, isInsecure } from "../utils/nostr";
    import { hexMlsGroupId } from "../utils/group";

    let { group }: { group: NostrMlsGroup } = $props();

    let counterpartyPubkey =
        group.group_type === NostrMlsGroupType.DirectMessage
            ? group.admin_pubkeys.filter((pubkey) => pubkey !== $currentIdentity)[0]
            : undefined;

    let enrichedCounterparty: EnrichedContact | undefined = $state(undefined);
    let groupName = $state("");

    $effect(() => {
        if (
            group.group_type === NostrMlsGroupType.DirectMessage &&
            counterpartyPubkey &&
            enrichedCounterparty
        ) {
            groupName = nameFromMetadata((enrichedCounterparty as EnrichedContact).metadata);
        } else {
            groupName = group.group_name;
        }
    });

    $effect(() => {
        if (counterpartyPubkey) {
            invoke("get_contact", { pubkey: counterpartyPubkey }).then((value) => {
                enrichedCounterparty = value as EnrichedContact;
            });
        }
    });

    let messageText = $state("");

    async function sendMessage(e: MouseEvent) {
        e.preventDefault();
        if (messageText.length === 0) return;

        // TODO: Send message to the MLS group
        console.log("Send Message", messageText);

        // Clear the message input
        messageText = "";
    }

    function messageTypeForEvent(event: NEvent) {
        return $currentIdentity === event.pubkey ? "sent" : "received";
    }

    const isMessageFirst = (event: NEvent) => {
        const messageIndex = group.transcript.indexOf(event);
        const previousMessage = group.transcript[messageIndex - 1];
        return (
            !previousMessage || messageTypeForEvent(previousMessage) !== messageTypeForEvent(event)
        );
    };
    const isMessageLast = (event: NEvent) => {
        const messageIndex = group.transcript.indexOf(event);
        const nextMessage = group.transcript[messageIndex + 1];
        return !nextMessage || messageTypeForEvent(nextMessage) !== messageTypeForEvent(event);
    };

    // Add state for controlling the popup
    let isNewChatPopupOpen = $state(false);
</script>

<Page class="messages-page bg-gray-900" noToolbar messagesContent>
    <Navbar class="messages-navbar justify-start py-8" backLink backLinkShowText={false}>
        <Link
            href={`/groups/${hexMlsGroupId(group.mls_group_id)}/group_info/`}
            slot="title"
            class="title-profile-link flex flex-row gap-2 items-center"
            routeProps={{
                group,
            }}
        >
            <GroupAvatar
                groupType={group.group_type}
                {groupName}
                {counterpartyPubkey}
                {enrichedCounterparty}
                pxSize={32}
            />
            <div class="flex flex-col">
                {groupName}
            </div>
        </Link>
    </Navbar>

    <Messagebar
        placeholder="Type a message&hellip;"
        bind:value={messageText}
        textareaId="messageTextarea"
        resizable={false}
    >
        <a href="/" class="link icon-only" slot="inner-end" onclick={sendMessage}>
            <ArrowCircleUp size={36} weight="fill" />
        </a>
    </Messagebar>

    <Messages class="pt-8">
        {#if group.transcript.length === 0}
            <div
                class="text-center text-2xl text-gray-400 mt-40 items-center flex flex-col gap-2 justify-center"
            >
                <span>GM {groupName}!</span>
            </div>
        {/if}
        {#each group.transcript as event, index}
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
                        <span class="italic">No content</span>
                    {/if}
                </span>
                <span
                    slot="text-footer"
                    class="mt-1 mb-0 text-sm p-0 flex flex-row gap-2 justify-end
                    {event.pubkey === $currentIdentity && f7.theme !== 'ios'
                        ? 'text-blue-800/70'
                        : 'text-gray-400'}
                    {event.pubkey === $currentIdentity && f7.theme === 'ios'
                        ? 'text-gray-100'
                        : ''}"
                >
                    {#if event.kind === 4}
                        <Warning weight="fill" size={18} class="warning-tooltip" />
                    {/if}
                    {formatMessageTime(event.created_at)}
                </span>
            </Message>
        {/each}
    </Messages>
</Page>
