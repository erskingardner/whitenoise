<script lang="ts">
    import SidebarHeader from "./SidebarHeader.svelte";
    import Contact from "./Contact.svelte";
    import Loader from "./Loader.svelte";
    import { UserPlus } from "phosphor-svelte";
    import type { NUsers, NMetadata } from "../types/nostr";
    import { onMount, onDestroy } from "svelte";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import { invoke } from "@tauri-apps/api/core";
    import { createEventDispatcher } from "svelte";
    // The pubkey of the currently selected contact
    let selectedContact: string | undefined = $state(undefined);
    let contacts: NUsers = $state({});
    let isLoading = $state(true);
    let searchTerm = $state("");
    let contactSearch = $state("");
    let selectedContactMetadata: NMetadata | undefined = $derived(
        selectedContact ? (contacts[selectedContact] as NMetadata) : undefined
    );

    const dispatch = createEventDispatcher();
    let modalVisible = $state(false);

    let filteredContacts: NUsers = $derived(
        Object.fromEntries(
            Object.entries(contacts).filter(([_pubkey, metadata]) => {
                return (
                    (metadata as NMetadata).name
                        ?.toLowerCase()
                        .includes(searchTerm.toLowerCase()) ||
                    (metadata as NMetadata).display_name
                        ?.toLowerCase()
                        .includes(searchTerm.toLowerCase()) ||
                    (metadata as NMetadata).nip05?.toLowerCase().includes(searchTerm.toLowerCase())
                );
            })
        )
    );

    async function getContacts(): Promise<void> {
        isLoading = true;
        contacts = {};
        selectedContact = undefined;
        try {
            const fetchedContacts: NUsers = await invoke("get_contacts");
            contacts = fetchedContacts;
        } catch (error) {
            console.error("Error fetching contacts:", error);
        } finally {
            isLoading = false;
        }
    }

    function handleSearch(event: CustomEvent<string>) {
        searchTerm = event.detail;
    }

    let unlisten: UnlistenFn;

    onMount(async () => {
        getContacts();
        unlisten = await listen<string>("identity_change", (_event) => getContacts());
    });

    onDestroy(() => {
        unlisten();
    });

    function toggleContactModal() {
        modalVisible = !modalVisible;
    }

    async function submitContactsSearch(event: KeyboardEvent | MouseEvent) {
        // TODO: implement contacts search
        if (event instanceof KeyboardEvent) {
            const { key } = event;
            if (key === "Enter") console.log("Submitted by keyboard");
        } else {
            console.log("Submitted by mouse");
        }
    }
</script>

<SidebarHeader
    title="New Chat"
    newIcon={UserPlus}
    showBackIcon={true}
    on:search={handleSearch}
    on:newIconClicked={toggleContactModal}
    on:backIconClicked={() => dispatch("backIconClicked")}
></SidebarHeader>
{#if isLoading}
    <div class="w-full h-10 mt-4 flex items-center justify-center">
        <Loader size={40} />
    </div>
{/if}
{#if !isLoading && Object.keys(filteredContacts).length === 0}
    <div class="text-gray-500 w-full p-4 text-center">No contacts found</div>
{/if}
{#each Object.entries(filteredContacts) as [pubkey, metadata] (pubkey)}
    <button onclick={() => (selectedContact = pubkey)} class="w-full">
        <Contact {pubkey} metadata={metadata as NMetadata} active={pubkey === selectedContact} />
    </button>
{/each}
