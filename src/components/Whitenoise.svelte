<script lang="ts">
    import MainPanel from "./MainPanel.svelte";
    import Sidebar from "./Sidebar.svelte";
</script>

<Sidebar>
    <slot name="sidebar" />
</Sidebar>
<MainPanel>
    <slot name="main" />
</MainPanel>
