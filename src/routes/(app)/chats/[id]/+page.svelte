<script lang="ts">
import { page } from "$app/state";
import GroupAvatar from "$lib/components/GroupAvatar.svelte";
import HeaderToolbar from "$lib/components/HeaderToolbar.svelte";
import MessageBar from "$lib/components/MessageBar.svelte";
import { accounts } from "$lib/stores/accounts";
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
import { CaretLeft, CheckCircle, CircleDashed } from "phosphor-svelte";
import { onMount } from "svelte";

let unlistenMlsMessageReceived: UnlistenFn;
let unlistenMlsMessageProcessed: UnlistenFn;

let group: NostrMlsGroup | undefined = $state(undefined);
let counterpartyPubkey: string | undefined = $state(undefined);
let enrichedCounterparty: EnrichedContact | undefined = $state(undefined);
let groupName = $state("");
let transcript: NEvent[] = $state([]);

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
    invoke("get_group", { groupId: page.params.id }).then((groupResponse) => {
        group = groupResponse as NostrMlsGroup;
        transcript = group.transcript.sort((a, b) => a.created_at - b.created_at);
        counterpartyPubkey =
            group.group_type === NostrMlsGroupType.DirectMessage
                ? group.admin_pubkeys.filter((pubkey) => pubkey !== $accounts.activeAccount)[0]
                : undefined;
        if (counterpartyPubkey) {
            invoke("query_enriched_contact", {
                pubkey: counterpartyPubkey,
                updateAccount: false,
            }).then((value) => {
                enrichedCounterparty = value as EnrichedContact;
            });
        }
    });
}

async function loadMessages() {
    await invoke("fetch_mls_messages");
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
            ({ payload: [updatedGroup, message] }) => {
                console.log("mls_message_processed event received", message.content);
                if (!transcript.some((m) => m.id === message.id)) {
                    console.log("pushing message to transcript");
                    transcript = [...transcript, message].sort(
                        (a, b) => a.created_at - b.created_at
                    );
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
                loadMessages();
            }
        );
    }

    await loadGroup();
    await loadMessages();
});

function handleNewMessage(message: NEvent, replaceTemp: boolean) {
    if (replaceTemp) {
        transcript = transcript.filter((event) => event.id !== "temp");
    }
    transcript = [...transcript, message].sort((a, b) => a.created_at - b.created_at);
    scrollToBottom();
}

// const fakeMessages: NEvent[] = [
//     {
//         id: "1",
//         content: "Hello, how are you?",
//         created_at: Math.floor(Date.now() / 1000) - 20000,
//         pubkey: "ee73f3642ebc2cfd10e5f67b285e06af5672416b4b916e9be38020ccdf5e1a84",
//         kind: 9,
//         tags: [],
//     },
//     {
//         id: "2",
//         content: "I'm fine, thank you!",
//         created_at: Math.floor(Date.now() / 1000) - 18000,
//         pubkey: "7a6ac2abc092d404a6a8ca93e97a58dc5082dfb8a744c984cd40c944fb6d6574",
//         kind: 9,
//         tags: [],
//     },
//     {
//         id: "3",
//         content:
//             "Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum.?",
//         created_at: Math.floor(Date.now() / 1000) - 17800,
//         pubkey: "ee73f3642ebc2cfd10e5f67b285e06af5672416b4b916e9be38020ccdf5e1a84",
//         kind: 9,
//         tags: [],
//     },
//     {
//         id: "4",
//         content: "Whoa dude.",
//         created_at: Math.floor(Date.now() / 1000) - 16000,
//         pubkey: "7a6ac2abc092d404a6a8ca93e97a58dc5082dfb8a744c984cd40c944fb6d6574",
//         kind: 9,
//         tags: [],
//     },
//     {
//         id: "5",
//         content: "Hello, how are you?",
//         created_at: Math.floor(Date.now() / 1000) - 15500,
//         pubkey: "ee73f3642ebc2cfd10e5f67b285e06af5672416b4b916e9be38020ccdf5e1a84",
//         kind: 9,
//         tags: [],
//     },
//     {
//         id: "6",
//         content:
//             "What is Lorem Ipsum? Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum.",
//         created_at: Math.floor(Date.now() / 1000) - 15000,
//         pubkey: "7a6ac2abc092d404a6a8ca93e97a58dc5082dfb8a744c984cd40c944fb6d6574",
//         kind: 9,
//         tags: [],
//     },
//     {
//         id: "7",
//         content: "Hello, how are you?",
//         created_at: Math.floor(Date.now() / 1000) - 9000,
//         pubkey: "ee73f3642ebc2cfd10e5f67b285e06af5672416b4b916e9be38020ccdf5e1a84",
//         kind: 9,
//         tags: [],
//     },
//     {
//         id: "8",
//         content:
//             "What is Lorem Ipsum? Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum.",
//         created_at: Math.floor(Date.now() / 1000) - 8000,
//         pubkey: "7a6ac2abc092d404a6a8ca93e97a58dc5082dfb8a744c984cd40c944fb6d6574",
//         kind: 9,
//         tags: [],
//     },
//     {
//         id: "9",
//         content: "Are you coming to the meeting later?",
//         created_at: Math.floor(Date.now() / 1000) - 7000,
//         pubkey: "ee73f3642ebc2cfd10e5f67b285e06af5672416b4b916e9be38020ccdf5e1a84",
//         kind: 9,
//         tags: [],
//     },
//     {
//         id: "10",
//         content: "Yes, I'll be there in 10 minutes.",
//         created_at: Math.floor(Date.now() / 1000) - 6000,
//         pubkey: "7a6ac2abc092d404a6a8ca93e97a58dc5082dfb8a744c984cd40c944fb6d6574",
//         kind: 9,
//         tags: [],
//     },
//     {
//         id: "11",
//         content: "Great! Looking forward to it.",
//         created_at: Math.floor(Date.now() / 1000) - 5000,
//         pubkey: "ee73f3642ebc2cfd10e5f67b285e06af5672416b4b916e9be38020ccdf5e1a84",
//         kind: 9,
//         tags: [],
//     },
//     {
//         id: "12",
//         content: "Did you finish the report?",
//         created_at: Math.floor(Date.now() / 1000) - 4000,
//         pubkey: "7a6ac2abc092d404a6a8ca93e97a58dc5082dfb8a744c984cd40c944fb6d6574",
//         kind: 9,
//         tags: [],
//     },
//     {
//         id: "13",
//         content: "Yes, I sent it to your email.",
//         created_at: Math.floor(Date.now() / 1000) - 3000,
//         pubkey: "ee73f3642ebc2cfd10e5f67b285e06af5672416b4b916e9be38020ccdf5e1a84",
//         kind: 9,
//         tags: [],
//     },
//     {
//         id: "14",
//         content: "Thanks! I'll check it out.",
//         created_at: Math.floor(Date.now() / 1000) - 2000,
//         pubkey: "7a6ac2abc092d404a6a8ca93e97a58dc5082dfb8a744c984cd40c944fb6d6574",
//         kind: 9,
//         tags: [],
//     },
// ];
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
            {#each transcript as message (message.id)}
                <div class={`flex ${message.pubkey === $accounts.activeAccount ? "justify-end" : "justify-start"}`}>
                    <div
                        class={`max-w-[70%] rounded-lg ${message.pubkey === $accounts.activeAccount ? "bg-chat-bg-me text-gray-50 rounded-br" : "bg-chat-bg-other text-gray-50 rounded-bl"} p-3`}
                    >
                        <div class="flex flex-col lg:flex-row gap-2 lg:gap-6 lg:items-end">
                            <span class="break-words">
                                {#if message.content.length > 0}
                                    {message.content}
                                {:else}
                                    <span class="italic opacity-60">No message content</span>
                                {/if}
                            </span>
                            <div class={`flex flex-row gap-2 items-center justify-end ${message.pubkey === $accounts.activeAccount ? "text-gray-300" : "text-gray-400"}`}>
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
