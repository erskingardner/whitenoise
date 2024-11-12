<script lang="ts">
    import HeaderToolbar from "$lib/components/HeaderToolbar.svelte";
    import { type NostrMlsGroup, type EnrichedContact, NostrMlsGroupType, type NEvent } from "$lib/types/nostr";
    import { invoke } from "@tauri-apps/api/core";
    import { nameFromMetadata } from "$lib/utils/nostr";
    import { accounts } from "$lib/stores/accounts";
    import { CaretLeft } from "phosphor-svelte";
    import MessageBar from "$lib/components/MessageBar.svelte";
    import { hexMlsGroupId } from "$lib/utils/group";
    import GroupAvatar from "$lib/components/GroupAvatar.svelte";
    import { formatMessageTime } from "$lib/utils/time";

    let { data } = $props();

    let group: NostrMlsGroup = data.group as NostrMlsGroup;

    let counterpartyPubkey =
        group.group_type === NostrMlsGroupType.DirectMessage
            ? group.admin_pubkeys.filter((pubkey) => pubkey !== $accounts.activeAccount)[0]
            : undefined;

    let enrichedCounterparty: EnrichedContact | undefined = $state(undefined);
    let groupName = $state("");

    // Test data - you can delete this later
    // const testTranscript = [
    //     {
    //         id: "1234567890",
    //         pubkey: $accounts.activeAccount,
    //         kind: 1,
    //         content: "Hey there! How are you?",
    //         created_at: Math.floor(Date.now() / 1000) - 45000, // 1 hour ago
    //     },
    //     {
    //         id: "1234567891",
    //         pubkey: counterpartyPubkey,
    //         kind: 1,
    //         content: "I'm doing great! Thanks for asking. How about you?",
    //         created_at: Math.floor(Date.now() / 1000) - 3500, // 58 minutes ago
    //     },
    //     {
    //         id: "1234567892",
    //         pubkey: $accounts.activeAccount,
    //         kind: 1,
    //         content: "Pretty good! Working on this new chat interface. What do you think about it?",
    //         created_at: Math.floor(Date.now() / 1000) - 3400, // 56 minutes ago
    //     },
    //     {
    //         id: "1234567893",
    //         pubkey: counterpartyPubkey,
    //         kind: 1,
    //         content:
    //             "It looks really nice! I especially like how the messages are aligned differently for each person.",
    //         created_at: Math.floor(Date.now() / 1000) - 3300, // 55 minutes ago
    //     },
    //     {
    //         id: "1234567894",
    //         pubkey: $accounts.activeAccount,
    //         kind: 1,
    //         content:
    //             "This is a longer message to test how the interface handles multiple lines of text. It should wrap nicely within the message bubble and not exceed 70% of the screen width.",
    //         created_at: Math.floor(Date.now() / 1000) - 3200, // 53 minutes ago
    //     },
    //     {
    //         id: "1234567895",
    //         pubkey: $accounts.activeAccount,
    //         kind: 1,
    //         content: "",
    //         created_at: Math.floor(Date.now() / 1000) - 3000, // 50 minutes ago
    //     },
    //     {
    //         id: "1234567896",
    //         pubkey: counterpartyPubkey,
    //         kind: 1,
    //         content:
    //             "By the way, have you seen the new updates to the protocol? They added some interesting features for group messaging.",
    //         created_at: Math.floor(Date.now() / 1000) - 2800, // ~47 minutes ago
    //     },
    //     {
    //         id: "1234567897",
    //         pubkey: $accounts.activeAccount,
    //         kind: 1,
    //         content:
    //             "Yes! I'm particularly excited about the enhanced encryption methods. We should implement those soon.",
    //         created_at: Math.floor(Date.now() / 1000) - 2700, // ~45 minutes ago
    //     },
    //     {
    //         id: "1234567898",
    //         pubkey: counterpartyPubkey,
    //         kind: 1,
    //         content: "Definitely! 🔐 Security first! When do you think we can start working on that?",
    //         created_at: Math.floor(Date.now() / 1000) - 2600, // ~43 minutes ago
    //     },
    //     {
    //         id: "1234567899",
    //         pubkey: $accounts.activeAccount,
    //         kind: 1,
    //         content:
    //             "Maybe we can start planning next week? I'll set up a meeting to discuss the implementation details.",
    //         created_at: Math.floor(Date.now() / 1000) - 2500, // ~42 minutes ago
    //     },
    // ];

    // // Temporarily override the group transcript with test data
    // group.transcript = testTranscript as NEvent[];

    $effect(() => {
        if (counterpartyPubkey) {
            invoke("fetch_enriched_contact", {
                pubkey: counterpartyPubkey,
                updateAccount: false,
            }).then((value) => {
                enrichedCounterparty = value as EnrichedContact;
            });
        }
    });

    $effect(() => {
        if (group.group_type === NostrMlsGroupType.DirectMessage && counterpartyPubkey && enrichedCounterparty) {
            groupName = nameFromMetadata(enrichedCounterparty.metadata, counterpartyPubkey);
        } else {
            groupName = group.name;
        }
    });
</script>

<HeaderToolbar alwaysShowCenter={true}>
    {#snippet center()}
        <a href={`/chats/${hexMlsGroupId(group.mls_group_id)}/info`} class="flex flex-row items-center gap-2">
            <GroupAvatar
                groupType={group.group_type}
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

<main class="flex flex-col relative min-h-screen">
    <div class="flex-1 overflow-y-auto px-8 flex flex-col gap-6 pt-10 pb-40">
        {#each group.transcript as message (message.id)}
            <div class={`flex ${message.pubkey === $accounts.activeAccount ? "justify-end" : "justify-start"}`}>
                <div
                    class={`max-w-[70%] rounded-lg ${message.pubkey === $accounts.activeAccount ? "bg-chat-bg-me text-gray-50 rounded-br-none" : "bg-chat-bg-other text-gray-50 rounded-bl-none"} p-3`}
                >
                    <div class="flex flex-col">
                        <span class="break-words">
                            {#if message.content.length > 0}
                                {message.content}
                            {:else}
                                <span class="italic opacity-60">No message content</span>
                            {/if}
                        </span>
                        <span class="text-sm opacity-60 self-end mt-2">
                            {formatMessageTime(message.created_at)}
                        </span>
                    </div>
                </div>
            </div>
        {/each}
    </div>
    <MessageBar {group} />
</main>
