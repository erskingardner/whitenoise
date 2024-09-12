<script lang="ts">
    import { onMount } from "svelte";
    import ndk from "../stores/ndk";
    import type { NDKUser, NDKUserProfile } from "@nostr-dev-kit/ndk";
    import { currentIdentity } from "../stores/identities";

    interface Props {
        pubkey: string;
        pxSize?: number;
        showRing?: boolean;
    }

    let { pubkey, pxSize = 32, showRing = false }: Props = $props();
    let user: NDKUser | null = $derived($ndk.getUser({ pubkey }));
    let profile: Promise<NDKUserProfile | null> | null = $derived(user?.fetchProfile());

    $inspect("profile", profile);
    $inspect("user", user);
</script>

<div class="flex flex-col items-center justify-center">
    {#if profile !== null}
        {#await profile then profile}
            {#if profile?.image}
                <img
                    src={user?.profile?.image}
                    alt="avatar"
                    class="shrink-0 avatar rounded-full bg-cover {$currentIdentity === pubkey &&
                    showRing
                        ? 'ring-4 ring-blue-600 ring-offset-2 ring-offset-gray-900'
                        : ''}"
                    style="width: {pxSize}px; height: {pxSize}px; min-width: {pxSize}px; min-height: {pxSize}px;"
                />
            {:else}
                <div
                    class="rounded-full font-semibold text-xl font-mono shrink-0 flex flex-col justify-center text-center {$currentIdentity ===
                        pubkey && showRing
                        ? 'ring-4 ring-blue-600 ring-offset-2 ring-offset-gray-900'
                        : ''}"
                    style="background-color: #{pubkey.slice(
                        0,
                        6
                    )}; width: {pxSize}px; height: {pxSize}px; min-width: {pxSize}px; min-height: {pxSize}px;"
                >
                    {pubkey.slice(0, 2)}
                </div>
            {/if}
        {/await}
    {/if}
</div>
