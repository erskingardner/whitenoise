<script lang="ts">
    import RespondPanel from "../../../components/RespondPanel.svelte";
    import Sidebar from "../../../components/Sidebar.svelte";
    import SidebarHeader from "../../../components/SidebarHeader.svelte";
    import MainPanel from "../../../components/MainPanel.svelte";

    // TODO: need to show conversation transcript on click in the main panel
    // TODO: need to load actual conversations
</script>

<Sidebar>
    <SidebarHeader title="Secure Chats" showNewIcon={false} />
</Sidebar>
<MainPanel>
    <div class="flex flex-col justify-center grow items-center gap-4 text-gray-500">
        Two weeks™
    </div>
    <!-- <RespondPanel /> -->
</MainPanel>
