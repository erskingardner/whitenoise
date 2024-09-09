<script lang="ts">
    import { onMount } from "svelte";
    import ndk from "../stores/ndk";
    import type { NDKUser, NDKUserProfile } from "@nostr-dev-kit/ndk";
    import { currentIdentity } from "../stores/identities";

    export let pubkey: string;
    export let pxSize: number = 32;

    let user: NDKUser;
    let profile: NDKUserProfile | null = null;
    
    onMount(async () => {
        user = $ndk.getUser({ pubkey });
        profile = await user.fetchProfile();
    });
</script>

<div class="flex flex-col items-center justify-center">
    {#if profile && profile?.image}
        <img src={profile.image} alt="avatar" class="avatar rounded-full bg-cover {$currentIdentity === pubkey
            ? 'ring-4 ring-blue-600 ring-offset-2 ring-offset-gray-900'
            : ''}" style="width: {pxSize}px; height: {pxSize}px;" />
    {:else}
        <div class="rounded-full font-semibold text-xl font-mono flex flex-col justify-center text-center {$currentIdentity === pubkey
            ? 'ring-4 ring-blue-600 ring-offset-2 ring-offset-gray-900'
            : ''}"
        style="background-color: #{pubkey.slice(0, 6)}; width: {pxSize}px; height: {pxSize}px;">
            {pubkey.slice(0, 2)}
        </div>
    {/if}
</div>