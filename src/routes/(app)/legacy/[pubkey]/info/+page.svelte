<script lang="ts">
import { page } from "$app/state";
import Avatar from "$lib/components/Avatar.svelte";
import HeaderToolbar from "$lib/components/HeaderToolbar.svelte";
import Name from "$lib/components/Name.svelte";
import type { EnrichedContact, NEvent } from "$lib/types/nostr";
import { invoke } from "@tauri-apps/api/core";
import { CaretLeft, LockKey } from "phosphor-svelte";

let counterpartyPubkey: string | undefined = $derived(page.params.pubkey);
let enrichedCounterparty: EnrichedContact | undefined = $state(undefined);

$effect(() => {
    if (counterpartyPubkey && !enrichedCounterparty) {
        invoke("query_enriched_contact", {
            pubkey: counterpartyPubkey,
            updateAccount: false,
        }).then((value) => {
            enrichedCounterparty = value as EnrichedContact;
        });
    }
});
</script>

{#if counterpartyPubkey}
    <HeaderToolbar>
        {#snippet left()}
            <button onclick={() => window.history.back()} class="p-2 -mr-2">
                <CaretLeft size={30} />
            </button>
        {/snippet}
    </HeaderToolbar>
    <div class="flex flex-col items-center justify-center gap-10 p-4">
        <Avatar pubkey={counterpartyPubkey} picture={enrichedCounterparty?.metadata.picture} pxSize={80} />
        <h1 class="text-2xl font-bold"><Name pubkey={counterpartyPubkey} metadata={enrichedCounterparty?.metadata} /></h1>
        <p class="text-gray-500 flex flex-row items-center gap-2">
            <LockKey size={20} />
            A NIP-17 Private Direct Message
        </p>
    </div>
{/if}
