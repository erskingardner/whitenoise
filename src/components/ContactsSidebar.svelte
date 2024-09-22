<script lang="ts">
    import SidebarHeader from "./SidebarHeader.svelte";
    import Contact from "./Contact.svelte";
    import Loader from "./Loader.svelte";
    import type { NUsers, NMetadata } from "../types/nostr";
    import { onMount, onDestroy } from "svelte";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import { invoke } from "@tauri-apps/api/core";
    import { createEventDispatcher } from "svelte";
    import { CheckCircle, Question } from "phosphor-svelte";

    // The pubkey of the currently selected contact
    let { selectedContact = $bindable() }: { selectedContact: string | undefined } = $props();

    let contacts: NUsers = $state({});
    let secureContacts: NUsers = $derived(
        Object.fromEntries(
            Object.entries(contacts).filter(([_pubkey, contactData]) => contactData.nip104)
        )
    );
    let insecureContacts: NUsers = $derived(
        Object.fromEntries(
            Object.entries(contacts).filter(([_pubkey, contactData]) => !contactData.nip104)
        )
    );
    let isLoading = $state(true);
    let searchTerm = $state("");
    let selectedContactMetadata: NMetadata | undefined = $derived(
        selectedContact ? (contacts[selectedContact] as NMetadata) : undefined
    );

    const dispatch = createEventDispatcher();

    let filteredSecureContacts: NUsers = $derived(
        Object.fromEntries(
            Object.entries(secureContacts).filter(([_pubkey, contactData]) => {
                return (
                    (contactData.metadata as NMetadata).name
                        ?.toLowerCase()
                        .includes(searchTerm.toLowerCase()) ||
                    (contactData.metadata as NMetadata).display_name
                        ?.toLowerCase()
                        .includes(searchTerm.toLowerCase()) ||
                    (contactData.metadata as NMetadata).nip05
                        ?.toLowerCase()
                        .includes(searchTerm.toLowerCase())
                );
            })
        )
    );
    let filteredInsecureContacts: NUsers = $derived(
        Object.fromEntries(
            Object.entries(insecureContacts).filter(([_pubkey, contactData]) => {
                return (
                    (contactData.metadata as NMetadata).name
                        ?.toLowerCase()
                        .includes(searchTerm.toLowerCase()) ||
                    (contactData.metadata as NMetadata).display_name
                        ?.toLowerCase()
                        .includes(searchTerm.toLowerCase()) ||
                    (contactData.metadata as NMetadata).nip05
                        ?.toLowerCase()
                        .includes(searchTerm.toLowerCase())
                );
            })
        )
    );

    async function getContacts(): Promise<void> {
        isLoading = true;
        contacts = {};
        selectedContact = undefined;
        try {
            contacts = await invoke("get_contacts");
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

    // Create a link in the empty sidebar to do a NIP-50 and primal cache search
    // This method will handle that
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
    showNewIcon={false}
    showBackIcon={true}
    on:search={handleSearch}
    on:backIconClicked={() => dispatch("backIconClicked")}
></SidebarHeader>
{#if isLoading}
    <div class="w-full h-10 mt-4 flex items-center justify-center">
        <Loader size={40} />
    </div>
{/if}
{#if !isLoading && Object.keys(contacts).length === 0}
    <div class="text-gray-500 w-full p-4 text-center">No contacts found</div>
{/if}

{#if Object.keys(filteredSecureContacts).length > 0}
    <div
        class="text-gray-400 w-full p-2 bg-gray-950 border-b border-b-gray-700 flex flex-row gap-1.5 items-center"
    >
        <CheckCircle size="1.2rem" weight="light" class="text-green-500" />
        Ready to chat securely
    </div>
    {#each Object.entries(filteredSecureContacts) as [pubkey, contactData] (pubkey)}
        <button onclick={() => (selectedContact = pubkey)} class="w-full">
            <Contact
                {pubkey}
                metadata={contactData.metadata as NMetadata}
                active={pubkey === selectedContact}
            />
        </button>
    {/each}
{/if}

{#if Object.keys(filteredInsecureContacts).length > 0}
    <div
        class="text-gray-400 w-full p-2 bg-gray-950 border-b border-b-gray-700 flex flex-row gap-1.5 items-center"
    >
        <Question size="1.2rem" weight="light" class="text-orange-500" />
        Invite to chat securely
    </div>
    {#each Object.entries(filteredInsecureContacts) as [pubkey, contactData] (pubkey)}
        <button onclick={() => (selectedContact = pubkey)} class="w-full">
            <Contact
                {pubkey}
                metadata={contactData.metadata as NMetadata}
                active={pubkey === selectedContact}
            />
        </button>
    {/each}
{/if}
