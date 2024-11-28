<script lang="ts">
import type { EnrichedContact, Invite } from "$lib/types/nostr";
import { NostrMlsGroupType } from "$lib/types/nostr";
import { invoke } from "@tauri-apps/api/core";
import { nameFromMetadata } from "../utils/nostr";
import GroupAvatar from "./GroupAvatar.svelte";
import InviteDetail from "./Modals/Invites/InviteDetail.svelte";
import Modal from "./Modals/Modal.svelte";

let { invite }: { invite: Invite } = $props<{
    invite: Invite;
}>();

let showModal = $state(false);
let enrichedInviter: EnrichedContact | undefined = $state(undefined);
let groupName = $state("");
let groupType = $derived(
    invite.member_count === 2 ? NostrMlsGroupType.DirectMessage : NostrMlsGroupType.Group
);
let inviteDescription = $derived(
    invite.member_count === 2 ? "private chat" : `group with ${invite.member_count} members`
);

$effect(() => {
    if (invite.inviter && !enrichedInviter) {
        invoke("query_enriched_contact", { pubkey: invite.inviter, updateAccount: false }).then(
            (value) => {
                enrichedInviter = value as EnrichedContact;
            }
        );
    }
    if (groupType === NostrMlsGroupType.DirectMessage && invite.inviter && enrichedInviter) {
        groupName = nameFromMetadata((enrichedInviter as EnrichedContact).metadata, invite.inviter);
    } else {
        groupName = invite.group_name;
    }
});
</script>

<button
    onclick={() => (showModal = !showModal)}
    class="flex flex-row gap-2 items-center px-4 py-3 border-b border-gray-700 hover:bg-gray-700"
>
    <GroupAvatar
        {groupType}
        {groupName}
        counterpartyPubkey={invite.inviter}
        enrichedCounterparty={enrichedInviter}
        pxSize={40}
    />
    <div class="flex flex-col gap-1 items-start">
        <span class="text-lg font-semibold">{groupName}</span>
        <span class="text-sm text-gray-400">Invite to join a {inviteDescription}</span>
    </div>
</button>

{#if showModal}
    <Modal initialComponent={InviteDetail} props={{ invite, enrichedInviter }} bind:showModal />
{/if}
