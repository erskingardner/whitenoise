<script lang="ts">
import { goto } from "$app/navigation";
import Loader from "$lib/components/Loader.svelte";
import {
    LoginError,
    activeAccount,
    createAccount,
    login,
    updateAccountsStore,
} from "$lib/stores/accounts";
import { isValidHexPubkey } from "$lib/types/nostr";
import { invoke } from "@tauri-apps/api/core";
import { type UnlistenFn, listen } from "@tauri-apps/api/event";
import { onDestroy, onMount } from "svelte";
import { expoInOut } from "svelte/easing";
import { type FlyParams, fly } from "svelte/transition";

let nsecOrHex = $state("");
let loading = $state(true);
let loginError = $state<LoginError | null>(null);
let flyParams: FlyParams = { duration: 150, easing: expoInOut, y: window.innerHeight };

let unlistenAccountChanged: UnlistenFn;
let unlistenNostrReady: UnlistenFn;

onMount(async () => {
    if (!unlistenAccountChanged) {
        unlistenAccountChanged = await listen<string>("account_changed", (_event) => {
            updateAccountsStore().then(async () => {
                console.log("Event received on root page: account_changed");
                loading = false;
                goto("/chats");
            });
        });
    }

    if (!unlistenNostrReady) {
        unlistenNostrReady = await listen<string>("nostr_ready", async (_event) => {
            console.log("Event received on root page: nostr_ready");
        });
    }

    updateAccountsStore().then(async () => {
        loading = false;
        if ($activeAccount?.pubkey && isValidHexPubkey($activeAccount?.pubkey)) {
            await invoke("init_nostr_for_current_user");
            console.log("Initialized Nostr for current user");
        }
    });
});

onDestroy(() => {
    unlistenAccountChanged?.();
    unlistenNostrReady?.();
});

async function handleLogin(e: Event) {
    e.preventDefault();
    if (loading) return;
    loading = true;
    login(nsecOrHex).catch((error) => {
        loginError = error;
        loading = false;
    });
}

async function handleCreateAccount() {
    if (loading) return;
    loading = true;
    createAccount().catch((error) => {
        loginError = error;
        loading = false;
    });
}
</script>

<div class="flex flex-col items-center justify-center w-screen h-dvh bg-gray-800" transition:fly={flyParams}>
    <div class="bg-gray-800 w-full h-2/3 flex flex-col items-center justify-center gap-6 py-12 px-6">
        <img src="whitenoise-login-logo2.png" alt="logo" class="w-32 lg:w-40" />
        <h2 class="text-xl lg:text-2xl font-medium text-center">Secure. Distributed. Uncensorable.</h2>
        <div class="h-[40px]">
            {#if loading}
                <Loader size={40} fullscreen={false} />
            {/if}
        </div>
        <form onsubmit={handleLogin} class="w-full md:w-4/5 flex flex-col gap-4">
            <input
                bind:value={nsecOrHex}
                type="password"
                placeholder="nsec1&hellip;"
                autocorrect="off"
                autocapitalize="off"
                class="text-lg px-3 py-2 bg-transparent ring-1 ring-gray-700 rounded-md"
            />
            {#if loginError}
                <p class="text-red-500">{loginError.message}</p>
            {/if}
            <button
                type="submit"
                disabled={loading}
                class="p-3 font-semibold bg-blue-700 hover:bg-blue-600 rounded-md ring-1 ring-blue-500"
            >
                Log In
            </button>
        </form>

        <h3 class="font-semibold text-gray-400">OR</h3>
        <button
            disabled={loading}
            class="p-3 w-full md:w-4/5 font-semibold bg-indigo-700 hover:bg-indigo-600 rounded-md ring-1 ring-indigo-500"
            onclick={handleCreateAccount}
        >
            Create a new Nostr identity
        </button>
    </div>
</div>
