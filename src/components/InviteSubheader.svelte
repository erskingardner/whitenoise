<script lang="ts">
    import Name from "./Name.svelte";
    import type { EnrichedContact, Invite } from "../types/nostr";
    let {
        invite,
        enrichedInvitee = $bindable(),
    }: { invite: Invite; enrichedInvitee: EnrichedContact | undefined } = $props();

    let subhead = $derived(
        invite.member_count === 2
            ? "has invited you to join a secure chat."
            : `has invited you to join ${invite.group_name}, a group with ${invite.member_count} members.`
    );
</script>

<span class="flex flex-row items-baseline gap-1">
    <Name pubkey={invite.invitee} metadata={enrichedInvitee?.metadata} />
    <span>{subhead}</span>
</span>
