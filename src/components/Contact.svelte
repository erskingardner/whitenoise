<script lang="ts">
    import type { NMetadata } from "../types/nostr";
    import Avatar from "./Avatar.svelte";
    import Name from "./Name.svelte";
    import { formatMessageTime } from "../utils/time";
    import { npubFromPubkey } from "../utils/nostr";
    import { LockKeyOpen } from "phosphor-svelte";

    interface Props {
        pubkey: string;
        active: boolean;
        metadata: NMetadata;
        lastMessageAt?: number;
        isLegacy?: boolean;
    }

    let { pubkey, active = false, metadata, lastMessageAt, isLegacy = false }: Props = $props();
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
    <div class="metadata flex flex-col gap-0.5 items-center justify-center ml-auto text-nowrap">
        {#if isLegacy}
            <LockKeyOpen size="1.5rem" weight="light" class="text-red-500" />
        {/if}
        {#if lastMessageAt}
            <div class="timestamp text-sm font-mono">{formatMessageTime(lastMessageAt)}</div>
        {/if}
    </div>
</div>
