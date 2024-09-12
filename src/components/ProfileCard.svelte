<script lang="ts">
    import { CheckFat, Copy, SignOut } from "phosphor-svelte";
    import ndk from "../stores/ndk";
    import { type NDKUser } from "@nostr-dev-kit/ndk";
    import { switchIdentity, logout } from "../stores/identities";
    import Avatar from "./Avatar.svelte";
    import Name from "./Name.svelte";

    export let pubkey: string;
    let copied = false;
    let user: NDKUser = $ndk.getUser({ pubkey });

    function copyNpub() {
        copied = true;
        setTimeout(() => {
            copied = false;
        }, 1000);
        navigator.clipboard.writeText(user.npub);
    }
</script>

<div class="rounded-lg bg-gray-800 flex flex-row items-center justify-between p-4">
    <div class="flex flex-row gap-6 items-center">
        <button onclick={() => switchIdentity(pubkey)}>
            <Avatar {pubkey} pxSize={40} showRing={true} />
        </button>
        <div class="flex flex-col gap-0">
            <Name {pubkey} />
            <span class="text-sm text-gray-400 flex flex-row gap-2 items-center">
                {user.npub}
                {#if copied}
                    <CheckFat size="1.5rem" weight="thin" class="text-green-500 cursor-pointer" />
                    <!-- content here -->
                {:else}
                    <Copy
                        onclick={copyNpub}
                        size="1.5rem"
                        weight="thin"
                        class="hover:text-gray-300 cursor-pointer"
                    />
                {/if}
            </span>
        </div>
    </div>
    <button
        onclick={() => logout(pubkey)}
        title="Logout from this account"
        class="flex flex-row gap-2 items-center px-3 py-2 rounded-lg bg-gray-700 hover:bg-gray-600 ring-1 ring-gray-500"
    >
        <SignOut size="2rem" weight="thin" />
        Sign out
    </button>
</div>
