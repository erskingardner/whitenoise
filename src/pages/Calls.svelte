<script lang="ts">
    import { Page, Navbar, Link, f7 } from "framework7-svelte";
</script>

<Page class="calls-page bg-gray-900">
    <Navbar title="Calls" large transparent>
        <Link slot="left" iconF7="bars" onClick={() => f7.panel.toggle("#menu-panel-left")} />
    </Navbar>
    <div
        class="calls-page-content flex flex-col justify-center items-center h-screen text-gray-500"
    >
        Two Weeks™
    </div>
</Page>
