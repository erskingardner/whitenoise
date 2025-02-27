<script lang="ts">
import { goto } from "$app/navigation";
import Header from "$lib/components/Header.svelte";
import HeaderToolbar from "$lib/components/HeaderToolbar.svelte";
import { colorForRelayStatus, relays } from "$lib/stores/accounts";
import { activeAccount } from "$lib/stores/accounts";
import type { EnrichedContact } from "$lib/types/nostr";
import { invoke } from "@tauri-apps/api/core";
import { CaretLeft, HardDrives, Plus, Trash } from "phosphor-svelte";
import { onMount } from "svelte";

function goBack() {
    goto("/settings");
}

let showAddRelay = $state(false);
let newRelayUrl = $state("");
// State for relay lists
let inboxRelays = $state<string[]>([]);
let keyPackageRelays = $state<string[]>([]);
let normalRelays = $state<string[]>([]);
let hasInboxRelays = $state(false);
let hasKeyPackageRelays = $state(false);
let hasNormalRelays = $state(false);
let isLoading = $state(true);

function addRelay() {
    // TODO: Implement relay addition
    showAddRelay = false;
    newRelayUrl = "";
}

function removeRelay(url: string) {
    // TODO: Implement relay removal
}

// Function to load relay lists
async function loadRelayLists() {
    isLoading = true;
    try {
        const account = $activeAccount;
        if (account) {
            let contact: EnrichedContact = await invoke("fetch_enriched_contact", {
                pubkey: account.pubkey,
                updateAccount: false,
            });

            console.log("contact", contact);

            inboxRelays = contact.inbox_relays;
            keyPackageRelays = contact.key_package_relays;
            normalRelays = contact.nostr_relays;

            hasInboxRelays = inboxRelays.length > 0;
            hasKeyPackageRelays = keyPackageRelays.length > 0;
            hasNormalRelays = normalRelays.length > 0;
        }
    } catch (error) {
        console.error("Failed to load relay lists:", error);
    } finally {
        isLoading = false;
    }
}

onMount(() => {
    loadRelayLists();
});
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
        </ul>
    </div>

    <h2 class="section-title">Your Relay List</h2>
    <div class="section">
        {#if isLoading}
            <div class="p-4 text-center text-gray-600 dark:text-gray-400">
                Loading...
            </div>
        {:else if hasNormalRelays}
            <ul class="section-list divide-y divide-gray-700">
                {#each normalRelays as url}
                    <li class="section-list-item pt-2">
                        <div class="flex flex-col w-full gap-1">
                            <div class="flex items-center justify-between w-full">
                                <span class="text-sm font-medium flex items-center gap-2">
                                    <HardDrives size={20} class="text-blue-500" />
                                    {url}
                                </span>
                            </div>
                        </div>
                    </li>
                {/each}
            </ul>
        {:else}
            <div class="p-4 text-sm text-gray-600 dark:text-gray-400">
                <p>You don't have any normal relays configured.</p>
            </div>
        {/if}
    </div>

    <h2 class="section-title">Your Inbox Relay List</h2>
    <div class="section">
        {#if isLoading}
            <div class="p-4 text-center text-gray-600 dark:text-gray-400">
                Loading...
            </div>
        {:else if hasInboxRelays}
            <ul class="section-list divide-y divide-gray-700">
                {#each inboxRelays as url}
                    <li class="section-list-item pt-2">
                        <div class="flex flex-col w-full gap-1">
                            <div class="flex items-center justify-between w-full">
                                <span class="text-sm font-medium flex items-center gap-2">
                                    <HardDrives size={20} class="text-blue-500" />
                                    {url}
                                </span>
                            </div>
                        </div>
                    </li>
                {/each}
            </ul>
        {:else}
            <div class="p-4 text-sm text-gray-600 dark:text-gray-400">
                <p>You don't have any inbox relays configured.</p>
            </div>
        {/if}
    </div>

    <h2 class="section-title">Your Key Package Relay List</h2>
    <div class="section">
        {#if isLoading}
            <div class="p-4 text-center text-gray-600 dark:text-gray-400">
                Loading...
            </div>
        {:else if hasKeyPackageRelays}
            <ul class="section-list divide-y divide-gray-700">
                {#each keyPackageRelays as url}
                    <li class="section-list-item pt-2">
                        <div class="flex flex-col w-full gap-1">
                            <div class="flex items-center justify-between w-full">
                                <span class="text-sm font-medium flex items-center gap-2">
                                    <HardDrives size={20} class="text-blue-500" />
                                    {url}
                                </span>
                            </div>
                        </div>
                    </li>
                {/each}
            </ul>
        {:else}
            <div class="p-4 text-sm text-gray-600 dark:text-gray-400">
                <p>You don't have any key package relays configured.</p>
            </div>
        {/if}
    </div>

    <h2 class="section-title">About Relays</h2>
    <div class="section">
        <div class="p-4 text-sm text-gray-600 dark:text-gray-400 space-y-2">
            <p>Relays are servers that help you connect with other users on the Nostr network. They store and forward messages between users.</p>
            <p>Having multiple relays improves your connectivity and helps ensure your messages reach their intended recipients.</p>
        </div>
    </div>

    <h2 class="section-title">About Relay Lists</h2>
    <div class="section">
        <div class="p-4 text-sm text-gray-600 dark:text-gray-400 space-y-2">
            <p>Relay lists are a way to manage your relays and broadcast to apps and other users which relays you use for various purposes.</p>
            <p><strong>Normal Relays (kind: 10002):</strong> These are your standard relays where your public posts and interactions are published and read from.</p>
            <p><strong>Inbox Relays (kind: 10050):</strong> These are relays where you expect to receive private direct messages.</p>
            <p><strong>Key Package Relays (kind: 10051):</strong> These are relays where you publish key packages that other users will use to add you to secure chat groups.</p>
        </div>

    </div>
</main>
