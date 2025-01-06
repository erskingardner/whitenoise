<script lang="ts">
import Header from "$lib/components/Header.svelte";
import HeaderToolbar from "$lib/components/HeaderToolbar.svelte";
import Loader from "$lib/components/Loader.svelte";
import PrivateMessageListItem from "$lib/components/PrivateMessageListItem.svelte";
import { accounts } from "$lib/stores/accounts";
import { getToastState } from "$lib/stores/toast-state.svelte";
import type { NEvent, NLegacies } from "$lib/types/nostr";
import { invoke } from "@tauri-apps/api/core";
import { type UnlistenFn, listen } from "@tauri-apps/api/event";
import { PlusCircle, Warning } from "phosphor-svelte";
import { onDestroy, onMount } from "svelte";

let unlistenAccountChanging: UnlistenFn;
let unlistenAccountChanged: UnlistenFn;
let unlistenNostrReady: UnlistenFn;

let toastState = getToastState();

let isLoading = $state(false);
let loadingError = $state<string | null>(null);

let privateMessages = $state<NLegacies>();

async function loadEvents() {
    isLoading = true;
    isLoading = false;
    // invoke("fecth_nip17_private_messages")
    //     .then((messages) => {
    //         privateMessages = messages as NLegacies;
    //     })
    //     .catch((error) => {
    //         loadingError = error as string;
    //         console.log(error);
    //     })
    //     .finally(() => {
    //         isLoading = false;
    //     });
}

onMount(async () => {
    if ($accounts.activeAccount) {
        await loadEvents();
    }

    if (!unlistenAccountChanging) {
        unlistenAccountChanging = await listen<string>("account_changing", async (_event) => {
            console.log("Event received on legacy chats page: account_changing");
            isLoading = true;
            privateMessages = undefined;
        });
    }

    if (!unlistenAccountChanged) {
        unlistenAccountChanged = await listen<string>("account_changed", async (_event) => {
            console.log("Event received on legacy chats page: account_changed");
        });
    }

    if (!unlistenNostrReady) {
        unlistenNostrReady = await listen<string>("nostr_ready", async (_event) => {
            console.log("Event received on legacy chats page: nostr_ready");
            if ($accounts.activeAccount) {
                await loadEvents();
            }
        });
    }
});

onDestroy(() => {
    unlistenAccountChanging?.();
    unlistenAccountChanged?.();
    unlistenNostrReady?.();
    toastState.cleanup();
});
</script>

<HeaderToolbar>
    {#snippet right()}
        <div>
            <button onclick={() => console.log("plus clicked")} class="p-2 -mr-2">
                <PlusCircle size={30} />
            </button>
        </div>
    {/snippet}
    {#snippet center()}
        <h1>Direct Messages</h1>
    {/snippet}
</HeaderToolbar>

<Header title="Direct Messages" />
<main class="">
    {#if isLoading}
        <div class="flex justify-center items-center mt-20 w-full">
            <Loader size={40} fullscreen={false} />
        </div>
    {:else if loadingError}
        <div class="text-red-500 px-4 font-medium flex flex-col gap-2">
            <span>Sorry, we couldn't load your direct messages because of an error.</span>
            <pre class="font-mono p-2 rounded-md ring-1 ring-red-500/30">{loadingError}</pre>
        </div>
    {:else}
        <div class="flex flex-col gap-0">
            {#if !privateMessages || Object.keys(privateMessages).length === 0}
                <div class="flex flex-col gap-2 items-center justify-center h-full">
                    <span class="text-gray-500 mt-40">Two Weeks™</span>
                </div>
            {:else if privateMessages && Object.keys(privateMessages).length > 0}
                {#each Object.entries(privateMessages) as [pubkey, messages]}
                    <PrivateMessageListItem {pubkey} messages={messages} />
                {/each}
            {/if}
        </div>
    {/if}
</main>
