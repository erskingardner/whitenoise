<script lang="ts">
import { goto } from "$app/navigation";
import Loader from "$lib/components/Loader.svelte";
import { accounts, activeAccount, updateAccountsStore } from "$lib/stores/accounts";
import { isValidHexPubkey } from "$lib/types/nostr";
import { invoke } from "@tauri-apps/api/core";
import { onMount } from "svelte";

onMount(async () => {
    updateAccountsStore().then(() => {
        console.log("outer page: accounts", $accounts);
        console.log("outer page: activeAccount", $activeAccount);
        if ($activeAccount?.pubkey && isValidHexPubkey($activeAccount?.pubkey)) {
            console.log("init_nostr_for_current_user");
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

<div class="flex flex-col items-center justify-center w-screen h-dvh bg-gray-800">
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
