<script lang="ts">
    import { accounts, updateAccountsStore } from "$lib/stores/accounts";
    import { goto } from "$app/navigation";
    import { onMount } from "svelte";
    import { isValidHexPubkey } from "$lib/types/nostr";
    import { invoke } from "@tauri-apps/api/core";

    onMount(async () => {
        updateAccountsStore().then(() => {
            if ($accounts.activeAccount && isValidHexPubkey($accounts.activeAccount)) {
                invoke("init_nostr_for_current_user");
                goto("/chats");
            } else {
                goto("/login");
            }
        });
    });
</script>
