<script lang="ts">
import { goto } from "$app/navigation";
import { page } from "$app/stores";
import Modal from "$lib/components/Modals/Modal.svelte";
import PreOnboard from "$lib/components/Modals/Onboarding/PreOnboard.svelte";
import Sidebar from "$lib/components/Sidebar.svelte";
import Tabbar from "$lib/components/Tabbar.svelte";
import { type Account, accounts, updateAccountsStore } from "$lib/stores/accounts";
import { invoke } from "@tauri-apps/api/core";
import { type UnlistenFn, listen } from "@tauri-apps/api/event";
import { onDestroy, onMount } from "svelte";

let { children } = $props();

let activeTab = $derived($page.url.pathname.split("/")[1] || "chats");
let isLoadingAccounts = $state(true);

let unlistenNostrReady: UnlistenFn;

// Start with true so we don't show until the preflight checks are done
let keyPackagePublished = $state(true);
let keyPackageRelaysPublished = $state(true);
let inboxRelaysPublished = $state(true);

let showPreflightModal = $state(false);
$effect(() => {
    showPreflightModal =
        !keyPackageRelaysPublished || !inboxRelaysPublished || !keyPackagePublished;
});

async function checkPreflight() {
    await updateAccountsStore();
    isLoadingAccounts = false;

    if (!Boolean($accounts.activeAccount)) {
        goto("/login");
    }

    if ($accounts.activeAccount) {
        let activeAccount: Account | undefined = $accounts.accounts.filter(
            (account: Account) => account.pubkey === $accounts.activeAccount
        )[0];
        if (activeAccount) {
            if (!activeAccount.metadata.display_name || !activeAccount.metadata.picture) {
                await invoke("query_enriched_contact", {
                    pubkey: activeAccount.pubkey,
                    updateAccount: true,
                });
            }
            inboxRelaysPublished = activeAccount.onboarding.inbox_relays;
            keyPackageRelaysPublished = activeAccount.onboarding.key_package_relays;
            keyPackagePublished = activeAccount.onboarding.publish_key_package;
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
});

onDestroy(() => {
    unlistenNostrReady?.();
});
</script>

<main class="flex flex-col md:flex-row min-w-96">
    <Sidebar {activeTab} />
    {#if !$page.url.pathname.match(/^\/chats\/*[a-zA-Z0-9]+\/*/)}
        <Tabbar {activeTab} />
    {/if}
    <div class="flex flex-col grow">
        {@render children()}
    </div>
</main>

{#if showPreflightModal}
    <Modal
        initialComponent={PreOnboard}
        props={{ inboxRelaysPublished, keyPackageRelaysPublished, keyPackagePublished }}
        bind:showModal={showPreflightModal}
    />
{/if}
