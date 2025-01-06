<script lang="ts">
import { page } from "$app/state";
import Avatar from "$lib/components/Avatar.svelte";
import HeaderToolbar from "$lib/components/HeaderToolbar.svelte";
import MessageBar from "$lib/components/MessageBar.svelte";
import Name from "$lib/components/Name.svelte";
import { accounts } from "$lib/stores/accounts";
import { type EnrichedContact, type NEvent } from "$lib/types/nostr";
import { formatMessageTime } from "$lib/utils/time";
import { invoke } from "@tauri-apps/api/core";
import { type UnlistenFn, listen } from "@tauri-apps/api/event";
import { CaretLeft, CheckCircle, CircleDashed } from "phosphor-svelte";
import { onMount } from "svelte";

let unlistenMessageReceived: UnlistenFn;
let unlistenMessageProcessed: UnlistenFn;

let counterpartyPubkey: string = $derived(page.params.pubkey);
let enrichedCounterparty: EnrichedContact | undefined = $state(undefined);

async function loadGroup() {}

async function loadMessages() {
    if (counterpartyPubkey && !enrichedCounterparty) {
        invoke("query_enriched_contact", {
            pubkey: counterpartyPubkey,
            updateAccount: false,
        })
            .then((value) => {
                enrichedCounterparty = value as EnrichedContact;
            })
            .catch((error) => {
                console.error("Error querying enriched contact", error);
            });
    }
    // await invoke("fetch_nip17_messages_for_pubkey", { pubkey: counterpartyPubkey });
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
    if (!unlistenMessageProcessed) {
        // unlistenMessageProcessed = await listen<[NostrMlsGroup, NEvent]>(
        //     "mls_message_processed",
        //     ({ payload: [updatedGroup, message] }) => {
        //         console.log("mls_message_processed event received", message.content);
        //         if (!transcript.some((m) => m.id === message.id)) {
        //             console.log("pushing message to transcript");
        //             transcript = [...transcript, message].sort(
        //                 (a, b) => a.created_at - b.created_at
        //             );
        //         }
        //         scrollToBottom();
        //     }
        // );
    }

    if (!unlistenMessageReceived) {
        // unlistenMessageReceived = await listen<NEvent>(
        //     "mls_message_received",
        //     ({ payload: _message }) => {
        //         console.log("mls_message_received event received");
        //         loadMessages();
        //     }
        // );
    }

    await loadMessages();
});

function handleNewMessage(message: NEvent, replaceTemp: boolean) {
    // if (replaceTemp) {
    //     transcript = transcript.filter((event) => event.id !== "temp");
    // }
    // transcript = [...transcript, message].sort((a, b) => a.created_at - b.created_at);
    // scrollToBottom();
}
</script>

{#if counterpartyPubkey}
    <HeaderToolbar alwaysShowCenter={true}>
        {#snippet center()}
            <a href={`/chats/${counterpartyPubkey}/info`} class="flex flex-row items-center gap-2">
                <Avatar pubkey={counterpartyPubkey} picture={enrichedCounterparty?.metadata.picture} pxSize={30} />
                <Name pubkey={counterpartyPubkey} metadata={enrichedCounterparty?.metadata} />
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
            <!-- {#each messages as message (message.id)}
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
            {/each} -->
        </div>
        <!-- <MessageBar {group} {handleNewMessage} /> -->
    </main>
{/if}
