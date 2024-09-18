<script lang="ts">
    import type { NMetadata } from "../types/nostr";
    import { npubFromPubkey } from "../utils/nostr";

    interface Props {
        pubkey: string;
        metadata?: NMetadata;
    }

    let { pubkey, metadata }: Props = $props();

    let name = $derived(metadata?.display_name || metadata?.name || npubFromPubkey(pubkey));
</script>

<div class="text-lg font-semibold truncate shrink">
    {name}
</div>
