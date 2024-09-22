<script lang="ts">
    import { CheckFat, Copy, SignOut } from "phosphor-svelte";
    import { npubFromPubkey } from "../utils/nostr";
    import { switchIdentity, logout } from "../stores/accounts";
    import Avatar from "./Avatar.svelte";
    import Name from "./Name.svelte";
    import type { NMetadata } from "../types/nostr";

    type Props = {
        pubkey: string;
        metadata: NMetadata;
    };

    let { pubkey, metadata }: Props = $props();
    let copied = $state(false);
    let npub = $derived(npubFromPubkey(pubkey));

    function copyNpub() {
        copied = true;
        setTimeout(() => {
            copied = false;
        }, 1000);
        navigator.clipboard.writeText(npub);
    }
</script>

<div class="rounded-lg bg-gray-800 flex flex-row gap-6 items-center justify-between p-4">
    <div class="flex flex-row gap-6 items-center shrink overflow-hidden p-2">
        <button onclick={() => switchIdentity(pubkey)} class="overflow-visible">
            <Avatar picture={metadata?.picture} {pubkey} pxSize={40} showRing={true} />
        </button>
        <div class="flex flex-col gap-0 min-w-0">
            <Name {pubkey} {metadata} />
            <span class="font-mono text-gray-400 flex flex-row gap-2 items-center truncate">
                <span class="shrink overflow-hidden truncate font-mono">{npub}</span>
                {#if copied}
                    <CheckFat
                        size="1.5rem"
                        weight="thin"
                        class="text-green-500 cursor-pointer flex-shrink-0"
                    />
                {:else}
                    <Copy
                        onclick={copyNpub}
                        size="1.5rem"
                        weight="thin"
                        class="hover:text-gray-300 cursor-pointer flex-shrink-0"
                    />
                {/if}
            </span>
        </div>
    </div>
    <button
        onclick={() => logout(pubkey)}
        title="Logout from this account"
        class="flex flex-row gap-2 items-center px-3 py-2 rounded-lg bg-gray-700 hover:bg-gray-600 ring-1 ring-gray-500 text-nowrap"
    >
        <SignOut size="2rem" weight="thin" />
        Sign out
    </button>
</div>
