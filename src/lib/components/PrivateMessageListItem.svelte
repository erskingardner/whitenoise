<script lang="ts">
import type { EnrichedContact, NEvent } from "$lib/types/nostr";
import { invoke } from "@tauri-apps/api/core";
import Avatar from "./Avatar.svelte";
import Name from "./Name.svelte";

let { pubkey, messages }: { pubkey: string; messages: NEvent[] } = $props();
let enrichedCounterparty: EnrichedContact | undefined = $state(undefined);

$effect(() => {
    if (!enrichedCounterparty) {
        invoke("query_enriched_contact", {
            pubkey,
            updateAccount: false,
        }).then((value) => {
            enrichedCounterparty = value as EnrichedContact;
        });
    }
});
</script>

<a
    href={`/legacy/${pubkey}/`}
    class="flex flex-row gap-2 items-center px-4 py-3 border-b border-gray-700 hover:bg-gray-700"
>
    <Avatar picture={enrichedCounterparty?.metadata.picture} {pubkey} pxSize={40} />
    <div class="flex flex-col gap-1">
        <span class="text-lg font-semibold">
            <Name {pubkey} metadata={enrichedCounterparty?.metadata} />
        </span>
        <!-- TODO: Add message preview -->
    </div>
</a>
