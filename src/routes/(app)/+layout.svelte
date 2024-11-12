<script lang="ts">
    import { page } from "$app/stores";
    import Sidebar from "$lib/components/Sidebar.svelte";
    import Tabbar from "$lib/components/Tabbar.svelte";
    import { onMount, onDestroy } from "svelte";
    import { updateAccountsStore, accounts, type Account } from "$lib/stores/accounts";
    import { goto } from "$app/navigation";
    import Modal from "$lib/components/Modals/Modal.svelte";
    import PreOnboard from "$lib/components/Modals/Onboarding/PreOnboard.svelte";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";

    let { children } = $props();

    let activeTab = $derived($page.url.pathname.split("/")[1] || "chats");
    let isLoadingAccounts = $state(true);

    let unlistenNostrReady: UnlistenFn;

    let keyPackagePublished = $state(false);
    let keyPackageRelaysPublished = $state(false);
    let inboxRelaysPublished = $state(false);
    let showPreflightModal = $state(false);

    $effect(() => {
        showPreflightModal = !keyPackageRelaysPublished || !inboxRelaysPublished || !keyPackagePublished;
    });

    async function checkPreflight() {
        await updateAccountsStore();
        isLoadingAccounts = false;

        if (!!!$accounts.activeAccount) {
            goto("/");
        }

        if ($accounts.activeAccount) {
            let activeAccount: Account | undefined = $accounts.accounts.filter(
                (account: Account) => account.pubkey === $accounts.activeAccount
            )[0];
            if (activeAccount) {
                if (activeAccount.onboarding.inbox_relays) {
                    inboxRelaysPublished = true;
                }
                if (activeAccount.onboarding.key_package_relays) {
                    keyPackageRelaysPublished = true;
                }
                if (activeAccount.onboarding.publish_key_package) {
                    keyPackagePublished = true;
                }
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
    <Tabbar {activeTab} />
    <div class="flex flex-col grow">
        {@render children()}
    </div>
</main>

{#if showPreflightModal}
    <Modal mainComponent={PreOnboard} bind:showModal={showPreflightModal} />
{/if}
