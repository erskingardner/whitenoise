<script lang="ts">
    import type { NMetadata } from "$lib/types/nostr";
    import { npubFromPubkey } from "$lib/utils/nostr";

    interface Props {
        pubkey: string;
        metadata?: NMetadata;
        extraClasses?: string;
        unstyled?: boolean;
    }

    let { pubkey, metadata, extraClasses, unstyled = false }: Props = $props();

    let name = $derived(metadata?.display_name || metadata?.name || npubFromPubkey(pubkey));
    let isNpub = $derived(!metadata?.display_name && !metadata?.name);

    let importantExtraClasses: string[] = $derived.by(() => {
        const classes: string[] = [];
        extraClasses?.split(" ").forEach((cls) => {
            classes.push("!" + cls);
        });
        return classes;
    });

    $inspect(importantExtraClasses);
</script>

{#if unstyled}
    <span>{name}</span>
{:else}
    <span
        class="text-lg font-semibold truncate shrink {isNpub
            ? 'font-mono'
            : ''} {importantExtraClasses.join(' ')}"
    >
        {name}
    </span>
{/if}
