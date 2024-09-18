<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { identities, currentIdentity, fetchAccounts } from "../../../../stores/accounts";
    import { Skull, UserList } from "phosphor-svelte";
    import { onMount } from "svelte";

    let events: Events;
    type Events = {
        database_contacts?: number;
        relay_contacts?: number;
        database_chats?: number;
        relay_chats?: number;
    };

    onMount(async () => {
        events = await invoke("fetch_dev_events");
    });
    async function nukeAll() {
        await invoke("delete_app_data");
        $identities = {};
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
            {#each Object.entries($identities) as [pubkey, _metadata]}
                {#if $currentIdentity === pubkey}
                    <code class="text-green-200">
                        {pubkey}
                        <span class="text-green-500">*active</span>
                    </code>
                {:else}
                    <code>
                        {pubkey}
                    </code>
                {/if}
            {/each}
        </div>
    </div>
    <div>
        {#if events}
            <pre>{JSON.stringify(events, null, 2)}</pre>
        {/if}
    </div>
    <button
        onclick={nukeAll}
        class="flex flex-row gap-4 items-center bg-gray-800 rounded-lg p-4 hover:ring-4 ring-gray-700 ring-offset-4 ring-offset-gray-900 w-full"
    >
        <Skull class="h-10 w-10" weight="thin" />
        Delete All App Data
    </button>
    <button
        onclick={fetchAccounts}
        class="flex flex-row gap-4 items-center bg-gray-800 rounded-lg p-4 hover:ring-4 ring-gray-700 ring-offset-4 ring-offset-gray-900 w-full"
    >
        <UserList class="h-10 w-10" weight="thin" />
        Fetch accounts
    </button>
</div>
