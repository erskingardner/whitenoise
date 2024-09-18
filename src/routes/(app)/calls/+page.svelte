<script lang="ts">
    import Sidebar from "../../../components/Sidebar.svelte";
    import SidebarHeader from "../../../components/SidebarHeader.svelte";
    import MainPanel from "../../../components/MainPanel.svelte";
    import { PhonePlus } from "phosphor-svelte";
</script>

<Sidebar>
    <SidebarHeader title="Calls" newIcon={PhonePlus} showNewIcon={false} />
</Sidebar>
<MainPanel>
    <div class="flex flex-col justify-center grow items-center gap-4 text-gray-500">
        Two weeks™
    </div>
</MainPanel>
