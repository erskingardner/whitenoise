<script lang="ts">
    import { accounts, createAccount, login, LoginError, updateAccountsStore } from "$lib/stores/accounts";
    import { goto } from "$app/navigation";
    import { fly, type FlyParams } from "svelte/transition";
    import { expoInOut } from "svelte/easing";
    import { onMount, onDestroy } from "svelte";
    import { WifiSlash } from "phosphor-svelte";
    import { isValidHexPubkey } from "$lib/types/nostr";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import Loader from "$lib/components/Loader.svelte";
    import { invoke } from "@tauri-apps/api/core";

    let nsecOrHex = $state("");
    let loading = $state(true);
    let loginError = $state<LoginError | null>(null);
    let flyParams: FlyParams = { duration: 300, easing: expoInOut, y: window.innerHeight };

    let unlistenAccountChanged: UnlistenFn;
    let unlistenNostrReady: UnlistenFn;

    onMount(async () => {
        if (!unlistenAccountChanged) {
            unlistenAccountChanged = await listen<string>("account_changed", (_event) => {
                updateAccountsStore().then(async () => {
                    console.log("Event received on root page: account_changed");
                });
            });
        }

        if (!unlistenNostrReady) {
            unlistenNostrReady = await listen<string>("nostr_ready", (_event) => {
                console.log("Event received on root page: nostr_ready");
                loading = false;
                goto("/chats");
            });
        }

        updateAccountsStore().then(() => {
            loading = false;
            if ($accounts.activeAccount && isValidHexPubkey($accounts.activeAccount)) {
                invoke("init_nostr_for_current_user");
            }
        });
    });

    onDestroy(() => {
        unlistenAccountChanged?.();
        unlistenNostrReady?.();
    });

    async function handleLogin(e: Event) {
        e.preventDefault();
        console.log("handleLogin");
        if (loading) return;
        loading = true;
        login(nsecOrHex).catch((error) => {
            loginError = error;
        });
    }

    async function handleCreateAccount() {
        if (loading) return;
        loading = true;
        createAccount().catch((error) => {
            loginError = error;
        });
    }
</script>

{#if false}
    <!-- Change this to be dependent on online state-->
    <div class="bg-gray-800 py-4">
        <span class="text-red-500 flex flex-row gap-2 items-center justify-center">
            <WifiSlash size={20} />
            Offline
        </span>
    </div>
{/if}
<div class="flex flex-col items-center justify-center w-screen h-screen bg-gray-800" transition:fly={flyParams}>
    <div class="bg-gray-800 w-full md:w-1/2 h-2/3 flex flex-col items-center justify-center gap-6 py-12 px-6">
        <h1 class="text-5xl font-extrabold text-center">White Noise</h1>
        <h2 class="text-3xl font-medium text-center">Secure. Distributed. Uncensorable.</h2>
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
    <div class="flex flex-row gap-1 items-end mt-20">
        Powered by
        <img src="/nostr.webp" alt="nostr" class="w-20" />
    </div>
</div>
