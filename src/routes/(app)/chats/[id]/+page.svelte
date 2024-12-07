<script lang="ts">
import { page } from "$app/stores";
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
import { CaretLeft } from "phosphor-svelte";
import { onMount } from "svelte";

let unlistenMlsMessageReceived: UnlistenFn;

let group: NostrMlsGroup | undefined = $state(undefined);
let counterpartyPubkey: string | undefined = $state(undefined);
let enrichedCounterparty: EnrichedContact | undefined = $state(undefined);
let groupName = $state("");
let transcript: NEvent[] = $state([]);

$inspect(transcript);

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
    invoke("get_group", { groupId: $page.params.id }).then((groupResponse) => {
        group = groupResponse as NostrMlsGroup;
        transcript = group.transcript;
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
    if (!unlistenMlsMessageReceived) {
        unlistenMlsMessageReceived = await listen<[NostrMlsGroup, NEvent]>(
            "mls_message_received",
            ({ payload: [updatedGroup, message] }) => {
                // console.log("updatedGroups:", updatedGroup);
                console.log("message received", message.content);
                if (!transcript.some((m) => m.id === message.id)) {
                    console.log("pushing message to transcript");
                    transcript = [...transcript, message];
                }
                scrollToBottom();
            }
        );
    }

    await loadGroup();
    await loadMessages();
});

function handleNewMessage(message: NEvent) {
    transcript = [...transcript, message];
    scrollToBottom();
}
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

    <main id="mainContainer" class="flex flex-col relative min-h-screen">
        <div
            id="messagesContainer"
            class="flex-1 px-8 flex flex-col gap-2 pt-10 pb-40 overflow-y-auto opacity-0 transition-opacity ease-in-out duration-50"
        >
            {#each transcript as message (message.id)}
                <div class={`flex ${message.pubkey === $accounts.activeAccount ? "justify-end" : "justify-start"}`}>
                    <div
                        class={`max-w-[70%] rounded-lg ${message.pubkey === $accounts.activeAccount ? "bg-chat-bg-me text-gray-50 rounded-br" : "bg-chat-bg-other text-gray-50 rounded-bl"} p-3`}
                    >
                        <div class="flex flex-row gap-4">
                            <span class="break-words">
                                {#if message.content.length > 0}
                                    {message.content}
                                {:else}
                                    <span class="italic opacity-60">No message content</span>
                                {/if}
                            </span>
                            <span class="text-sm opacity-60 self-end mt-2 whitespace-nowrap">
                                {formatMessageTime(message.created_at)}
                            </span>
                        </div>
                    </div>
                </div>
            {/each}
        </div>
        <MessageBar {group} {handleNewMessage} />
    </main>
{/if}
