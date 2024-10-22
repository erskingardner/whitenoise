<script lang="ts">
    import {
        f7,
        Navbar,
        Link,
        Page,
        Messages,
        Message,
        Messagebar,
        Button,
        Popup,
    } from "framework7-svelte";
    import Avatar from "../components/Avatar.svelte";
    import Name from "../components/Name.svelte";
    import type { NChat, NEvent, EnrichedContact, NostrMlsGroup } from "../types/nostr";
    import { currentIdentity } from "../stores/accounts";
    import { formatMessageTime } from "../utils/time";
    import { Warning, WarningOctagon, ArrowCircleUp } from "phosphor-svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { nameFromMetadata, isInsecure } from "../utils/nostr";
    import { hexMlsGroupId } from "../utils/group";
    import type { Router as F7Router } from "framework7/types";

    let { pubkey, chat, f7router }: { pubkey: string; chat: NChat; f7router: F7Router.Router } =
        $props();

    let enrichedContact: EnrichedContact | undefined = $state(undefined);
    let groupError: string | undefined = $state(undefined);

    $effect(() => {
        invoke("get_contact", { pubkey }).then((value) => {
            enrichedContact = value as EnrichedContact;
        });
    });

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

    function messageTypeForEvent(event: NEvent) {
        return $currentIdentity === event.pubkey ? "sent" : "received";
    }

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

    // Add state for controlling the popup
    let isNewChatPopupOpen = $state(false);

    async function startSecureChat() {
        console.log("Starting secure chat with: ", nameFromMetadata(chat.metadata));
        // This will fetch prekeys, validate them, create the group, and invite the other party
        invoke("create_group", {
            creatorPubkey: $currentIdentity,
            memberPubkeys: [pubkey],
            adminPubkeys: [$currentIdentity, pubkey],
            groupName: "Secure DM",
            description: "",
        })
            .then((group) => {
                const nostrMlsGroup = group as NostrMlsGroup;
                console.log("Group created & welcome message sent");
                f7.popup.close("#new-chat-popup");
                setTimeout(() => {
                    f7router.back();
                }, 150);
                setTimeout(() => {
                    f7router.navigate(`/groups/${hexMlsGroupId(nostrMlsGroup.mls_group_id)}/`, {
                        props: { group: nostrMlsGroup },
                    });
                }, 150);
            })
            .catch((error) => {
                console.error(error);
                groupError = error;
            });
    }

    function inviteToWhiteNoise() {
        console.log("Invite to White Noise");
        isNewChatPopupOpen = false;
        messageText =
            "Hey! I'd love to chat securely with you on White Noise. Download it here: https://whitenoise.chat";

        // TODO: Send message to other user
    }
</script>

<Page
    class="messages-page bg-gray-900"
    noToolbar
    messagesContent
    on:pageAfterIn={async () => {
        console.log("pageAfterIn: LegacyChat");
        warningTooltip = f7.tooltip.create({
            targetEl: ".warning-tooltip",
            text: "This is a NIP-04 encrypted message.<br /><em>All metadata is publicly visible.</em>",
        });
    }}
    on:pageBeforeRemove={() => {
        console.log("pageBeforeRemove: LegacyChat");
        if (warningTooltip) f7.tooltip.destroy(warningTooltip);
    }}
>
    <Navbar class="messages-navbar justify-start py-8" backLink backLinkUrl="/chats/">
        <Link
            href={`/chats/${pubkey}/info/`}
            slot="title"
            class="title-profile-link flex flex-row gap-2 items-center"
            routeProps={{
                chat,
                enrichedContact,
            }}
        >
            <Avatar picture={chat.metadata.picture} {pubkey} pxSize={32} />
            <div class="flex flex-col">
                <Name {pubkey} metadata={chat.metadata} />
            </div>
        </Link>

        <Button
            slot="right"
            raised
            fill
            small
            color="red"
            class="flex flex-row gap-2 items-center {enrichedContact?.nip104 ? '' : 'hidden'}"
            on:click={() => (isNewChatPopupOpen = true)}
        >
            <Warning size={18} class="text-white" />
            <span>Insecure</span>
        </Button>
    </Navbar>

    <Messagebar
        placeholder="Type a message&hellip;"
        bind:value={messageText}
        textareaId="messageTextarea"
        resizable={false}
        class="pl-4 py-1 border-t border-gray-700"
    >
        <a href="/" class="link icon-only" slot="inner-end" onclick={sendMessage}>
            <ArrowCircleUp size={36} weight="fill" />
        </a>
    </Messagebar>

    <Messages class="pt-8">
        {#if chat.events.length === 0}
            <div
                class="text-center text-2xl text-gray-400 mt-40 items-center flex flex-col gap-2 justify-center"
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
        {#if chat.events.length === 0 || isInsecure(chat.events[0])}
            <div
                class="text-center text-gray-400 mt-20 mb-10 items-center flex flex-col gap-2 justify-center"
            >
                <span class="text-red-500 flex flex-row gap-2 items-center">
                    <Warning size={18} class="warning-tooltip text-red-500" />
                    <span>
                        This is an <span class="font-semibold underline">insecure</span> chat that leaks
                        metadata.
                    </span>
                </span>
                <span>
                    Would you like to start a secure chat with {nameFromMetadata(chat.metadata)}?
                </span>
                <Button tonal raised large class="mt-4" on:click={() => (isNewChatPopupOpen = true)}
                    >Start a secure chat</Button
                >
            </div>
        {/if}
    </Messages>

    <Popup
        push
        id="new-chat-popup"
        bind:opened={isNewChatPopupOpen}
        onPopupClosed={() => {
            groupError = undefined;
            isNewChatPopupOpen = false;
        }}
    >
        <Page>
            <Navbar title="Start a Secure Chat with {nameFromMetadata(chat.metadata)}">
                <Button slot="right" popupClose>Close</Button>
            </Navbar>
            <div class="block">
                {#if enrichedContact?.nip104}
                    <p class="my-4 text-base">
                        This chat is insecure because it's using a combination of NIP-04 and NIP-17
                        messaging.
                    </p>
                    <p class="my-4 text-base">
                        Secure chats use Messaging Layer Security (MLS) for post compromise security
                        and forward secrecy. The encrypted conversation is stored on a per device
                        basis (like Signal).
                    </p>
                    <p class="my-4 text-base">
                        To see your secure conversations on another device or client you'll need to
                        add that device separately.
                    </p>
                    <Button fill on:click={startSecureChat}>I understand, let's go!</Button>
                    {#if groupError}
                        <div class="my-4 text-base text-red-500 flex flex-row gap-2 items-center">
                            <WarningOctagon size={36} class="warning-tooltip text-red-500" />
                            <span>{groupError}</span>
                        </div>
                    {/if}
                {:else}
                    <p class="my-4 text-base">
                        It doesn't look like {nameFromMetadata(chat.metadata)} has published a key package
                        yet. Key packages are needed in order to start a secure chat.
                    </p>
                    <p class="my-4 text-base">
                        Do you want to invite them to use White Noise and we'll help them get set
                        up?
                    </p>
                    <Button fill on:click={inviteToWhiteNoise}
                        >Invite {nameFromMetadata(chat.metadata)} to White Noise</Button
                    >
                {/if}
            </div>
        </Page>
    </Popup>
</Page>
