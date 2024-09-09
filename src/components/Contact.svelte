<script lang="ts">
    import type { NDKUser } from "@nostr-dev-kit/ndk";
    import { CheckCircle } from "phosphor-svelte";
    import Avatar from "./Avatar.svelte";
    import Name from "./Name.svelte";

    export let pubkey: string;
    export let active: boolean = false;

    let user: NDKUser;
</script>

<div
    class="chat-item flex flex-row gap-4 items-center justify-start px-4 py-2  border-b border-gray-700 hover:bg-gray-800 {active
        ? 'bg-gray-800'
        : ''}"
>
    <Avatar pubkey={pubkey} pxSize={36} />
    <div class="grow flex flex-col justify-start items-start">
        <div class="font-medium">
            <Name pubkey={pubkey} />
        </div>
        <div class="font-extralight line-clamp-2">
            TODO: last seen <!-- TODO: last seen -->
        </div>
    </div>
    <div class="metadata flex flex-col gap-1 items-center justify-center ml-auto">
        <div class="timestamp">Fri</div> <!-- TODO: last message timestamp -->
        <div class="read-state">
            <CheckCircle size="1rem" weight="thin" />
        </div>
    </div>
</div>