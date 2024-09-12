<script lang="ts">
    import Sidebar from "../../../components/Sidebar.svelte";
    import SidebarHeader from "../../../components/SidebarHeader.svelte";
    import MainPanel from "../../../components/MainPanel.svelte";
    import { UserPlus } from "phosphor-svelte";
    import Contact from "../../../components/Contact.svelte";
    import Loader from "../../../components/Loader.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { type NContact } from "../../../types/nostr";
    import { onMount, onDestroy } from "svelte";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";

    let selectedContact: NContact | undefined = $state(undefined);
    let contacts: NContact[] = $state([]);
    let isLoading = $state(true);

    async function getContacts(): Promise<void> {
        isLoading = true;
        contacts = [];
        selectedContact = undefined;
        try {
            const fetchedContacts = await invoke("get_contacts");
            contacts = fetchedContacts as NContact[];
        } catch (error) {
            console.error("Error fetching contacts:", error);
        } finally {
            isLoading = false;
        }
    }

    let unlisten: UnlistenFn;

    onMount(async () => {
        getContacts();
        unlisten = await listen<string>("identity_change", (event) => {
            console.log("identity_change on contacts", event);
            getContacts();
        });
    });

    onDestroy(() => {
        unlisten();
    });

    // TODO: need to sort by something (last seen or last message or name?)
    // TODO: need to add a search bar
    // TODO: need to add a filter to only show contacts with messages
    // TODO: need to show conversation transcript on click in the main panel
</script>

<Sidebar>
    <SidebarHeader title="Contacts" newIcon={UserPlus} />
    {#if isLoading}
        <div class="w-full h-10 mt-4 flex items-center justify-center">
            <Loader size={40} />
        </div>
    {/if}
    {#if !isLoading && contacts.length === 0}
        <div class="text-gray-500 w-full p-4 text-center">No contacts found</div>
    {/if}
    {#each contacts as contact (contact.public_key)}
        <button onclick={() => (selectedContact = contact)} class="w-full">
            <Contact
                pubkey={contact.public_key}
                active={contact.public_key === selectedContact?.public_key}
            />
        </button>
    {/each}
</Sidebar>
<MainPanel>
    <div class="py-4 px-6">
        <h1 class="text-xl font-semibold mb-6">Contacts coming soon</h1>
    </div>
</MainPanel>
