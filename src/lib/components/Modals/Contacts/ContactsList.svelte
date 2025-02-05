<script lang="ts">
import { getToastState } from "$lib/stores/toast-state.svelte";
import type { PushView } from "$lib/types/modal";
import type { EnrichedContact, EnrichedContactsMap } from "$lib/types/nostr";
import { npubFromPubkey } from "$lib/utils/nostr";
import { invoke } from "@tauri-apps/api/core";
import { type UnlistenFn, listen } from "@tauri-apps/api/event";
import { CaretRight } from "phosphor-svelte";
import { onDestroy, onMount } from "svelte";
import Avatar from "../../Avatar.svelte";
import Loader from "../../Loader.svelte";
import Name from "../../Name.svelte";
import ContactDetail from "./ContactDetail.svelte";

let toastState = getToastState();

let unlistenAccountChanging: UnlistenFn;
let unlistenAccountChanged: UnlistenFn;
let unlistenNostrReady: UnlistenFn;

let { pushView } = $props<{
    pushView?: PushView;
}>();

let isLoading = $state(true);
// TODO: create a load error state
let loadingError = $state<string | null>(null);

let contacts = $state<EnrichedContactsMap>({});
let search = $state("");
let filteredContacts = $state<EnrichedContactsMap>({});

async function loadContacts() {
    const contactsResponse = await invoke("fetch_enriched_contacts");
    // Sort contacts by name
    contacts = Object.fromEntries(
        Object.entries(contactsResponse as EnrichedContactsMap).sort(
            ([_keyA, contactA], [_keyB, contactB]) => {
                const nameA = contactA.metadata.display_name || contactA.metadata.name || "";
                const nameB = contactB.metadata.display_name || contactB.metadata.name || "";
                // If either name is empty, sort it to the bottom
                if (!nameA && !nameB) return 0;
                if (!nameA) return 1;
                if (!nameB) return -1;
                // Otherwise do normal string comparison
                return nameA.localeCompare(nameB);
            }
        )
    );
    isLoading = false;
}

onMount(async () => {
    await loadContacts();

    if (!unlistenAccountChanging) {
        unlistenAccountChanging = await listen<string>("account_changing", async (_event) => {
            console.log("Event received in contacts list: account_changing");
            contacts = {};
        });
    }

    if (!unlistenAccountChanged) {
        unlistenAccountChanged = await listen<string>("account_changed", async (_event) => {
            console.log("Event received in contacts list: account_changed");
        });
    }

    if (!unlistenNostrReady) {
        unlistenNostrReady = await listen<string>("nostr_ready", async (_event) => {
            console.log("Event received in contacts list: nostr_ready");
            await loadContacts();
        });
    }
});

onDestroy(() => {
    unlistenAccountChanging?.();
    unlistenAccountChanged?.();
    unlistenNostrReady?.();
    toastState.cleanup();
});

$effect(() => {
    if (!search || search === "") {
        filteredContacts = contacts;
    } else {
        filteredContacts = Object.fromEntries(
            Object.entries(contacts as EnrichedContactsMap).filter(
                ([pubkey, contact]) =>
                    contact.metadata.name?.toLowerCase().includes(search.toLowerCase()) ||
                    contact.metadata.display_name?.toLowerCase().includes(search.toLowerCase()) ||
                    pubkey.toLowerCase().includes(search.toLowerCase()) ||
                    npubFromPubkey(pubkey).toLowerCase().includes(search.toLowerCase())
            )
        );
    }
});

function viewContact(pubkey: string, contact: EnrichedContact): void {
    pushView(ContactDetail, { pubkey, contact });
}

function searchRelays(): void {
    console.log(`Searching relays for "${search}"...`);
}
</script>

<input
    type="search"
    placeholder="Search..."
    bind:value={search}
    class="bg-transparent ring-1 ring-gray-700 rounded-md px-3 py-2 w-full"
/>
<div class="flex flex-col mt-10">
    {#if isLoading}
        <Loader size={40} fullscreen={false} />
    {:else}
        {#if filteredContacts && Object.keys(filteredContacts).length > 0}
            {#each Object.entries(filteredContacts) as [pubkey, contact] (pubkey)}
                <button
                    onclick={() => viewContact(pubkey, contact)}
                    class="flex flex-row gap-2 items-center px-2 py-3 border-b border-gray-700 hover:bg-gray-700"
                >
                    <Avatar {pubkey} picture={contact.metadata.picture} pxSize={40} />
                    <div class="flex flex-col items-start justify-start truncate">
                        <Name {pubkey} metadata={contact.metadata} />
                        <span class="text-gray-400 text-sm font-mono">{npubFromPubkey(pubkey)}</span>
                    </div>
                    <CaretRight size={20} class="ml-auto" />
                </button>
            {/each}
        {:else}
            <div class="flex flex-col gap-6 items-center justify-center h-full">
                <span class="text-gray-400">No contacts found</span>
                <button class="button-primary" onclick={searchRelays}>Search all of Nostr?</button>
            </div>
        {/if}
    {/if}
</div>
