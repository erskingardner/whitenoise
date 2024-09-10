<script lang="ts">
    import Sidebar from "../../../components/Sidebar.svelte";
    import SidebarHeader from "../../../components/SidebarHeader.svelte";
    import MainPanel from "../../../components/MainPanel.svelte";
    import { UserPlus } from "phosphor-svelte";
    import ndk from "../../../stores/ndk";
    import { type NDKUser } from "@nostr-dev-kit/ndk";
    import Contact from "../../../components/Contact.svelte";
    import Loader from "../../../components/Loader.svelte";

    let selectedContact: NDKUser | undefined;

    async function getContacts() {
        const contacts = await $ndk.activeUser?.follows();
        return contacts ? contacts : new Set<NDKUser>();
    }

    // TODO: need to sort by something (last seen or last message or name?)
    // TODO: need to add a search bar
    // TODO: need to add a filter to only show contacts with messages
    // TODO: need to show conversation transcript on click in the main panel
</script>

<Sidebar>
    <SidebarHeader title="Contacts" newIcon={UserPlus} />
    {#await getContacts()}
        <div class="w-full h-10 mt-4 flex items-center justify-center">
            <Loader size={40}/>
        </div>
    {:then contacts}
        {#each contacts as contact (contact.pubkey)}
            <button on:click={() => (selectedContact = contact)} class="w-full">
                <Contact
                    pubkey={contact.pubkey}
                    active={contact.pubkey === selectedContact?.pubkey}
                />
            </button>
        {:else}
            <div class="text-gray-500 w-full p-4 text-center">No contacts found</div>
        {/each}
    {:catch error}
        <div class="text-red-500 w-full p-4 text-center">Error loading contacts</div>
    {/await}
</Sidebar>
<MainPanel>
    <div class="py-4 px-6">
        <h1 class="text-xl font-semibold mb-6">Contacts coming soon</h1>
    </div>
</MainPanel>
