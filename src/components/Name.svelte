<script lang="ts">
    import { onMount } from "svelte";
    import ndk from "../stores/ndk";
    import type { NDKUser, NDKUserProfile } from "@nostr-dev-kit/ndk";

    interface Props {
        pubkey: string;
    }

    let { pubkey }: Props = $props();

    let user: NDKUser | null = $state(null);
    let profile: NDKUserProfile | null = $state(null);
    
    onMount(async () => {
        user = $ndk.getUser({ pubkey });
        profile = await user.fetchProfile();
    });
</script>

<div class="text-lg font-semibold">
    {#if user && profile && (profile?.displayName || profile?.name)}
        {profile.displayName || profile.name}
    {:else}
        {`${user?.npub.slice(0, 20)}...`}
    {/if}
</div>
