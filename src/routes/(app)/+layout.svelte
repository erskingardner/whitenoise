<script lang="ts">
import { goto } from "$app/navigation";
import { page } from "$app/state";
import Sidebar from "$lib/components/Sidebar.svelte";
import Tabbar from "$lib/components/Tabbar.svelte";
import { activeAccount, updateAccountsStore } from "$lib/stores/accounts";
import { invoke } from "@tauri-apps/api/core";
import { type UnlistenFn, listen } from "@tauri-apps/api/event";
import { isPermissionGranted, requestPermission } from "@tauri-apps/plugin-notification";
import { onDestroy, onMount } from "svelte";

let { children } = $props();

let activeTab = $derived(page.url.pathname.split("/")[1] || "chats");
let isLoadingAccounts = $state(true);

let unlistenNostrReady: UnlistenFn;

async function checkPreflight() {
    await updateAccountsStore();
    isLoadingAccounts = false;

    if (!$activeAccount) {
        goto("/login");
    }

    if ($activeAccount) {
        if (!$activeAccount.metadata.display_name || !$activeAccount.metadata.picture) {
            await invoke("query_enriched_contact", {
                pubkey: $activeAccount.pubkey,
                updateAccount: true,
            });
        }
    }
}

onMount(async () => {
    if (!unlistenNostrReady) {
        unlistenNostrReady = await listen<string>("nostr_ready", async (_event) => {
            console.log("Event received on layout page: nostr_ready");
            checkPreflight();
        });
    }

    checkPreflight();

    // Do you have permission to send a notification?
    let permissionGranted = await isPermissionGranted();

    // If not we need to request it
    if (!permissionGranted) {
        console.log("Requesting permission");
        const permission = await requestPermission();
        permissionGranted = permission === "granted";
    }
});

onDestroy(() => {
    unlistenNostrReady?.();
});
</script>

<main class="flex flex-col md:flex-row min-h-screen">
    <Sidebar {activeTab} />
    {#if !page.url.pathname.match(/^\/chats\/*[a-zA-Z0-9]+\/*/)}
        <Tabbar {activeTab} />
    {/if}
    <div class="flex flex-col grow md:w-4/5 bg-gray-900">
        {@render children()}
    </div>
</main>
