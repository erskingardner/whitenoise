<script lang="ts">
    import { onMount } from "svelte";
    import ndk from "../stores/ndk";
    import type { NDKUser, NDKUserProfile } from "@nostr-dev-kit/ndk";

    interface Props {
        pubkey: string;
    }

    let { pubkey }: Props = $props();

    let user: NDKUser | null = $derived($ndk.getUser({ pubkey }));
    let profile: Promise<NDKUserProfile | null> | null = $derived(user?.fetchProfile());
</script>

<div class="text-lg font-semibold">
    {#if profile !== null}
        {#await profile then profile}
            {#if profile?.displayName || profile?.name}
                {profile.displayName || profile.name}
            {:else}
                {`${user?.npub.slice(0, 20)}...`}
            {/if}
        {/await}
    {/if}
</div>
