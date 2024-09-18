<script lang="ts">
    import "../app.pcss";
    import { currentIdentity, identities, fetchAccounts } from "../stores/accounts";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import { onDestroy, onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";

    let unlisten: UnlistenFn;

    onMount(async () => {
        updateIdentities();
        unlisten = await listen<string>("identity_change", (_event) => updateIdentities());
    });

    onDestroy(() => {
        unlisten();
    });

    async function updateIdentities() {
        await fetchAccounts();
        if (
            Object.keys($identities).length > 0 &&
            $currentIdentity &&
            ($page.url.pathname === "/" || $page.url.pathname === "/login")
        ) {
            goto("/chats");
        } else if (
            Object.keys($identities).length === 0 ||
            (!$currentIdentity && $page.url.pathname !== "/login")
        ) {
            goto("/login");
        }
    }
</script>

<slot />
