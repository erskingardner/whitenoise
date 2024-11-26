<script lang="ts">
import GroupListItem from "$lib/components/GroupListItem.svelte";
import Header from "$lib/components/Header.svelte";
import HeaderToolbar from "$lib/components/HeaderToolbar.svelte";
import InviteListItem from "$lib/components/InviteListItem.svelte";
import Loader from "$lib/components/Loader.svelte";
import ContactsList from "$lib/components/Modals/Contacts/ContactsList.svelte";
import Modal from "$lib/components/Modals/Modal.svelte";
import { getToastState } from "$lib/stores/toast-state.svelte";
import type { Invite, InvitesWithFailures, NostrMlsGroup } from "$lib/types/nostr";
import { invoke } from "@tauri-apps/api/core";
import { type UnlistenFn, listen } from "@tauri-apps/api/event";
import { PlusCircle } from "phosphor-svelte";
import { onDestroy, onMount } from "svelte";

let unlistenAccountChanging: UnlistenFn;
let unlistenAccountChanged: UnlistenFn;
let unlistenNostrReady: UnlistenFn;
let unlistenGroupAdded: UnlistenFn;
let unlistenInviteAccepted: UnlistenFn;
let unlistenInviteDeclined: UnlistenFn;

let toastState = getToastState();

let showModal = $state(false);

let isLoading = $state(true);
let loadingError = $state<string | null>(null);

let groups = $state<NostrMlsGroup[]>([]);
let invites = $state<Invite[]>([]);
let failures = $state<[string, string][]>([]);

async function loadEvents() {
    isLoading = true;
    try {
        const [groupsResponse, invitesResponse] = await Promise.all([
            invoke("get_groups"),
            invoke("get_invites"),
        ]);

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

    if (!unlistenGroupAdded) {
        unlistenGroupAdded = await listen<NostrMlsGroup>("group_added", (event) => {
            const addedGroup = event.payload as NostrMlsGroup;
            console.log("Event received on chats page: group_added", addedGroup);
            groups = [...groups, addedGroup];
        });
    }

    if (!unlistenInviteAccepted) {
        unlistenInviteAccepted = await listen<Invite>("invite_accepted", (event) => {
            const acceptedInvite = event.payload as Invite;
            console.log("Event received on chats page: invite_accepted", acceptedInvite);
            invites = invites.filter((invite) => invite.event.id !== acceptedInvite.event.id);
        });
    }

    if (!unlistenInviteDeclined) {
        unlistenInviteDeclined = await listen<Invite>("invite_declined", (event) => {
            const declinedInvite = event.payload as Invite;
            console.log("Event received on chats page: invite_declined", declinedInvite);
            invites = invites.filter((invite) => invite.event.id !== declinedInvite.event.id);
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
