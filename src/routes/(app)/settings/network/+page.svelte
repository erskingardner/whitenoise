<script lang="ts">
import { goto } from "$app/navigation";
import Header from "$lib/components/Header.svelte";
import HeaderToolbar from "$lib/components/HeaderToolbar.svelte";
import { colorForRelayStatus, relays } from "$lib/stores/accounts";
import { CaretLeft, HardDrives, Plus, Trash } from "phosphor-svelte";

function goBack() {
    goto("/settings");
}

let showAddRelay = $state(false);
let newRelayUrl = $state("");

function addRelay() {
    // TODO: Implement relay addition
    showAddRelay = false;
    newRelayUrl = "";
}

function removeRelay(url: string) {
    // TODO: Implement relay removal
}
</script>

<HeaderToolbar>
    {#snippet left()}
        <button class="flex flex-row gap-0.5 items-center" onclick={goBack}>
            <CaretLeft size={24} weight="bold" />
            <span class="font-medium text-lg">Back</span>
        </button>
    {/snippet}
    {#snippet center()}
        <h1>Network Settings</h1>
    {/snippet}
</HeaderToolbar>

<Header title="Network Settings" />

<main class="px-4 flex flex-col pb-32">
    <h2 class="section-title">Connected Relays</h2>
    <div class="section">
        <ul class="section-list divide-y divide-gray-700">
            {#each Object.entries($relays) as [url, status]}
                <li class="section-list-item pt-2">
                    <div class="flex flex-col w-full gap-1">
                        <div class="flex items-center justify-between w-full">
                            <span class="text-sm font-medium flex items-center gap-2">
                                <HardDrives size={20} class={colorForRelayStatus(status)} />
                                {url}
                            </span>
                            <!-- <button
                                class="p-2 rounded-md hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-red-500 hover:text-red-600"
                                onclick={() => removeRelay(url)}
                            >
                                <Trash size={16} />
                            </button> -->
                        </div>
                        <div class="text-sm text-gray-600 dark:text-gray-400 flex items-center gap-2">
                            <span class={`inline-block w-2 h-2 rounded-full ${status === 'Connected' ? 'bg-green-500' : status === 'Connecting' ? 'bg-yellow-500' : 'bg-red-500'}`}></span>
                            <span>{status}</span>
                        </div>
                    </div>
                </li>
            {/each}

            <!-- {#if showAddRelay}
                <li class="section-list-item">
                    <div class="flex flex-col w-full gap-2">
                        <span class="text-sm font-medium">Add New Relay</span>
                        <div class="flex gap-2">
                            <input
                                type="text"
                                bind:value={newRelayUrl}
                                placeholder="wss://relay.example.com"
                                class="w-full px-4 py-2 rounded-lg border dark:border-gray-700 bg-white dark:bg-gray-800"
                            />
                            <button
                                class="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                                onclick={addRelay}
                                disabled={!newRelayUrl}
                            >
                                Add
                            </button>
                        </div>
                    </div>
                </li>
            {:else}
                <li class="section-list-item">
                    <button
                        class="row-button text-blue-500 hover:text-blue-600"
                        onclick={() => showAddRelay = true}
                    >
                        <Plus size={24} class="shrink-0" />
                        <span>Add New Relay</span>
                    </button>
                </li>
            {/if} -->
        </ul>
    </div>

    <h2 class="section-title">About Relays</h2>
    <div class="section">
        <div class="p-4 text-sm text-gray-600 dark:text-gray-400 space-y-2">
            <p>Relays are servers that help you connect with other users on the Nostr network. They store and forward messages between users.</p>
            <p>Having multiple relays improves your connectivity and helps ensure your messages reach their intended recipients.</p>
        </div>
    </div>
</main>
