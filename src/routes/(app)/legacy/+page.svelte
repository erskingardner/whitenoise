<script lang="ts">
    import MainPanel from "../../../components/MainPanel.svelte";
    import Contact from "../../../components/Contact.svelte";
    import Sidebar from "../../../components/Sidebar.svelte";
    import SidebarHeader from "../../../components/SidebarHeader.svelte";
    import {getLegacyChats} from '../../../stores/legacyChats.svelte';

    const legacyChats = getLegacyChats()
</script>

<Sidebar>
    <SidebarHeader title="Legacy Chats" />
    {#each legacyChats.conversations as pubkey}
      <Contact {pubkey} />
    {/each}
</Sidebar>
<MainPanel>
    <div class="py-4 px-6">
        <h1 class="text-xl font-semibold mb-6">Legacy chats coming soon</h1>
    </div>
</MainPanel>
