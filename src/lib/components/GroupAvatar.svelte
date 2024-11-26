<script lang="ts">
import { NostrMlsGroupType } from "$lib/types/nostr";
import type { EnrichedContact } from "$lib/types/nostr";
import Avatar from "./Avatar.svelte";

let {
    groupType = $bindable(),
    groupName = $bindable(),
    counterpartyPubkey = $bindable(),
    enrichedCounterparty = $bindable(),
    pxSize,
}: {
    groupType: NostrMlsGroupType;
    groupName: string;
    counterpartyPubkey: string | undefined;
    enrichedCounterparty: EnrichedContact | undefined;
    pxSize: number;
} = $props();

let groupAvatarColor: string = $derived(
    groupName
        .split("")
        .reduce((acc: number, char: string) => acc + char.charCodeAt(0), 0)
        .toString(16)
        .padStart(6, "5")
        .slice(0, 6)
);
</script>

{#if groupType === NostrMlsGroupType.DirectMessage && counterpartyPubkey && enrichedCounterparty}
    <Avatar picture={enrichedCounterparty?.metadata.picture} pubkey={counterpartyPubkey} {pxSize} />
{:else}
    <div
        class="flex flex-col items-center justify-center rounded-full bg-gray-900"
        style="width: {pxSize}px; height: {pxSize}px; min-width: {pxSize}px; min-height: {pxSize}px;"
    >
        <div
            class="w-full h-full rounded-full font-semibold text-xl font-mono shrink-0 flex flex-col justify-center text-center"
            style="background-color: #{groupAvatarColor};"
        >
            {groupName.slice(0, 2)}
        </div>
    </div>
{/if}
