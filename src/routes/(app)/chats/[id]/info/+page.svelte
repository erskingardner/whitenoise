<script lang="ts">
import { page } from "$app/stores";
import GroupAvatar from "$lib/components/GroupAvatar.svelte";
import HeaderToolbar from "$lib/components/HeaderToolbar.svelte";
import { accounts } from "$lib/stores/accounts";
import { type NostrMlsGroup, NostrMlsGroupType } from "$lib/types/nostr";
import type { EnrichedContact, NEvent } from "$lib/types/nostr";
import { nameFromMetadata } from "$lib/utils/nostr";
import { invoke } from "@tauri-apps/api/core";
import { CaretLeft, LockKey } from "phosphor-svelte";
import { onMount } from "svelte";

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

onMount(async () => {
    await loadGroup();
});
</script>

{#if group}
    <HeaderToolbar>
        {#snippet left()}
            <button onclick={() => window.history.back()} class="p-2 -mr-2">
                <CaretLeft size={30} />
            </button>
        {/snippet}
    </HeaderToolbar>
    <div class="flex flex-col items-center justify-center gap-10 p-4">
        <GroupAvatar groupType={group.group_type} {groupName} {counterpartyPubkey} {enrichedCounterparty} pxSize={80} />
        <h1 class="text-2xl font-bold">{groupName}</h1>
        <p class="text-gray-500 flex flex-row items-center gap-2">
            <LockKey size={20} />
            {group.description || "A secure chat"}
        </p>
    </div>
{/if}
