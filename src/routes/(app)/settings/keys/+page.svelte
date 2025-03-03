<script lang="ts">
import { goto } from "$app/navigation";
import Header from "$lib/components/Header.svelte";
import HeaderToolbar from "$lib/components/HeaderToolbar.svelte";
import { activeAccount } from "$lib/stores/accounts";
import { npubFromPubkey } from "$lib/utils/nostr";
import { invoke } from "@tauri-apps/api/core";
import { CaretLeft, Copy, Eye, EyeSlash } from "phosphor-svelte";
import { onMount } from "svelte";
import type { PageData } from "./$types";

let { data }: { data: PageData } = $props();

let pubkey = $state("");
let npub = $state("");
let nsec = $state("");
let showPrivateKey = $state(false);
let pubkeyCopySuccess = $state(false);
let npubCopySuccess = $state(false);
let nsecCopySuccess = $state(false);

async function loadKeys() {
    try {
        const account = $activeAccount;
        if (account) {
            pubkey = account.pubkey;
            npub = npubFromPubkey(pubkey);
            // We'll load the private key only when needed
        }
    } catch (error) {
        console.error("Failed to load keys:", error);
    }
}

async function togglePrivateKey() {
    if (!showPrivateKey && !nsec) {
        try {
            nsec = await invoke("export_nsec", { pubkey });
        } catch (error) {
            console.error("Failed to load private key:", error);
        }
    }
    showPrivateKey = !showPrivateKey;
}

async function copyToClipboard(text: string, type: "pubkey" | "npub" | "nsec") {
    try {
        await navigator.clipboard.writeText(text);
        if (type === "pubkey") {
            pubkeyCopySuccess = true;
        } else if (type === "npub") {
            npubCopySuccess = true;
        } else if (type === "nsec") {
            nsecCopySuccess = true;
        }
        setTimeout(() => {
            pubkeyCopySuccess = false;
            npubCopySuccess = false;
            nsecCopySuccess = false;
        }, 2000);
    } catch (error) {
        console.error("Failed to copy to clipboard:", error);
    }
}

function goBack() {
    goto("/settings");
}

onMount(() => {
    loadKeys();
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
        <h1>Your Nostr Keys</h1>
    {/snippet}
</HeaderToolbar>

<Header title="Your Nostr Keys" />

<main class="px-4 flex flex-col pb-32">
    <h2 class="section-title">Public Key</h2>
    <div class="section">
        <ul class="section-list">
            <li class="section-list-item">
                <div class="flex flex-col w-full gap-1">
                    <span class="text-sm font-medium">Hex Format</span>
                    <div class="flex gap-2">
                        <input
                            type="text"
                            value={pubkey}
                            readonly
                            class="font-mono w-full px-4 py-2 rounded-lg border dark:border-gray-700 bg-white dark:bg-gray-800"
                        />
                        <button
                            class="p-2 rounded-md hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                            onclick={() => copyToClipboard(pubkey, 'pubkey')}
                        >
                            <Copy size={20} weight={pubkeyCopySuccess ? "fill" : "regular"} class={`${pubkeyCopySuccess ? "text-green-500" : ""} transition-colors duration-200`} />
                        </button>
                    </div>
                </div>
            </li>
            <li class="section-list-item">
                <div class="flex flex-col w-full gap-1">
                    <span class="text-sm font-medium">npub Format</span>
                    <div class="flex gap-2">
                        <input
                            type="text"
                            value={npub}
                            readonly
                            class="font-mono w-full px-4 py-2 rounded-lg border dark:border-gray-700 bg-white dark:bg-gray-800"
                        />
                        <button
                            class="p-2 rounded-md hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                            onclick={() => copyToClipboard(npub, 'npub')}
                        >
                            <Copy size={20} weight={npubCopySuccess ? "fill" : "regular"} class={`${npubCopySuccess ? "text-green-500" : ""} transition-colors duration-200`} />
                        </button>
                    </div>
                </div>
            </li>
        </ul>
    </div>

    <h2 class="section-title">Private Key</h2>
    <div class="section">
        <ul class="section-list">
            <li class="section-list-item">
                <div class="flex flex-col w-full gap-1">
                    <span class="text-sm font-medium">nsec Format</span>
                    <div class="flex gap-2">
                        <input
                            type={showPrivateKey ? "text" : "password"}
                            value={nsec}
                            readonly
                            placeholder="Click the eye icon to reveal"
                            class="font-mono w-full px-4 py-2 rounded-lg border dark:border-gray-700 bg-white dark:bg-gray-800"
                        />
                        <button
                            class="p-2 rounded-md hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                            onclick={togglePrivateKey}
                        >
                            {#if showPrivateKey}
                                <EyeSlash size={20} />
                            {:else}
                                <Eye size={20} />
                            {/if}
                        </button>
                        <button
                            class="p-2 rounded-md hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                            onclick={() => copyToClipboard(nsec, 'nsec')}
                            disabled={!nsec}
                        >
                            <Copy size={20} weight={nsecCopySuccess ? "fill" : "regular"} class={`${nsecCopySuccess ? "text-green-500" : ""} transition-colors duration-200`} />
                        </button>
                    </div>
                </div>
            </li>
        </ul>
    </div>
</main>
