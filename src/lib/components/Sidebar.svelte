<script lang="ts">
    import { ChatsCircle, Eyes, Phone, Gear, Warning } from "phosphor-svelte";
    import Avatar from "./Avatar.svelte";
    import { accounts, setActiveAccount } from "$lib/stores/accounts";
    import { goto } from "$app/navigation";

    let { activeTab } = $props();

    function handleAccountChange(pubkey: string) {
        goto(`/chats`);
        setTimeout(() => {
            setActiveAccount(pubkey);
        }, 100);
    }
</script>

<div class="hidden md:flex">
    <div
        class="sticky top-0 left-0 bottom-0 border-r border-gray-700 bg-gray-800 h-screen p-2 gap-4 flex flex-col justify-between"
    >
        <div class="flex flex-col gap-4 items-center">
            <a href="/chats" class="p-2">
                <span class="sidebar-link {activeTab === 'chats' ? 'active' : ''}">
                    <ChatsCircle size={32} weight={activeTab === "chats" ? "fill" : "light"} />
                </span>
            </a>
            <a href="/legacy" class="p-2">
                <span class="sidebar-link relative {activeTab === 'legacy' ? 'active' : ''}">
                    <Eyes size={30} weight={activeTab === "legacy" ? "fill" : "light"} />
                    <Warning size={18} weight="fill" class="absolute -bottom-2 -right-2 text-red-500" />
                </span>
            </a>
            <a href="/calls" class="p-2">
                <span class="sidebar-link {activeTab === 'calls' ? 'active' : ''}">
                    <Phone size={32} weight={activeTab === "calls" ? "fill" : "light"} />
                </span>
            </a>
        </div>
        <div class="flex flex-col gap-4 justify-between items-center mt-auto mb-0">
            {#each $accounts.accounts as account}
                <button onclick={() => handleAccountChange(account.pubkey)}>
                    <Avatar
                        pubkey={account.pubkey}
                        picture={account.metadata.picture}
                        pxSize={32}
                        showRing={account.pubkey === $accounts.activeAccount}
                    />
                </button>
            {/each}
            <a href="/settings" class="p-2">
                <span class="sidebar-link {activeTab === 'settings' ? 'active' : ''}">
                    <Gear size={32} weight={activeTab === "settings" ? "fill" : "light"} />
                </span>
            </a>
        </div>
    </div>
</div>

<style lang="postcss">
    .sidebar-link {
        @apply flex flex-row items-center font-medium text-lg gap-2;
    }
    .sidebar-link.active {
        @apply text-blue-500;
    }
</style>
