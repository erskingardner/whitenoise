<script lang="ts">
    import Sidebar from "../../../components/Sidebar.svelte";
    import SidebarHeader from "../../../components/SidebarHeader.svelte";
    import MainPanel from "../../../components/MainPanel.svelte";
    import { PhonePlus } from "phosphor-svelte";
</script>

<Sidebar>
    <SidebarHeader title="Calls" newIcon={PhonePlus} />
</Sidebar>
<MainPanel>
    <div class="py-4 px-6">
        <h1 class="text-xl font-semibold mb-6">Calls coming soon</h1>
    </div>
</MainPanel>
