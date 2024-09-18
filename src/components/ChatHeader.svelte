<script lang="ts">
    import { MagnifyingGlass, DotsThreeCircle, User } from "phosphor-svelte";
    import Avatar from "./Avatar.svelte";
    import Name from "./Name.svelte";
    import { npubFromPubkey } from "../utils/nostr";
    import type { NMetadata } from "../types/nostr";

    interface Props {
        pubkey: string;
        metadata?: NMetadata;
    }

    let { pubkey, metadata }: Props = $props();
</script>

<div class="sticky top-0 p-4 border-b border-gray-700 bg-gray-900">
    <div class="flex justify-between items-center w-full grow">
        <div class="flex flex-row gap-4 items-center">
            <Avatar {pubkey} picture={metadata?.picture} pxSize={40} />
            <div class="flex flex-col gap-0">
                <Name {pubkey} {metadata} />
                <div class="font-mono text-gray-500 truncate">{npubFromPubkey(pubkey)}</div>
            </div>
        </div>
        <div class="icons flex flex-row gap-4 items-center">
            <MagnifyingGlass size="1.5rem" weight="thin" />
            <DotsThreeCircle size="1.5rem" weight="thin" />
        </div>
    </div>
</div>
