<script lang="ts">
    import { onMount } from "svelte";
    import ndk from "../../../stores/ndk";
    import { currentIdentity } from "../../../stores/identities";
    import type { NDKEvent } from "@nostr-dev-kit/ndk";
    import MainPanel from "../../../components/MainPanel.svelte";
    import Sidebar from "../../../components/Sidebar.svelte";
    import SidebarHeader from "../../../components/SidebarHeader.svelte";
    import { page } from "$app/stores";
    import { getVersion } from "@tauri-apps/api/app";

    let version: string;
    
    let profile: NDKEvent | null = null;
    onMount(async () => {
        version = await getVersion();
        profile = await $ndk.fetchEvent({ kinds: [0], authors: [$currentIdentity] });
    });
</script>

<Sidebar>
    <SidebarHeader title="Settings" showNewIcon={false} showSearch={false} />
    <div class="settings-links">
        <a
            class={$page.url.pathname === "/settings/profile" ? "bg-gray-800" : ""}
            href="/settings/profile">Profile</a
        >
        <a
            class={$page.url.pathname === "/settings/privacy" ? "bg-gray-800" : ""}
            href="/settings/privacy">Privacy & Security</a
        >
        <a
            class={$page.url.pathname === "/settings/developer" ? "bg-gray-800" : ""}
            href="/settings/developer">Developer</a
        >
        <div class="fixed bottom-0 text-sm self-center mb-6 font-extralight font-mono">v{version} - RIP Telegram</div>
    </div>
</Sidebar>
<MainPanel>
    <div class="py-4 px-6">
        <slot />
    </div>
</MainPanel>

<style lang="postcss">
    .settings-links {
        @apply flex flex-col items-start;
    }

    .settings-links a {
        @apply py-2 px-4 hover:bg-gray-800 w-full;
    }
</style>
