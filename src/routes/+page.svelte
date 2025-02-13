<script lang="ts">
import { goto } from "$app/navigation";
import Loader from "$lib/components/Loader.svelte";
import { activeAccount, updateAccountsStore } from "$lib/stores/accounts";
import { isValidHexPubkey } from "$lib/types/nostr";
import { invoke } from "@tauri-apps/api/core";
import { onMount } from "svelte";

onMount(async () => {
    updateAccountsStore().then(async () => {
        if ($activeAccount?.pubkey && isValidHexPubkey($activeAccount?.pubkey)) {
            await invoke("init_nostr_for_current_user");
            console.log("Initialized Nostr for current user");
            setTimeout(() => {
                goto("/chats");
            }, 2000);
        } else {
            goto("/login");
        }
    });
});
</script>

<div class="flex flex-col items-center justify-center w-screen h-dvh bg-gray-800">
    <div class="bg-gray-800 w-full md:w-1/2 h-2/3 flex flex-col items-center justify-center gap-6 py-12 px-6">
        <img src="whitenoise-login-logo2.png" alt="logo" class="w-32 lg:w-40" />
        <h2 class="text-xl lg:text-2xl font-medium text-center">Secure. Distributed. Uncensorable.</h2>
        <div class="h-[40px]">
            <Loader size={40} fullscreen={false} />
        </div>
    </div>
</div>
