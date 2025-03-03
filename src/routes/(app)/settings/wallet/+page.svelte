<script lang="ts">
import { goto } from "$app/navigation";
import Header from "$lib/components/Header.svelte";
import HeaderToolbar from "$lib/components/HeaderToolbar.svelte";
import {
    NostrWalletConnectError,
    hasNostrWalletConnectUri,
    removeNostrWalletConnectUri,
    setNostrWalletConnectUri,
} from "$lib/stores/accounts";
import { CaretLeft, Lightning } from "phosphor-svelte";
import { onMount } from "svelte";

let hasWallet = false;
let nwcUri = "";
let error = "";
let loading = false;

async function checkWalletStatus() {
    try {
        hasWallet = await hasNostrWalletConnectUri();
        error = "";
    } catch (e) {
        if (e instanceof NostrWalletConnectError) {
            error = e.message;
        } else {
            error = "An unexpected error occurred";
        }
    }
}

async function handleSetWallet() {
    if (!nwcUri) return;
    loading = true;
    try {
        await setNostrWalletConnectUri(nwcUri);
        await checkWalletStatus();
        nwcUri = "";
        error = "";
    } catch (e) {
        if (e instanceof NostrWalletConnectError) {
            error = e.message;
        } else {
            error = "An unexpected error occurred";
        }
    } finally {
        loading = false;
    }
}

async function handleRemoveWallet() {
    loading = true;
    try {
        await removeNostrWalletConnectUri();
        await checkWalletStatus();
        error = "";
    } catch (e) {
        if (e instanceof NostrWalletConnectError) {
            error = e.message;
        } else {
            error = "An unexpected error occurred";
        }
    } finally {
        loading = false;
    }
}

function goBack() {
    goto("/settings");
}

onMount(() => {
    checkWalletStatus();
});
</script>

<HeaderToolbar>
    {#snippet left()}
        <button class="flex flex-row gap-0.5 items-center" on:click={goBack}>
            <CaretLeft size={24} weight="bold" />
            <span class="font-medium text-lg">Back</span>
        </button>
    {/snippet}
    {#snippet center()}
        <h1>Wallet</h1>
    {/snippet}
</HeaderToolbar>

<Header title="Wallet" />
<main class="px-4 flex flex-col">
    <h2 class="section-title">
        Nostr Wallet Connect
    </h2>
    <section class="flex flex-col gap-4">
        <div class="section">
            <ul class="section-list">
                <li class="section-list-item">
                    {#if hasWallet}
                        <div class="flex flex-col gap-4">
                            <p class="text-green-500">
                                You have already configured your Nostr Wallet Connect
                            </p>
                            <button
                                class="flex flex-row gap-2 items-center px-2 py-3 hover:bg-gray-700 w-full"
                                on:click={handleRemoveWallet}
                                disabled={loading}
                            >
                                {loading ? 'Removing...' : 'Remove Wallet Connection'}
                            </button>
                        </div>
                    {:else}
                        <div class="flex flex-col gap-4">
                            <div class="form-control w-full">
                                <label class="block text-sm font-medium mb-1" for="nwc-uri">
                                    Nostr Wallet Connect URI
                                </label>
                                <input
                                    type="text"
                                    id="nwc-uri"
                                    class="w-full px-4 py-2 rounded-lg border dark:border-gray-700 bg-white dark:bg-gray-800 focus:ring-2 focus:ring-blue-500 dark:focus:ring-blue-400"
                                    placeholder="nostr+walletconnect://"
                                    bind:value={nwcUri}
                                />
                                {#if error}
                                    <div class="text-red-500">{error}</div>
                                {/if}
                            </div>
                            <button
                                class="button-primary"
                                on:click={handleSetWallet}
                                disabled={!nwcUri || loading}
                            >
                                {loading ? 'Saving...' : 'Save Wallet Connection'}
                            </button>
                        </div>
                    {/if}
                </li>
            </ul>
        </div>
    </section>
</main>
