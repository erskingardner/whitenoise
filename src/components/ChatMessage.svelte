<script lang="ts">
    import type { NEvent, NMetadata } from "../types/nostr";
    import { currentIdentity } from "../stores/accounts";
    import Avatar from "./Avatar.svelte";
    import { formatMessageTime } from "../utils/time";
    import { LockKeyOpen } from "phosphor-svelte";
    import { Tooltip } from "flowbite-svelte";

    interface Props {
        event: NEvent;
        metadata?: NMetadata;
    }

    let { event, metadata }: Props = $props();
</script>

<div
    class="px-4 flex flex-row gap-4 items-end {$currentIdentity === event.pubkey
        ? 'flex-row-reverse'
        : ''}"
>
    <Avatar pubkey={event.pubkey} picture={metadata?.picture} />
    <div
        class="rounded-lg {$currentIdentity === event.pubkey
            ? 'bg-gray-700'
            : 'bg-blue-700'} {$currentIdentity === event.pubkey
            ? 'ml-auto'
            : ''} px-4 py-2 max-w-3/4 flex flex-col gap-2 break-all whitespace-break-spaces"
    >
        {event.content}
        <div class="text-xs text-gray-400 self-end flex flex-row gap-1 items-center">
            {formatMessageTime(event.created_at)}
            {#if event.kind === 4}
                <LockKeyOpen size="1rem" weight="regular" class="text-red-500" />
            {/if}
            <Tooltip defaultClass="tooltip">
                This is a NIP-04 encrypted message.<br />
                <span class="font-medium italic">All metadata is publicly visible.</span>
            </Tooltip>
        </div>
    </div>
</div>
