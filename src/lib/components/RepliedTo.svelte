<script lang="ts">
import { activeAccount } from "$lib/stores/accounts";
import type { NEvent } from "$lib/types/nostr";
import { invoke } from "@tauri-apps/api/core";
import { onMount } from "svelte";
import Name from "./Name.svelte";

let { messageId }: { messageId?: string } = $props();

let message: NEvent | undefined = $state(undefined);

onMount(() => {
    if (messageId) {
        invoke("query_message", {
            messageId,
        }).then((messageResponse) => {
            message = messageResponse as NEvent;
        });
    }
});
</script>

{#if message}
    <div class="flex flex-col gap-1 bg-blue-900/80 rounded-r-lg p-2 border-l-4 border-l-white/50 pl-4 mb-2 text-sm">
            {#if message.pubkey === $activeAccount?.pubkey}
                <span class="font-bold">You</span>
            {:else}
                <span class="font-bold truncate">
                    <Name pubkey={message.pubkey} unstyled={true} />
                </span>
            {/if}
        <span class="break-words-smart">{message.content}</span>
    </div>
{:else}
    <div class="flex flex-col gap-1 bg-blue-900/80 rounded-lg p-2 border-l-4 border-l-white/50 pl-4 mb-2 text-sm">
        <span class="font-bold">
          <span>Loading...</span>
        </span>
    </div>
{/if}
