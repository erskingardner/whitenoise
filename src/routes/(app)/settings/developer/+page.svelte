<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { identities, currentIdentity } from "../../../../stores/identities";
    import { Skull } from "phosphor-svelte";

    async function nukeAll() {
        await invoke("delete_app_data");
        $identities = [];
        $currentIdentity = "";
    }
</script>

<h1 class="text-xl font-semibold mb-6">Developer Tools</h1>

<div class="flex flex-col gap-6">
    <div class="flex flex-col gap-4">
        <div
            id="nostr-identity-panel"
            class="flex flex-col gap-2 ring-2 ring-gray-700 rounded-lg p-4 text-sm"
        >
            {#each $identities as identity}
                {#if $currentIdentity === identity.pubkey}
                    <code class="text-green-200">
                        {identity.pubkey}
                        <span class="text-green-500">*active</span>
                    </code>
                {:else}
                    <code>
                        {identity.pubkey}
                    </code>
                {/if}
            {/each}
        </div>
    </div>
    <button
        onclick={nukeAll}
        class="flex flex-row gap-4 items-center bg-gray-800 rounded-lg p-4 hover:ring-4 ring-gray-700 ring-offset-4 ring-offset-gray-900 w-full"
    >
        <Skull class="h-10 w-10" weight="thin" />
        Delete All App Data
    </button>
</div>
