<script lang="ts">
    import { onMount } from "svelte";
    import ndk from "../stores/ndk";
    import type { NDKUser, NDKUserProfile } from "@nostr-dev-kit/ndk";

    export let pubkey: string;

    let user: NDKUser;
    let profile: NDKUserProfile | null = null;
    user = $ndk.getUser({ pubkey });

    onMount(async () => {
        profile = await user.fetchProfile();
    });
</script>

<div class="text-lg font-semibold">
    {#if profile && (profile?.displayName || profile?.name)}
        {profile.displayName || profile.name}
    {:else}
        {`${user.npub.slice(0, 20)}...`}
    {/if}
</div>
