<script lang="ts">
import type { EnrichedContact, NMetadata } from "$lib/types/nostr";
import { npubFromPubkey } from "$lib/utils/nostr";
import { invoke } from "@tauri-apps/api/core";

interface Props {
    pubkey: string;
    metadata?: NMetadata;
    extraClasses?: string;
    unstyled?: boolean;
}

let { pubkey, metadata, extraClasses, unstyled = false }: Props = $props();

let user: EnrichedContact | undefined = $state(undefined);
let userMetadata: NMetadata | undefined = $derived.by(
    () => metadata ?? user?.metadata ?? undefined
);
let name = $derived(userMetadata?.display_name || userMetadata?.name || npubFromPubkey(pubkey));
let isNpub = $derived(!userMetadata?.display_name && !userMetadata?.name);
let userFetched: boolean = $state(false);

// Watch for prop changes
$effect(() => {
    const currentPubkey = pubkey;
    const currentMetadata = metadata;

    // Only reset when props actually change
    user = undefined;
    userFetched = false;
});

// Handle fetching
$effect(() => {
    if (!userMetadata && !userFetched) {
        invoke("query_enriched_contact", {
            pubkey,
            updateAccount: false,
        })
            .then((userResp) => {
                user = userResp as EnrichedContact;
                userFetched = true;
            })
            .catch((e) => console.error(e));
    }
});
</script>

{#if unstyled}
    <span>{name}</span>
{:else}
    <span class="text-lg font-semibold truncate shrink {isNpub ? 'font-mono' : ''} {extraClasses}">
        {name}
    </span>
{/if}
