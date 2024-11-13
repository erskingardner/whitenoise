<script lang="ts">
    import Header from "$lib/components/Header.svelte";
    import HeaderToolbar from "$lib/components/HeaderToolbar.svelte";
    import { PlusCircle } from "phosphor-svelte";
    import { invoke } from "@tauri-apps/api/core";
    import type { NostrMlsGroup, Invite, InvitesWithFailures } from "$lib/types/nostr";
    import { onMount, onDestroy } from "svelte";
    import { nameFromMetadata } from "$lib/utils/nostr";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import Loader from "$lib/components/Loader.svelte";
    import { getToastState } from "$lib/stores/toast-state.svelte";
    import type { EnrichedContact } from "$lib/types/nostr";
    import ContactsList from "$lib/components/Modals/Contacts/ContactsList.svelte";
    import Modal from "$lib/components/Modals/Modal.svelte";
    import GroupListItem from "$lib/components/GroupListItem.svelte";
    import InviteListItem from "$lib/components/InviteListItem.svelte";

    let unlistenAccountChanging: UnlistenFn;
    let unlistenAccountChanged: UnlistenFn;
    let unlistenNostrReady: UnlistenFn;

    let toastState = getToastState();

    let createGroupData = $state<{ pubkey: string; contact: EnrichedContact } | null>(null);
    let inviteeName = $derived(
        createGroupData?.contact ? nameFromMetadata(createGroupData.contact.metadata, createGroupData.pubkey) : ""
    );

    let showModal = $state(false);

    let isLoading = $state(true);
    let loadingError = $state<string | null>(null);

    let groups = $state<NostrMlsGroup[]>([]);
    let invites = $state<Invite[]>([]);
    let failures = $state<[string, string][]>([]);

    async function loadEvents() {
        isLoading = true;
        try {
            const [groupsResponse, invitesResponse] = await Promise.all([invoke("get_groups"), invoke("get_invites")]);

            groups = groupsResponse as NostrMlsGroup[];
            invites = (invitesResponse as InvitesWithFailures).invites;
            failures = (invitesResponse as InvitesWithFailures).failures;
        } catch (error) {
            loadingError = error as string;
            console.log(error);
        } finally {
            isLoading = false;
        }
    }

    $inspect("Groups", groups);
    $inspect("Invites", invites);
    $inspect("Failures", failures); // TODO: Store failures in the database so we don't check them in the future

    onMount(async () => {
        await loadEvents();

        if (!unlistenAccountChanging) {
            unlistenAccountChanging = await listen<string>("account_changing", async (_event) => {
                console.log("Event received on chats page: account_changing");
                isLoading = true;
                groups = [];
                invites = [];
            });
        }

        if (!unlistenAccountChanged) {
            unlistenAccountChanged = await listen<string>("account_changed", async (_event) => {
                console.log("Event received on chats page: account_changed");
            });
        }

        if (!unlistenNostrReady) {
            unlistenNostrReady = await listen<string>("nostr_ready", async (_event) => {
                console.log("Event received on chats page: nostr_ready");
                await loadEvents();
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
            <button onclick={() => (showModal = !showModal)} class="p-2 -mr-2">
                <PlusCircle size={30} />
            </button>
        </div>
    {/snippet}
    {#snippet center()}
        <h1>Chats</h1>
    {/snippet}
</HeaderToolbar>

<Header title="Chats" />
<main class="">
    {#if isLoading}
        <div class="flex justify-center items-center mt-20 w-full">
            <Loader size={40} fullscreen={false} />
        </div>
    {:else if loadingError}
        <div class="text-red-500 px-4 font-medium flex flex-col gap-2">
            <span>Sorry, we couldn't load your chats because of an error.</span>
            <pre class="font-mono p-2 rounded-md ring-1 ring-red-500/30">{loadingError}</pre>
        </div>
    {:else}
        <div class="px-4 py-2 bg-gray-800 text-lg font-bold border-t border-b border-gray-700">Invites</div>
        <div class="flex flex-col gap-2">
            {#each invites as invite}
                <InviteListItem {invite} />
            {/each}
        </div>
        <div class="px-4 py-2 bg-gray-800 text-lg font-bold border-t border-b border-gray-700">Groups</div>
        <div class="flex flex-col">
            {#each groups as group}
                <GroupListItem {group} />
            {/each}
        </div>
    {/if}
</main>

{#if showModal}
    <Modal initialComponent={ContactsList} props={{}} bind:showModal />
{/if}
