<script lang="ts">
    import { PhoneCall, Gear, Users, ChatsCircle, ShieldWarning } from "phosphor-svelte";
    import { page } from "$app/stores";
    import { identities, switchIdentity } from "../stores/identities";
    import Avatar from "./Avatar.svelte";
</script>

<div
    class="sidebar-menu border-r border-r-gray-700 bg-gray-900 relative w-[77px] h-screen overflow-y-scroll shrink-0"
>
    <div
        class="flex flex-col mx-auto items-center align-center gap-4 p-3 top-0 left-0 bottom-0 h-screen fixed"
    >
        <a
            href="/contacts"
            class="p-4 hover:bg-gray-800 rounded-lg {$page.url.pathname === '/contacts'
                ? 'bg-gray-800'
                : ''}"
        >
            <Users size="2rem" weight="thin" />
        </a>
        <a
            href="/chats"
            class="p-4 hover:bg-gray-800 rounded-lg {$page.url.pathname === '/chats'
                    ? 'bg-gray-800'
                    : ''}"
            >
            <ChatsCircle size="2rem" weight="thin" />
        </a>
        <a
            href="/legacy"
            class="p-4 hover:bg-gray-800 rounded-lg relative {$page.url.pathname === '/legacy'
                ? 'bg-gray-800'
                : ''}"
        >
            <ChatsCircle size="2rem" weight="thin" />
            <ShieldWarning size="1.5rem" weight="bold" class="text-red-500 absolute top-2 right-2" />
        </a>
        <a
            href="/calls"
            class="p-4 hover:bg-gray-800 rounded-lg {$page.url.pathname === '/calls'
                ? 'bg-gray-800'
                : ''}"
        >
            <PhoneCall size="2rem" weight="thin" />
        </a>
        <div class="mt-auto align-middle flex flex-col gap-6 items-center">
            {#each $identities as identity}
                <button onclick={() => switchIdentity(identity.pubkey)}>
                    <Avatar pubkey={identity.pubkey} pxSize={32} showRing={true} />
                </button>
            {/each}

            <a href="/settings/profile" class="p-4">
                <Gear size="2rem" weight="thin" />
            </a>
        </div>
    </div>
</div>
