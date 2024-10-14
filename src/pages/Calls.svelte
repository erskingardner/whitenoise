<script lang="ts">
    import { Page, Navbar, Link, f7 } from "framework7-svelte";
</script>

<Page
    class="calls-page bg-gray-900"
    on:pageInit={() => {
        console.log("pageInit: Calls");
    }}
    on:pageTabShow={async () => {
        console.log("pageTabShow: Calls");
    }}
    on:pageBeforeRemove={() => {
        console.log("pageBeforeRemove: Calls");
    }}
>
    <Navbar title="Calls" large transparent>
        <Link
            id="menu-button"
            slot="left"
            iconF7="bars"
            on:click={() => f7.panel.toggle("#menu-panel-left")}
            style={!f7.device.desktop ? "display: none;" : ""}
        />
    </Navbar>
    <div
        class="calls-page-content flex flex-col justify-start items-center h-screen text-gray-500 mt-64"
    >
        Two Weeks™
    </div>
</Page>
