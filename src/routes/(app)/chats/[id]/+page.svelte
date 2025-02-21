<script lang="ts">
import { goto } from "$app/navigation";
import { page } from "$app/state";
import { getToastState } from "$lib/stores/toast-state.svelte";
import GroupAvatar from "$lib/components/GroupAvatar.svelte";
import HeaderToolbar from "$lib/components/HeaderToolbar.svelte";
import MessageBar from "$lib/components/MessageBar.svelte";
import RepliedTo from "$lib/components/RepliedTo.svelte";
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
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import {
    ArrowBendUpLeft,
    CaretLeft,
    CheckCircle,
    CircleDashed,
    CopySimple,
    DotsThree,
    Lightning,
} from "phosphor-svelte";
import { onDestroy, onMount, tick } from "svelte";
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
let isSelectedMessageBolt11: boolean | null | undefined = $state(false);
let messageMenuPosition = $state({ x: 0, y: 0 });
let messageMenuExtendedPosition = $state({ x: 0, y: 0 });
let replyToMessageEvent: NEvent | undefined = $state(undefined);
let toastState = getToastState();

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
    invoke("get_group_and_messages", { groupId: page.params.id }).then(async (groupResponse) => {
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
        await scrollToBottom();
    });
}

async function scrollToBottom() {
    await tick();
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
function findBolt11Tag(message: NEvent): string | undefined {
    return message.tags.find((t) => t[0] === "bolt11")?.[1];
}

function doesMessageHaveBolt11Tag(message: NEvent): boolean {
    return findBolt11Tag(message) !== undefined;
}

function handlePress(event: PressCustomEvent | MouseEvent) {
    const target = event.target as HTMLElement;
    const messageContainer = target.closest("[data-message-container]");
    const messageId = messageContainer?.getAttribute("data-message-id");
    const isCurrentUser = messageContainer?.getAttribute("data-is-current-user") === "true";
    selectedMessageId = messageId;
    const message = messages.find((m) => m.id === messageId);
    if(message) {
       isSelectedMessageBolt11 = doesMessageHaveBolt11Tag(message);
    }
    const messageBubble = messageContainer?.parentElement?.querySelector(
        "[data-message-container]:not(button)"
    );
    const rect = messageBubble?.getBoundingClientRect() || target.getBoundingClientRect();

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

        // Calculate viewport-relative positions
        const viewportX = isCurrentUser ? rect.right - reactionMenuWidth : rect.left;
        const viewportY = rect.top - 60;

        messageMenuPosition = {
            x: viewportX,
            y: viewportY,
        };

        messageMenuExtendedPosition = {
            x: isCurrentUser ? rect.right - extendedMenuWidth : rect.left,
            y: rect.bottom + 10,
        };

        showMessageMenu = true;

        // Apply animation to the message bubble
        if (messageBubble instanceof HTMLElement) {
            messageBubble.style.transform = "scale(1.10)";
            messageBubble.style.transformOrigin = isCurrentUser ? "right" : "left";
            messageBubble.style.transition = "transform 0.10s ease-out";

            setTimeout(() => {
                messageBubble.style.transform = "scale(1)";
            }, 100);

            messageBubble.addEventListener(
                "pointerup",
                () => {
                    messageBubble.style.transform = "scale(1)";
                },
                { once: true }
            );
        }
    }, 0);
}

function handleOutsideClick() {
    showMessageMenu = false;
    selectedMessageId = undefined;
    isSelectedMessageBolt11 = undefined;
}

async function sendReaction(reaction: string, messageId: string | null | undefined) {
    if (!group) {
        console.error("no group found");
        return;
    }
    if (!messageId) {
        console.error("no message selected");
        return;
    }
    const message = messages.find((m) => m.id === messageId);
    if (!message) {
        console.error("message not found");
        return;
    }

    // Filter out tags that are not "e" or "p" (or invalid)
    let tags = message.tags.filter((t) => t.length >= 2 && (t[0] === "e" || t[0] === "p"));
    // Now add our own tags for the reaction
    tags = [...tags, ["e", messageId], ["p", message.pubkey], ["k", message.kind.toString()]];

    console.log("Sending reaction", reaction);
    invoke("send_mls_message", {
        group,
        message: reaction,
        kind: 7,
        tags: tags,
    })
        .then((reactionEvent) => {
            console.log("reaction sent", reactionEvent);
            handleNewMessage(reactionEvent as NEvent, false);
        })
        .finally(() => {
            showMessageMenu = false;
        });
}

async function copyMessage() {
    const message = messages.find((m) => m.id === selectedMessageId);
    if (message) {
        await writeText(message.content);
        const button = document.querySelector("[data-copy-button]");
        button?.classList.add("copy-success");
        setTimeout(() => {
            button?.classList.remove("copy-success");
            showMessageMenu = false;
        }, 1000);
    }
}

async function payInvoice() {
    if (!group) {
        console.error("no group found");
        return;
    }
    if (!selectedMessageId) {
        console.error("no message selected");
        return;
    }
    const message = messages.find((m) => m.id === selectedMessageId);
    if (!message) {
        console.error("message not found");
        return;
    }
    
    if (!isSelectedMessageBolt11) {
        console.error("message is not a bolt11 invoice");
        return;
    }
    const invoice = findBolt11Tag(message);
    // Filter out tags that are not "e" or "p" (or invalid)
    let tags = message.tags.filter((t) => t.length >= 2 && (t[0] === "e" || t[0] === "p"));
    // Now add our own tags for the reaction
    tags = [...tags, ["e", selectedMessageId], ["p", message.pubkey], ["k", message.kind.toString()]];
    console.log("Sending payment", tags);
    invoke("pay_invoice", {
        group,
        tags: tags,
        bolt11: invoice
    })
        .then((reactionEvent) => {
            console.log("reaction sent", reactionEvent);
            toastState.add("Payment success", "Successfully sent payment to invoice", "success");
            handleNewMessage(reactionEvent as NEvent, false);
        }, (e) => {
            toastState.add(
                "Error sending payment",
                `Failed to send payment: ${e.message}`,
                "error"
            );
            console.error("Error sending payment", e);
        })
        .finally(() => {
            showMessageMenu = false;
        });
}

function replyToMessage() {
    replyToMessageEvent = messages.find((m) => m.id === selectedMessageId);
    document.getElementById("newMessageInput")?.focus();
    showMessageMenu = false;
}

function editMessage() {
    console.log("editing message");
}

function deleteMessage() {
    console.log("deleting message");
}

function isSingleEmoji(str: string) {
    const trimmed = str.trim();
    // This regex matches a single emoji (including compound emojis like 👨‍👩‍👧‍👦 or 👨🏻‍💻)
    const emojiRegex =
        /^(?:\p{Emoji_Presentation}|\p{Emoji}\uFE0F)\p{Emoji_Modifier}*(?:\u200D(?:\p{Emoji_Presentation}|\p{Emoji}\uFE0F)\p{Emoji_Modifier}*)*$/u;
    return emojiRegex.test(trimmed);
}

function reactionsForMessage(message: NEvent): { content: string; count: number }[] {
    const reactions = messages.filter(
        (m) => m.kind === 7 && m.tags.some((t) => t[0] === "e" && t[1] === message.id)
    );
    return reactions.reduce(
        (acc, reaction) => {
            const existingReaction = acc.find((r) => r.content === reaction.content);
            if (existingReaction) {
                existingReaction.count++;
            } else {
                acc.push({ content: reaction.content, count: 1 });
            }
            return acc;
        },
        [] as { content: string; count: number }[]
    );
}

onDestroy(() => {
    unlistenMlsMessageProcessed();
    unlistenMlsMessageReceived();
    toastState.cleanup();
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
            <button onclick={() => goto(`/chats`)} class="p-2 -mr-2">
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
                {#if message.kind === 9}
                    <div
                        class={`flex justify-end ${message.pubkey === $activeAccount?.pubkey ? "" : "flex-row-reverse"} items-center gap-4 group ${reactionsForMessage(message).length > 0 ? "mb-6" : ""}`}
                    >
                        <button
                            onclick={handlePress}
                            data-message-container
                            data-message-id={message.id}
                            data-is-current-user={message.pubkey === $activeAccount?.pubkey}
                            class="p-2 opacity-0 group-hover:opacity-100 transition-opacity duration-200"
                        >
                            <DotsThree size="24" weight="bold" />
                        </button>
                        <div
                            use:press={()=>({ triggerBeforeFinished: true, timeframe: 300 })}
                            onpress={handlePress}
                            data-message-container
                            data-message-id={message.id}
                            data-is-current-user={message.pubkey === $activeAccount?.pubkey}
                            class={`relative max-w-[70%] ${!isSingleEmoji(message.content) ? `rounded-lg ${message.pubkey === $activeAccount?.pubkey ? "bg-chat-bg-me text-gray-50 rounded-br" : "bg-chat-bg-other text-gray-50 rounded-bl"} p-3` : ''} ${showMessageMenu && message.id === selectedMessageId ? 'relative z-20' : ''}`}
                        >
                            {#if message.tags.find((t) => t[0] === "q")?.[1]}
                                <RepliedTo messageId={message.tags.find((t) => t[0] === "q")?.[1]} />
                            {/if}
                            <div class="flex {message.content.trim().length < 50 && !isSingleEmoji(message.content) ? "flex-row gap-6" : "flex-col gap-2 justify-end w-full"} items-end {isSingleEmoji(message.content) ? 'mb-4 my-6' : ''}">
                                <div class="break-words-smart {isSingleEmoji(message.content) ? 'text-7xl leading-none' : ''}">
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
                            <div class="reactions flex flex-row gap-2 absolute -bottom-6 right-0">
                                {#each reactionsForMessage(message) as reaction}
                                    <button onclick={() => sendReaction(reaction.content, message.id)} class="py-1 px-2 bg-gray-900 rounded-full flex flex-row gap-1 items-center">
                                        {reaction.content}
                                        {#if reaction.count > 1}
                                            <span class="text-sm opacity-60">{reaction.count}</span>
                                        {/if}
                                    </button>
                                {/each}
                            </div>
                        </div>
                    </div>
                {/if}
            {/each}
        </div>
        <MessageBar {group} bind:replyToMessageEvent={replyToMessageEvent} {handleNewMessage} />
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
    class="{showMessageMenu ? 'visible' : 'invisible'} fixed bg-gray-900/90 backdrop-blur-sm drop-shadow-md drop-shadow-black py-1 px-2 rounded-full ring-1 ring-gray-700 z-30 translate-x-0"
    style="left: {messageMenuPosition.x}px; top: {messageMenuPosition.y}px;"
    role="menu"
>
    <div class="flex flex-row gap-3 text-xl">
        <button onclick={() => sendReaction("❤️", selectedMessageId)} class="p-3">❤️</button>
        <button onclick={() => sendReaction("👍", selectedMessageId)} class="p-3">👍</button>
        <button onclick={() => sendReaction("👎", selectedMessageId)} class="p-3">👎</button>
        <button onclick={() => sendReaction("😂", selectedMessageId)} class="p-3">😂</button>
        <button onclick={() => sendReaction("🤔", selectedMessageId)} class="p-3">🤔</button>
        <button onclick={() => sendReaction("🤙", selectedMessageId)} class="p-3">🤙</button>
        <button onclick={() => sendReaction("😥", selectedMessageId)} class="p-3">😥</button>
    </div>
</div>

<div
    id="messageMenuExtended"
    class="{showMessageMenu ? 'opacity-100 visible' : 'opacity-0 invisible'} fixed bg-gray-900/90 backdrop-blur-sm drop-shadow-md drop-shadow-black rounded-md ring-1 ring-gray-700 z-30 translate-x-0 transition-opacity duration-200"
    style="left: {messageMenuExtendedPosition.x}px; top: {messageMenuExtendedPosition.y}px;"
    role="menu"
>
    <div class="flex flex-col justify-start items-between divide-y divide-gray-800">
        <button data-copy-button onclick={copyMessage} class="px-4 py-2 flex flex-row gap-20 items-center justify-between hover:bg-gray-700">Copy <CopySimple size={20} /></button>
        <button onclick={replyToMessage} class="px-4 py-2 flex flex-row gap-20 items-center justify-between hover:bg-gray-700">Reply <ArrowBendUpLeft size={20} /></button>
        {#if isSelectedMessageBolt11}
            <button onclick={payInvoice} class="glow-button px-4 py-2 flex flex-row gap-20 items-center justify-between hover:bg-gray-700">Pay<Lightning size={20} weight="fill" /></button>
        {/if}
        <!-- <button onclick={editMessage} class="px-4 py-2 flex flex-row gap-20 items-center justify-between">Edit <PencilSimple size={20} /></button>
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

    .glow-button {
        position: relative;
        color: #fff;
        transition: all 0.3s ease;
        background: rgba(173, 0, 255, 0.1);
    }

    .glow-button::before {
        content: '';
        position: absolute;
        inset: -1px;
        background: linear-gradient(90deg, #ff00ea 0%, #ad00ff 100%);
        z-index: -1;
        opacity: 0.15;
        filter: blur(8px);
        border-radius: 0.375rem;
    }

    .glow-button:hover {
        background: rgba(173, 0, 255, 0.2);
    }

    /* Ensure immediate visibility state change */
    .invisible {
        display: none;
    }
</style>
