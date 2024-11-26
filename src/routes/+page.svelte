<script lang="ts">
import { accounts, updateAccountsStore } from "$lib/stores/accounts";
import { goto } from "$app/navigation";
import { onMount } from "svelte";
import { isValidHexPubkey } from "$lib/types/nostr";
import { invoke } from "@tauri-apps/api/core";
import Loader from "$lib/components/Loader.svelte";

onMount(async () => {
    updateAccountsStore().then(() => {
        if ($accounts.activeAccount && isValidHexPubkey($accounts.activeAccount)) {
            invoke("init_nostr_for_current_user");
            setTimeout(() => {
                goto("/chats");
            }, 3000);
        } else {
            goto("/login");
        }
    });
});
</script>

<div class="flex flex-col items-center justify-center w-screen h-screen bg-gray-800">
    <div class="bg-gray-800 w-full md:w-1/2 h-2/3 flex flex-col items-center justify-center gap-6 py-12 px-6">
        <h1 class="text-5xl font-extrabold text-center">White Noise</h1>
        <h2 class="text-3xl font-medium text-center">Secure. Distributed. Uncensorable.</h2>
        <div class="h-[40px]">
            <Loader size={40} fullscreen={false} />
        </div>
    </div>
    <div class="flex flex-row gap-1 items-end mt-20">
        Powered by
        <img src="/nostr.webp" alt="nostr" class="w-20" />
    </div>
</div>
