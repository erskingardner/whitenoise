<script lang="ts">
    import type { NEvent } from "../types/nostr";
    import Avatar from "./Avatar.svelte";
    import Name from "./Name.svelte";
    import { formatMessageTime } from "../utils/time";
    import ndk from "../stores/ndk";
    import { type NDKUser } from "@nostr-dev-kit/ndk";

    interface Props {
        pubkey: string;
        active: boolean;
        lastMessageAt?: number;
    }

    let { pubkey, active = false, lastMessageAt }: Props = $props();

    const user: NDKUser = $derived($ndk.getUser({pubkey}));
</script>

<div
    class="chat-item flex flex-row gap-4 items-center justify-start px-4 py-2  border-b border-gray-700 hover:bg-gray-800 {active
        ? 'bg-gray-800'
        : ''}"
>
    <Avatar pubkey={pubkey} pxSize={36} />
    <div class="grow flex flex-col justify-start items-start">
        <div class="font-medium">
            <Name pubkey={pubkey} />
        </div>
        <div class="font-light font-mono line-clamp-2">
            {user.npub.slice(0, 18)}...
        </div>
    </div>
    <div class="metadata flex flex-col gap-1 items-center justify-center ml-auto">
        {#if lastMessageAt}
            <div class="timestamp text-xs">{formatMessageTime(lastMessageAt)}</div>
        {/if}
    </div>
</div>