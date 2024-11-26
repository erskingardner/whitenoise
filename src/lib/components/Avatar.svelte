<script lang="ts">
import { accounts } from "$lib/stores/accounts";

interface Props {
    picture?: string;
    pubkey: string;
    pxSize?: number;
    showRing?: boolean;
}
let { pubkey, picture, pxSize = 32, showRing = false }: Props = $props();
</script>

<div
    class="flex flex-col items-center justify-center rounded-full bg-gray-900 {$accounts.activeAccount === pubkey &&
    showRing
        ? 'ring-4 ring-blue-600 ring-offset-2 ring-offset-gray-900'
        : ''}"
    style="width: {pxSize}px; height: {pxSize}px; min-width: {pxSize}px; min-height: {pxSize}px;"
>
    {#if picture}
        <img src={picture} alt="avatar" class="shrink-0 w-full h-full rounded-full object-cover" />
    {:else}
        <div
            class="w-full h-full rounded-full font-semibold text-xl font-mono shrink-0 flex flex-col justify-center text-center"
            style="background-color: #{pubkey.slice(0, 6)};"
        >
            {pubkey.slice(0, 2)}
        </div>
    {/if}
</div>
