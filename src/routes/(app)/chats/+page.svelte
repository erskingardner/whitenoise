<script lang="ts">
    import RespondPanel from "../../../components/RespondPanel.svelte";
    import Sidebar from "../../../components/Sidebar.svelte";
    import SidebarHeader from "../../../components/SidebarHeader.svelte";
    import MainPanel from "../../../components/MainPanel.svelte";
    import SidebarContent from "../../../components/SidebarContent.svelte";

    // TODO: need to show conversation transcript on click in the main panel
    // TODO: need to load actual conversations
</script>

<Sidebar>
    <SidebarHeader title="Secure Chats" />
    <SidebarContent />
</Sidebar>
<MainPanel>
    <RespondPanel />
</MainPanel>
