<script lang="ts">
    import type { NMetadata } from "../types/nostr";
    import Avatar from "./Avatar.svelte";
    import Name from "./Name.svelte";
    import { formatMessageTime } from "../utils/time";
    import { npubFromPubkey } from "../utils/nostr";

    interface Props {
        pubkey: string;
        active: boolean;
        metadata: NMetadata;
        lastMessageAt?: number;
    }

    let { pubkey, active = false, metadata, lastMessageAt }: Props = $props();
</script>

<div
    class="chat-item flex flex-row gap-4 items-center justify-start px-4 py-2 border-b border-gray-700 hover:bg-gray-800 {active
        ? 'bg-gray-800'
        : ''}"
>
    <Avatar picture={metadata.picture} {pubkey} pxSize={36} />
    <div class="grow flex flex-col justify-start items-start overflow-x-hidden truncate">
        <div class="font-medium truncate">
            <Name {pubkey} {metadata} />
        </div>
        <div class="font-light font-mono text-base w-full truncate">
            {npubFromPubkey(pubkey)}
        </div>
    </div>
    <div class="metadata flex flex-col gap-1 items-center justify-center ml-auto">
        {#if lastMessageAt}
            <div class="timestamp text-xs">{formatMessageTime(lastMessageAt)}</div>
        {/if}
    </div>
</div>
