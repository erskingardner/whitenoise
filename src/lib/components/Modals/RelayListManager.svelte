<script lang="ts">
import { activeAccount } from "$lib/stores/accounts";
import { getToastState } from "$lib/stores/toast-state.svelte";
import type { NRelayList, RelayMeta, RelayWithMeta } from "$lib/types/nostr";
import { invoke } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";
import { HardDrives, Plus, Trash } from "phosphor-svelte";
import { onMount } from "svelte";

// Props
let { kind, title, relays, closeModal } = $props<{
    kind: number;
    title: string;
    relays: NRelayList;
    closeModal: () => void;
}>();

// State
let newRelayUrl = $state("");
let readWriteMode = $state<RelayMeta>("ReadWrite");
let isSubmitting = $state(false);

// For normal relays (10002), we need to handle read/write permissions
const isNormalRelayList = $derived(kind === 10002);

// Toast for notifications
const toastState = getToastState();

// Add a new relay
async function addRelay() {
    // TODO: checks for duplicates, etc.
    isSubmitting = true;
}

// Update relay permission
async function updateRelayPermission(url: string, permission: string) {
    isSubmitting = true;
}

// Remove a relay
async function removeRelay(url: string) {
    isSubmitting = true;
}
</script>

<div class="flex flex-col gap-4">
    <div class="flex flex-col gap-2">
        <h2 class="text-xl font-bold">{title}</h2>
        <p class="text-sm text-gray-400">
            {#if kind === 10002}
                These are your standard relays where your public posts and interactions are published and read from.
            {:else if kind === 10050}
                These are relays where you expect to receive private direct messages.
            {:else if kind === 10051}
                These are relays where you publish key packages that other users will use to add you to secure chat groups.
            {/if}
        </p>
    </div>

    <!-- Current Relays -->
    <div class="mt-4">
        <h3 class="text-lg font-semibold mb-2">Current Relays</h3>
        {#if relays.length === 0}
            <div class="p-4 text-center text-gray-400">
                No relays configured. Add one below.
            </div>
        {:else}
            <ul class="divide-y divide-gray-700">
                {#each relays.relays as relay}
                    <li class="py-3 flex flex-col gap-2">
                        <div class="flex items-center justify-between">
                            <span class="text-sm font-medium flex items-center gap-2">
                                <HardDrives size={20} class="text-blue-500" />
                                {relay.url}
                            </span>
                            <button
                                class="p-2 rounded-md hover:bg-gray-700 transition-colors text-red-500 hover:text-red-400 disabled:opacity-50"
                                onclick={() => removeRelay(relay.url)}
                                disabled={isSubmitting}
                            >
                                <Trash size={18} />
                            </button>
                        </div>

                        {#if isNormalRelayList}
                            <div class="flex items-center gap-2">
                                <span class="text-xs text-gray-400">Permission:</span>
                                <select
                                    value={relay.meta}
                                    onchange={(e) => updateRelayPermission(relay.url, e.currentTarget.value)}
                                    class="text-xs bg-transparent ring-1 ring-gray-700 rounded-md px-2 py-1"
                                    disabled={isSubmitting}
                                >
                                    <option value="ReadWrite">Read & Write</option>
                                    <option value="Read">Read Only</option>
                                    <option value="Write">Write Only</option>
                                </select>
                            </div>
                        {/if}
                    </li>
                {/each}
            </ul>
        {/if}
    </div>

    <!-- Add New Relay -->
    <div class="mt-4">
        <h3 class="text-lg font-semibold mb-2">Add New Relay</h3>
        <div class="flex flex-col gap-3">
            <input
                type="text"
                bind:value={newRelayUrl}
                placeholder="wss://relay.example.com"
                class="w-full px-3 py-2 bg-transparent ring-1 ring-gray-700 rounded-md"
                disabled={isSubmitting}
            />

            <select
                bind:value={readWriteMode}
                class="w-full px-3 py-2 bg-transparent ring-1 ring-gray-700 rounded-md"
                disabled={isSubmitting}
            >
                <option value="ReadWrite">Read & Write</option>
                <option value="Read">Read Only</option>
                <option value="Write">Write Only</option>
            </select>

            <button
                class="button-primary flex items-center justify-center gap-2"
                onclick={addRelay}
                disabled={isSubmitting || !newRelayUrl.trim()}
            >
                <Plus size={18} />
                Add Relay
            </button>
        </div>
    </div>
</div>
