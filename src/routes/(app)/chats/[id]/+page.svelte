<script lang="ts">
import { page } from "$app/state";
import GroupAvatar from "$lib/components/GroupAvatar.svelte";
import HeaderToolbar from "$lib/components/HeaderToolbar.svelte";
import MessageBar from "$lib/components/MessageBar.svelte";
import { activeAccount } from "$lib/stores/accounts";
import {
    type EnrichedContact,
    type NEvent,
    type NostrMlsGroup,
    NostrMlsGroupType,
} from "$lib/types/nostr";
import { hexMlsGroupId } from "$lib/utils/group";
import { nameFromMetadata } from "$lib/utils/nostr";
import { formatMessageTime } from "$lib/utils/time";
import { invoke } from "@tauri-apps/api/core";
import { type UnlistenFn, listen } from "@tauri-apps/api/event";
import {
    ArrowBendUpLeft,
    CaretLeft,
    CheckCircle,
    CircleDashed,
    CopySimple,
    PencilSimple,
    TrashSimple,
} from "phosphor-svelte";
import { onDestroy, onMount } from "svelte";
import { type PressCustomEvent, press } from "svelte-gestures";

let unlistenMlsMessageReceived: UnlistenFn;
let unlistenMlsMessageProcessed: UnlistenFn;

let group: NostrMlsGroup | undefined = $state(undefined);
let counterpartyPubkey: string | undefined = $state(undefined);
let enrichedCounterparty: EnrichedContact | undefined = $state(undefined);
let groupName = $state("");
let messages: NEvent[] = $state([]);
let showMessageMenu = $state(false);
let selectedMessageId: string | null | undefined = $state(undefined);
let messageMenuPosition = $state({ x: 0, y: 0 });
let messageMenuExtendedPosition = $state({ x: 0, y: 0 });

$effect(() => {
    if (
        group &&
        group.group_type === NostrMlsGroupType.DirectMessage &&
        counterpartyPubkey &&
        enrichedCounterparty
    ) {
        groupName = nameFromMetadata(enrichedCounterparty.metadata, counterpartyPubkey);
    } else if (group) {
        groupName = group.name;
    }
});

async function loadGroup() {
    invoke("get_group_and_messages", { groupId: page.params.id }).then((groupResponse) => {
        [group, messages] = groupResponse as [NostrMlsGroup, NEvent[]];
        if (!counterpartyPubkey) {
            counterpartyPubkey =
                group.group_type === NostrMlsGroupType.DirectMessage
                    ? group.admin_pubkeys.filter((pubkey) => pubkey !== $activeAccount?.pubkey)[0]
                    : undefined;
        }
        if (counterpartyPubkey) {
            invoke("query_enriched_contact", {
                pubkey: counterpartyPubkey,
                updateAccount: false,
            }).then((value) => {
                enrichedCounterparty = value as EnrichedContact;
            });
        }
    });
    scrollToBottom();
}

function scrollToBottom() {
    const messagesContainer = document.getElementById("messagesContainer");
    const screenHeight = window.innerHeight;
    if (messagesContainer && screenHeight < messagesContainer.scrollHeight) {
        const lastMessage = messagesContainer.lastElementChild;
        lastMessage?.scrollIntoView({ behavior: "instant" });
    }
    if (messagesContainer) {
        messagesContainer.style.opacity = "1";
    }
}

onMount(async () => {
    if (!unlistenMlsMessageProcessed) {
        unlistenMlsMessageProcessed = await listen<[NostrMlsGroup, NEvent]>(
            "mls_message_processed",
            ({ payload: [_updatedGroup, message] }) => {
                console.log("mls_message_processed event received", message.content);
                if (!messages.some((m) => m.id === message.id)) {
                    console.log("pushing message to transcript");
                    messages = [...messages, message].sort((a, b) => a.created_at - b.created_at);
                }
                scrollToBottom();
            }
        );
    }

    if (!unlistenMlsMessageReceived) {
        unlistenMlsMessageReceived = await listen<NEvent>(
            "mls_message_received",
            ({ payload: _message }) => {
                console.log("mls_message_received event received");
                loadGroup();
            }
        );
    }

    await loadGroup();
});

function handleNewMessage(message: NEvent, replaceTemp: boolean) {
    if (replaceTemp) {
        messages = messages.filter((event) => event.id !== "temp");
    }
    messages = [...messages, message].sort((a, b) => a.created_at - b.created_at);
    scrollToBottom();
}

function handlePress(event: PressCustomEvent) {
    const target = event.target as HTMLElement;
    const messageContainer = target.closest("[data-message-container]");
    const messageId = messageContainer?.getAttribute("data-message-id");
    const isCurrentUser = messageContainer?.getAttribute("data-is-current-user") === "true";
    selectedMessageId = messageId;
    const rect = target.getBoundingClientRect();

    // Temporarily make menus visible but with measuring class
    const reactionMenu = document.getElementById("messageMenu");
    const extendedMenu = document.getElementById("messageMenuExtended");
    if (reactionMenu) reactionMenu.classList.replace("invisible", "visible");
    if (extendedMenu) extendedMenu.classList.replace("invisible", "visible");

    // Add measuring class
    if (reactionMenu) reactionMenu.classList.add("measuring");
    if (extendedMenu) extendedMenu.classList.add("measuring");

    // Use setTimeout to ensure the menus are rendered before measuring
    setTimeout(() => {
        const reactionMenuWidth = reactionMenu?.getBoundingClientRect().width || 0;
        const extendedMenuWidth = extendedMenu?.getBoundingClientRect().width || 0;

        // Remove measuring class
        if (reactionMenu) reactionMenu.classList.remove("measuring");
        if (extendedMenu) extendedMenu.classList.remove("measuring");

        // Calculate positions relative to the message's right edge for current user
        // or left edge for other users
        const messageRight = rect.right;
        const messageLeft = rect.left;
        const messageTop = rect.top;
        const messageBottom = rect.bottom;

        messageMenuPosition = {
            x: isCurrentUser ? messageRight - reactionMenuWidth : messageLeft,
            y: messageTop - 60,
        };

        messageMenuExtendedPosition = {
            x: isCurrentUser ? messageRight - extendedMenuWidth : messageLeft,
            y: messageBottom + 10,
        };

        // Show the menu
        showMessageMenu = true;

        // Existing animation code
        target.style.transform = "scale(1.10)";
        target.style.transformOrigin = "right";
        target.style.transition = "transform 0.10s ease-out";

        setTimeout(() => {
            target.style.transform = "scale(1)";
        }, 100);

        target.addEventListener(
            "pointerup",
            () => {
                target.style.transform = "scale(1)";
            },
            { once: true }
        );
    }, 0);
}

function handleOutsideClick() {
    showMessageMenu = false;
    selectedMessageId = undefined;
}

async function sendReaction(reaction: string) {
    console.log("sending reaction", reaction);
    const reactionResp = await invoke("send_reaction", {
        groupId: page.params.id,
        reaction: reaction,
        messageId: selectedMessageId,
    });
}

function copyMessage() {
    const message = messages.find((m) => m.id === selectedMessageId);
    if (message) {
        navigator.clipboard.writeText(message.content);
        const button = document.querySelector("[data-copy-button]");
        button?.classList.add("copy-success");
        setTimeout(() => {
            button?.classList.remove("copy-success");
        }, 1000);
    }
}

function replyToMessage() {
    console.log("replying to message");
}

function editMessage() {
    console.log("editing message");
}

function deleteMessage() {
    console.log("deleting message");
}

onDestroy(() => {
    unlistenMlsMessageProcessed();
    unlistenMlsMessageReceived();
});
</script>

{#if group}
    <HeaderToolbar alwaysShowCenter={true}>
        {#snippet center()}
            <a href={`/chats/${hexMlsGroupId(group!.mls_group_id)}/info`} class="flex flex-row items-center gap-2">
                <GroupAvatar
                    groupType={group!.group_type}
                    {groupName}
                    {counterpartyPubkey}
                    {enrichedCounterparty}
                    pxSize={30}
                />
                {groupName}
            </a>
        {/snippet}
        {#snippet left()}
            <button onclick={() => window.history.back()} class="p-2 -mr-2">
                <CaretLeft size={30} />
            </button>
        {/snippet}
    </HeaderToolbar>

    <main id="mainContainer" class="flex flex-col relative min-h-dvh">
        <div
            id="messagesContainer"
            class="flex-1 px-8 flex flex-col gap-2 pt-10 pb-40 overflow-y-auto opacity-100 transition-opacity ease-in-out duration-50"
        >
            {#each messages as message (message.id)}
                <div
                    class={`flex ${message.pubkey === $activeAccount?.pubkey ? "justify-end" : "justify-start"}`}
                >
                    <div
                        use:press={()=>({ triggerBeforeFinished: true, timeframe: 300 })}
                        onpress={handlePress}
                        data-message-container
                        data-message-id={message.id}
                        data-is-current-user={message.pubkey === $activeAccount?.pubkey}
                        class={`max-w-[70%] rounded-lg ${message.pubkey === $activeAccount?.pubkey ? "bg-chat-bg-me text-gray-50 rounded-br" : "bg-chat-bg-other text-gray-50 rounded-bl"} p-3 ${showMessageMenu && message.id === selectedMessageId ? 'relative z-20' : ''}`}
                    >
                        <div class="flex {message.content.trim().length < 50 ? "flex-row gap-6" : "flex-col gap-2 justify-end w-full"} items-end">
                            <div class="break-words">
                                {#if message.content.trim().length > 0}
                                    {message.content}
                                {:else}
                                    <span class="italic opacity-60">No message content</span>
                                {/if}
                            </div>
                            <div class={`flex flex-row gap-2 items-center ${message.pubkey === $activeAccount?.pubkey ? "text-gray-300" : "text-gray-400"} ${message.content.trim().length < 50 ? "flex-shrink-0" : "justify-end w-full shrink"}`}>
                                {#if message.id !== "temp"}
                                    <span><CheckCircle size={18} weight="light" /></span>
                                {:else}
                                    <span><CircleDashed size={18} weight="light" class="animate-spin-slow"/></span>
                                {/if}
                                <span class="text-sm opacity-60 whitespace-nowrap">
                                    {formatMessageTime(message.created_at)}
                                </span>
                            </div>
                        </div>
                    </div>
                </div>
            {/each}
        </div>
        <MessageBar {group} {handleNewMessage} />
    </main>
{/if}

{#if showMessageMenu}
    <button
        type="button"
        class="fixed inset-0 backdrop-blur-sm z-10"
        onclick={handleOutsideClick}
        onkeydown={(e) => e.key === 'Escape' && handleOutsideClick()}
        aria-label="Close message menu"
    ></button>
{/if}

<div
    id="messageMenu"
    class="{showMessageMenu ? 'visible' : 'invisible'} fixed bg-gray-800/60 backdrop-blur-sm drop-shadow-md drop-shadow-black py-1 px-2 rounded-full ring-1 ring-gray-700 z-30 translate-x-0"
    style="left: {messageMenuPosition.x}px; top: {messageMenuPosition.y}px;"
    role="menu"
>
    <div class="flex flex-row gap-3 text-xl">
        <button onclick={() => sendReaction("❤️")} class="p-3">❤️</button>
        <button onclick={() => sendReaction("👍")} class="p-3">👍</button>
        <button onclick={() => sendReaction("👎")} class="p-3">👎</button>
        <button onclick={() => sendReaction("😂")} class="p-3">😂</button>
        <button onclick={() => sendReaction("🤔")} class="p-3">🤔</button>
        <button onclick={() => sendReaction("🤙")} class="p-3">🤙</button>
        <button onclick={() => sendReaction("😥")} class="p-3">😥</button>
    </div>
</div>

<div
    id="messageMenuExtended"
    class="{showMessageMenu ? 'visible' : 'invisible'} fixed bg-gray-800/60 backdrop-blur-sm drop-shadow-md drop-shadow-black rounded-md ring-1 ring-gray-700 z-30 translate-x-0"
    style="left: {messageMenuExtendedPosition.x}px; top: {messageMenuExtendedPosition.y}px;"
    role="menu"
>
    <div class="flex flex-col gap-2 justify-start items-between divide-y divide-gray-800">
        <button data-copy-button onclick={copyMessage} class="px-4 py-2 flex flex-row gap-20 items-center justify-between">Copy <CopySimple size={20} /></button>
        <!-- <button onclick={replyToMessage} class="px-4 py-2 flex flex-row gap-20 items-center justify-between">Reply <ArrowBendUpLeft size={20} /></button>
        <button onclick={editMessage} class="px-4 py-2 flex flex-row gap-20 items-center justify-between">Edit <PencilSimple size={20} /></button>
        <button onclick={deleteMessage} class="text-red-500 px-4 py-2 flex flex-row gap-20 items-center justify-between">Delete <TrashSimple size={20} /></button> -->
    </div>
</div>

<style>
    .measuring {
        position: fixed !important;
        visibility: hidden !important;
        top: -9999px !important;
        left: -9999px !important;
    }

    .copy-success {
        color: rgb(34 197 94); /* text-green-500 */
        transition: color 0.2s ease-in-out;
    }
</style>

