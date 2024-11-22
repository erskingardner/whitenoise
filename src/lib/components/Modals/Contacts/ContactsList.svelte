<script lang="ts">
    import Avatar from "../../Avatar.svelte";
    import Name from "../../Name.svelte";
    import { npubFromPubkey } from "$lib/utils/nostr";
    import { CaretRight } from "phosphor-svelte";
    import ContactDetail from "./ContactDetail.svelte";
    import type { EnrichedContactsMap, EnrichedContact } from "$lib/types/nostr";
    import type { PushView } from "$lib/types/modal";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import { onMount, onDestroy } from "svelte";
    import { getToastState } from "$lib/stores/toast-state.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import Loader from "../../Loader.svelte";

    let toastState = getToastState();

    let unlistenAccountChanging: UnlistenFn;
    let unlistenAccountChanged: UnlistenFn;
    let unlistenNostrReady: UnlistenFn;

    let { pushView } = $props<{
        pushView: PushView;
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
            Object.entries(contactsResponse as EnrichedContactsMap).sort(([_keyA, contactA], [_keyB, contactB]) => {
                const nameA = contactA.metadata.display_name || contactA.metadata.name || "";
                const nameB = contactB.metadata.display_name || contactB.metadata.name || "";
                return nameA.localeCompare(nameB);
            })
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
                        pubkey.toLowerCase().includes(search.toLowerCase())
                )
            );
        }
    });

    function viewContact(pubkey: string, contact: EnrichedContact): void {
        pushView(ContactDetail, { pubkey, contact });
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
    {/if}
    {#if filteredContacts && Object.keys(filteredContacts).length > 0}
        {#each Object.entries(filteredContacts) as [pubkey, contact]}
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
    {/if}
</div>
