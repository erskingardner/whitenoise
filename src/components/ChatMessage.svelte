<script lang="ts">
    import type { NEvent } from "../types/nostr";
    import { currentIdentity } from "../stores/identities";
    import Avatar from "./Avatar.svelte";
    import { formatMessageTime } from "../utils/time";

    interface Props {
        event: NEvent;
    }

    let { event }: Props = $props();
</script>

<div class="px-4 flex flex-row gap-4 items-end {$currentIdentity === event.pubkey ? 'flex-row-reverse' : ''}">
    <Avatar pubkey={event.pubkey} />
    <div
        class="rounded-lg {$currentIdentity === event.pubkey ? 'bg-gray-700' : 'bg-blue-700'} {$currentIdentity === event.pubkey
            ? 'ml-auto'
            : ''} px-4 py-2 max-w-3/4 flex flex-col gap-2 break-all whitespace-break-spaces"
    >
        {event.content}
        <div class="text-xs text-gray-400 self-end">{formatMessageTime(event.created_at)}</div>
    </div>
</div>
