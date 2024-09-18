<script lang="ts">
    import Sidebar from "../../../components/Sidebar.svelte";
    import SidebarHeader from "../../../components/SidebarHeader.svelte";
    import MainPanel from "../../../components/MainPanel.svelte";
    import { UserPlus, MagnifyingGlass } from "phosphor-svelte";
    import Contact from "../../../components/Contact.svelte";
    import Loader from "../../../components/Loader.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { type NUsers, type NMetadata } from "../../../types/nostr";
    import { onMount, onDestroy } from "svelte";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import SidebarModal from "../../../components/SidebarModal.svelte";
    import ChatHeader from "../../../components/ChatHeader.svelte";
    import ndk from "../../../stores/ndk";
    import { type NDKUser } from "@nostr-dev-kit/ndk";

    // The pubkey of the currently selected contact
    let selectedContact: string | undefined = $state(undefined);
    let contacts: NUsers = $state({});
    let isLoading = $state(true);
    let searchTerm = $state("");
    let contactSearch = $state("");
    let selectedContactMetadata: NMetadata | undefined = $derived(
        selectedContact ? (contacts[selectedContact] as NMetadata) : undefined
    );

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

    function openContactModal() {
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

<Sidebar>
    <SidebarHeader
        title="Contacts"
        newIcon={UserPlus}
        on:search={handleSearch}
        on:newIconClicked={openContactModal}
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
            <Contact
                {pubkey}
                metadata={metadata as NMetadata}
                active={pubkey === selectedContact}
            />
        </button>
    {/each}
</Sidebar>
<MainPanel>
    {#if selectedContact === undefined}
        <div class="flex items-center justify-center w-full text-gray-500 h-screen grow">
            Select a contact
        </div>
    {:else}
        <ChatHeader pubkey={selectedContact} metadata={selectedContactMetadata} />
        <div class="flex flex-col gap-10 py-10">
            <img
                src={selectedContactMetadata?.banner}
                alt="Banner"
                class="object-cover bg-cover w-full h-48 absolute -mt-10 {selectedContactMetadata?.banner
                    ? 'opacity-30'
                    : 'opacity-0'}"
            />
            <div class="flex flex-col gap-2 px-6">
                <h3 class="text-xl">{selectedContactMetadata?.about}</h3>
                <!-- TODO: Implement this once we have a way to check if a nip05 is valid -->
                <!-- <p class="flex flex-row gap-2 items-center">
                    <SealCheck size="1.5rem" weight="thin" />
                    {selectedContactMetadata?.nip05}
                </p> -->
            </div>
        </div>
    {/if}
</MainPanel>

<SidebarModal {modalVisible} title="Add a new contact">
    <div class="flex flex-row items-center relative">
        <input
            class="ring-1 ring-gray-700 w-full p-2 rounded-md ring-1 ring-gray-700 bg-transparent w-full flex grow"
            type="text"
            bind:value={contactSearch}
            placeholder="Search for a contact&hellip;"
            onkeyup={submitContactsSearch}
        />
        <button class="absolute right-3" onclick={submitContactsSearch}>
            <MagnifyingGlass size="1.5rem" weight="thin" />
        </button>
    </div>
</SidebarModal>
