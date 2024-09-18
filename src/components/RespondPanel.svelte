<script lang="ts">
    import { PaperPlaneTilt } from "phosphor-svelte";
    import type { NEvent } from "../types/nostr";
    import { unixTimestamp } from "../utils/time";
    import { invoke } from "@tauri-apps/api/core";

    type Props = {
        pubkey: string;
    };

    let { pubkey }: Props = $props();

    let message = $state("");

    async function sendMessage() {
        const event_id = await invoke("send_message", {
            pubkey: pubkey,
            message: message,
        });

        // Clear the message input
        message = "";
    }
</script>

<div class="sticky bottom-0 flex flex-row gap-4 p-4 bg-gray-800 items-center mt-auto">
    <form onsubmit={sendMessage} class="flex flex-row gap-4 itmes-center w-full">
        <input
            type="text"
            bind:value={message}
            placeholder="Type a message&hellip;"
            class="bg-transparent ring-1 ring-gray-700 rounded-md grow p-2"
        />
        <button
            type="submit"
            class="rounded-full ring-1 ring-gray-700 p-2 hover:bg-gray-700 hover:text-white"
        >
            <PaperPlaneTilt size="1.5rem" weight="thin" />
        </button>
    </form>
</div>
