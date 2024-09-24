<script lang="ts">
    import { PhoneCall, Gear, ChatCircle } from "phosphor-svelte";
    import { identities, switchIdentity, currentIdentity } from "../stores/accounts";
    import Avatar from "./Avatar.svelte";
    import type { NMetadata } from "../types/nostr";

    type props = {
        f7route: any;
    };

    let { f7route }: props = $props();

    let picture = $derived(
        $identities !== undefined &&
            $currentIdentity !== undefined &&
            Object.keys($identities).length > 0
            ? ($identities[$currentIdentity] as NMetadata)?.picture
            : undefined
    );
</script>

<div
    class="hidden md:block sidebar-menu border-r border-r-gray-700 bg-gray-900 relative w-[77px] h-screen overflow-y-scroll shrink-0"
>
    <div
        class="flex flex-col mx-auto items-center align-center gap-4 p-3 top-0 left-0 bottom-0 h-screen fixed"
    >
        <a
            href="/chats"
            class="p-4 hover:bg-gray-800 rounded-lg {f7route.path === '/chats'
                ? 'bg-gray-800'
                : ''}"
        >
            <ChatCircle size="2rem" weight="thin" />
        </a>
        <a
            href="/calls"
            class="p-4 hover:bg-gray-800 rounded-lg {f7route.path === '/calls'
                ? 'bg-gray-800'
                : ''}"
        >
            <PhoneCall size="2rem" weight="thin" />
        </a>
        <div class="mt-auto align-middle flex flex-col gap-6 items-center">
            {#each Object.entries($identities) as [pubkey, metadata]}
                <button onclick={() => switchIdentity(pubkey)}>
                    <Avatar
                        {pubkey}
                        picture={(metadata as NMetadata).picture}
                        pxSize={32}
                        showRing={true}
                    />
                </button>
            {/each}

            <a href="/settings/profile" class="p-4">
                <Gear size="2rem" weight="thin" />
            </a>
        </div>
    </div>
</div>

<div
    class="mobile-menu fixed bottom-0 w-full py-2 px-6 bg-gray-900 border-t border-t-gray-700 md:hidden flex flex-row justify-between items-center z-50"
>
    <a href="/chats" class="p-4 flex flex-col gap-1 items-center">
        <ChatCircle size="2.75rem" weight={f7route.path === "/chats" ? "fill" : "thin"} />
        Chats
    </a>
    <a href="/calls" class="p-4 flex flex-col gap-1 items-center">
        <PhoneCall size="2.75rem" weight={f7route.path === "/calls" ? "fill" : "thin"} />
        Calls
    </a>
    <button onclick={() => console.log("Change!")} class="p-4 flex flex-col gap-1 items-center">
        <Avatar pubkey={$currentIdentity} {picture} pxSize={40} showRing={false} />
        Profile
    </button>
    <a href="/settings/profile" class="p-4 flex flex-col gap-1 items-center">
        <Gear size="2.75rem" weight={f7route.path.match(/\/settings\/profile/) ? "fill" : "thin"} />
        Settings
    </a>
</div>
