<script lang="ts">
    import Sidebar from "../../../components/Sidebar.svelte";
    import SidebarHeader from "../../../components/SidebarHeader.svelte";
    import MainPanel from "../../../components/MainPanel.svelte";
    import { UserPlus } from "phosphor-svelte";
    import ndk from "../../../stores/ndk";
    import { type NDKUser } from "@nostr-dev-kit/ndk";
    import { onMount } from "svelte";
    import Contact from "../../../components/Contact.svelte";

    let contacts: Set<NDKUser> | undefined;
    let selectedContact: NDKUser | undefined;
    onMount(async () => {
        contacts = await $ndk.activeUser?.follows();
    });

    // TODO: need to sort by something (last seen or last message or name?)
    // TODO: need to add a search bar
    // TODO: need to add a filter to only show contacts with messages
    // TODO: need to show conversation transcript on click in the main panel
    // TODO: need to make loading way faster
</script>

<Sidebar>
    <SidebarHeader title="Contacts" newIcon={UserPlus} />
    {#if contacts}
        {#each contacts as contact}
            <button on:click={() => (selectedContact = contact)} class="w-full">
                <Contact pubkey={contact.pubkey} active={contact.pubkey === selectedContact?.pubkey} />
            </button>
        {/each}
    {/if}
</Sidebar>
<MainPanel>
    <div class="py-4 px-6">
        <h1 class="text-xl font-semibold mb-6">Contacts coming soon</h1>
    </div>
</MainPanel>
