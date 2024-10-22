<script lang="ts">
    import type { NMetadata } from "../types/nostr";
    import { npubFromPubkey } from "../utils/nostr";

    interface Props {
        pubkey: string;
        metadata?: NMetadata;
        extraClasses?: string;
        unstyled?: boolean;
    }

    let { pubkey, metadata, extraClasses, unstyled = false }: Props = $props();

    let name = $derived(metadata?.display_name || metadata?.name || npubFromPubkey(pubkey));
    let isNpub = $derived(!metadata?.display_name && !metadata?.name);
</script>

{#if unstyled}
    <span>{name}</span>
{:else}
    <span class="text-lg font-semibold truncate shrink {isNpub ? 'font-mono' : ''} {extraClasses}">
        {name}
    </span>
{/if}
