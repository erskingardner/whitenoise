<script lang="ts">
import type { PushView } from "$lib/types/modal";
import PublishInboxRelays from "./PublishInboxRelays.svelte";

let {
    pushView,
    inboxRelaysPublished = $bindable(),
    keyPackageRelaysPublished = $bindable(),
    keyPackagePublished = $bindable(),
} = $props<{
    pushView?: PushView;
    inboxRelaysPublished?: boolean;
    keyPackageRelaysPublished?: boolean;
    keyPackagePublished?: boolean;
}>();

function startOnboarding(): void {
    pushView(PublishInboxRelays, {
        inboxRelaysPublished,
        keyPackageRelaysPublished,
        keyPackagePublished,
    });
}
</script>

<div class="flex flex-col gap-4 justify-start items-center w-full md:w-2/3 lg:w-1/2 mx-auto h-full">
    <div class="flex flex-col gap-4 justify-start items-center">
        <div class="text-7xl">💬</div>
        <h1 class="text-2xl font-bold text-white">Welcome to White Noise</h1>
        <h2 class="text-xl font-medium text-gray-300">The most secure chat app on Nostr</h2>
        <p class=" text-gray-400">
            Under the hood, we use the Messaging Layer Security protocol (MLS) to ensure end-to-end encryption of your
            messages; and with no servers to compromise, you can rest assured that your messages are private and secure.
        </p>
        <div class="text-gray-400">
            Let's get you set up so that you can start messaging with friends. This will take less than one minute.
        </div>
        <button class="button-primary" onclick={startOnboarding}> 🤝 Let's get started </button>
    </div>
</div>
