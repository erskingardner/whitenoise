<script lang="ts">
    import { accounts } from "$lib/stores/accounts";
    import ndk from "$lib/stores/ndk";
    import type { NDKUser, NDKUserProfile } from "@nostr-dev-kit/ndk";

    interface Props {
        picture?: string;
        pubkey: string;
        pxSize?: number;
        showRing?: boolean;
    }
    let { pubkey, picture, pxSize = 32, showRing = false }: Props = $props();

    let user: NDKUser = $ndk.getUser({ pubkey });
    let profile: NDKUserProfile | undefined = $state(undefined);

    $effect(() => {
        user.fetchProfile().then((userProfile: NDKUserProfile) => {
            profile = userProfile;
        });
    });

    let image = $derived(profile?.picture || picture || undefined);
</script>

<div
    class="flex flex-col items-center justify-center rounded-full bg-gray-900 {$accounts.activeAccount === pubkey &&
    showRing
        ? 'ring-4 ring-blue-600 ring-offset-2 ring-offset-gray-900'
        : ''}"
    style="width: {pxSize}px; height: {pxSize}px; min-width: {pxSize}px; min-height: {pxSize}px;"
>
    {#if image}
        <img src={image} alt="avatar" class="shrink-0 w-full h-full rounded-full object-cover" />
    {:else}
        <div
            class="w-full h-full rounded-full font-semibold text-xl font-mono shrink-0 flex flex-col justify-center text-center"
            style="background-color: #{pubkey.slice(0, 6)};"
        >
            {pubkey.slice(0, 2)}
        </div>
    {/if}
</div>
