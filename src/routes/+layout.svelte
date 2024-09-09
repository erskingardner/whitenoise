<script lang="ts">
    import "../app.pcss";
    import { currentIdentity, identities, type Identity } from "../stores/identities";
    import { listen, type UnlistenFn } from '@tauri-apps/api/event';
    import { onDestroy, onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";

    let unlisten: UnlistenFn;

    onMount(async () => {
        updateIdentities();
        unlisten = await listen<string>('identity_change', (event) => {
            updateIdentities();
        });
    });

    onDestroy(() => {
        unlisten();
    });

    async function updateIdentities() {
        const ids: string[] = await invoke("get_identities");
        identities.set(ids ? ids.map((id: string) => ({ pubkey: id }) as Identity) : []);
        currentIdentity.set(await invoke("get_current_identity"));
        if ($identities.length > 0 && $currentIdentity && ($page.url.pathname === "/" || $page.url.pathname === "/login")) {
            goto("/chats");
        } else if ($identities.length === 0 || !$currentIdentity && $page.url.pathname !== "/login") {
            goto("/login");
        }
    }
</script>

<slot />