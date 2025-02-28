<script lang="ts">
import { activeAccount } from "$lib/stores/accounts";
import { getToastState } from "$lib/stores/toast-state.svelte";
import type { EnrichedContact } from "$lib/types/nostr";
import { invoke } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";
import { HardDrives, Plus, Trash } from "phosphor-svelte";
import { onMount } from "svelte";

// Props
let { kind, title, closeModal } = $props<{
    kind: number;
    title: string;
    closeModal: () => void;
}>();

// State
let relays = $state<string[]>([]);
let relayPermissions = $state<Map<string, string>>(new Map());
let newRelayUrl = $state("");
let isLoading = $state(true);
let isSubmitting = $state(false);
let readWriteMode = $state<"ReadWrite" | "Read" | "Write">("ReadWrite");

// For normal relays (10002), we need to handle read/write permissions
const isNormalRelayList = $derived(kind === 10002);

// Toast for notifications
const toastState = getToastState();

// Load relay list based on kind
async function loadRelayList() {
    isLoading = true;
    try {
        const account = $activeAccount;
        if (account) {
            let contact: EnrichedContact = await invoke("fetch_enriched_contact", {
                pubkey: account.pubkey,
                updateAccount: false,
            });

            // Set relays based on kind
            if (kind === 10002) {
                relays = contact.nostr_relays;
                // For now, we don't have permissions info from the backend
                // In a real implementation, we would fetch this from the backend
                for (const url of relays) {
                    relayPermissions.set(url, "ReadWrite");
                }
            } else if (kind === 10050) {
                relays = contact.inbox_relays;
            } else if (kind === 10051) {
                relays = contact.key_package_relays;
            }
        }
    } catch (error) {
        console.error(`Failed to load relay list for kind ${kind}:`, error);
        toastState.add("Error", `Failed to load relay list: ${error}`, "error");
    } finally {
        isLoading = false;
    }
}

// Add a new relay
async function addRelay() {
    if (!newRelayUrl.trim()) return;

    // Basic URL validation
    if (!newRelayUrl.startsWith("wss://") && !newRelayUrl.startsWith("ws://")) {
        toastState.add("Invalid URL", "Relay URL must start with wss:// or ws://", "error");
        return;
    }

    // Check if relay already exists
    if (relays.includes(newRelayUrl)) {
        toastState.add("Duplicate", "This relay is already in your list", "info");
        return;
    }

    isSubmitting = true;

    try {
        // Add to local list first
        const updatedRelays = [...relays, newRelayUrl];

        if (isNormalRelayList) {
            // For NIP-65 relays, we need to handle permissions
            const relayEntries = updatedRelays.map((url) => {
                if (url === newRelayUrl) {
                    return [url, readWriteMode] as [string, string];
                }
                return [url, relayPermissions.get(url) || "ReadWrite"] as [string, string];
            });

            // Save to backend using the NIP-65 command
            await invoke("publish_relay_list", {
                relayEntries: relayEntries,
                kind: kind,
            });

            // Update local state
            relays = updatedRelays;
            relayPermissions.set(newRelayUrl, readWriteMode);

            newRelayUrl = "";
            readWriteMode = "ReadWrite";

            // Notify success
            toastState.add("Relay Added", "Relay has been added to your list", "success");

            // Emit event to refresh relay lists
            emit("account_changed", {});
            return;
        }

        // For other relay types, use the standard command
        await invoke("publish_relay_list", {
            relays: updatedRelays,
            kind: kind,
        });

        // Update local state
        relays = updatedRelays;
        newRelayUrl = "";

        // Notify success
        toastState.add("Relay Added", "Relay has been added to your list", "success");

        // Emit event to refresh relay lists
        emit("account_changed", {});
    } catch (error) {
        console.error("Failed to add relay:", error);
        toastState.add("Error", `Failed to add relay: ${error}`, "error");
    } finally {
        isSubmitting = false;
    }
}

// Update relay permission
async function updateRelayPermission(url: string, permission: string) {
    if (!isNormalRelayList) return;

    isSubmitting = true;

    try {
        // Update local state first
        relayPermissions.set(url, permission);

        // Create relay entries with updated permission
        const relayEntries = relays.map((relayUrl) => {
            return [relayUrl, relayPermissions.get(relayUrl) || "ReadWrite"] as [string, string];
        });

        // Save to backend
        await invoke("publish_relay_list", {
            relayEntries: relayEntries,
            kind: kind,
        });

        // Notify success
        toastState.add("Permission Updated", "Relay permission has been updated", "success");

        // Emit event to refresh relay lists
        emit("account_changed", {});
    } catch (error) {
        console.error("Failed to update relay permission:", error);
        toastState.add("Error", `Failed to update permission: ${error}`, "error");
    } finally {
        isSubmitting = false;
    }
}

// Remove a relay
async function removeRelay(url: string) {
    isSubmitting = true;

    try {
        // Remove from local list first
        const updatedRelays = relays.filter((relay) => relay !== url);

        if (isNormalRelayList) {
            // For NIP-65 relays, we need to handle permissions
            const relayEntries = updatedRelays.map((relayUrl) => {
                return [relayUrl, relayPermissions.get(relayUrl) || "ReadWrite"] as [
                    string,
                    string,
                ];
            });

            // Save to backend
            await invoke("publish_relay_list", {
                relayEntries: relayEntries,
                kind: kind,
            });

            // Update local state
            relays = updatedRelays;
            relayPermissions.delete(url);
        } else {
            // For other relay types, use the standard command
            await invoke("publish_relay_list", {
                relays: updatedRelays,
                kind: kind,
            });

            // Update local state
            relays = updatedRelays;
        }

        // Notify success
        toastState.add("Relay Removed", "Relay has been removed from your list", "success");

        // Emit event to refresh relay lists
        emit("account_changed", {});
    } catch (error) {
        console.error("Failed to remove relay:", error);
        toastState.add("Error", `Failed to remove relay: ${error}`, "error");
    } finally {
        isSubmitting = false;
    }
}

// Load data on mount
onMount(() => {
    loadRelayList();
});
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
        {#if isLoading}
            <div class="p-4 text-center text-gray-400">
                Loading...
            </div>
        {:else if relays.length === 0}
            <div class="p-4 text-center text-gray-400">
                No relays configured. Add one below.
            </div>
        {:else}
            <ul class="divide-y divide-gray-700">
                {#each relays as url}
                    <li class="py-3 flex flex-col gap-2">
                        <div class="flex items-center justify-between">
                            <span class="text-sm font-medium flex items-center gap-2">
                                <HardDrives size={20} class="text-blue-500" />
                                {url}
                            </span>
                            <button
                                class="p-2 rounded-md hover:bg-gray-700 transition-colors text-red-500 hover:text-red-400 disabled:opacity-50"
                                onclick={() => removeRelay(url)}
                                disabled={isSubmitting}
                            >
                                <Trash size={18} />
                            </button>
                        </div>

                        {#if isNormalRelayList}
                            <div class="flex items-center gap-2">
                                <span class="text-xs text-gray-400">Permission:</span>
                                <select
                                    value={relayPermissions.get(url) || "ReadWrite"}
                                    onchange={(e) => updateRelayPermission(url, e.currentTarget.value)}
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

            {#if isNormalRelayList}
                <select
                    bind:value={readWriteMode}
                    class="w-full px-3 py-2 bg-transparent ring-1 ring-gray-700 rounded-md"
                    disabled={isSubmitting}
                >
                    <option value="ReadWrite">Read & Write</option>
                    <option value="Read">Read Only</option>
                    <option value="Write">Write Only</option>
                </select>
            {/if}

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
