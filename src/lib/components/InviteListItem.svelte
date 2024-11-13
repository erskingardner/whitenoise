<script lang="ts">
    import GroupAvatar from "./GroupAvatar.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import type { EnrichedContact, Invite } from "$lib/types/nostr";
    import Modal from "./Modals/Modal.svelte";
    import InviteDetail from "./Modals/Invites/InviteDetail.svelte";

    let { invite } = $props<{
        invite: Invite;
    }>();

    let showModal = $state(false);
    let enrichedInviter: EnrichedContact | undefined = $state(undefined);

    $effect(() => {
        if (invite.inviter) {
            invoke("fetch_enriched_contact", { pubkey: invite.inviter, updateAccount: false }).then((value) => {
                enrichedInviter = value as EnrichedContact;
            });
        }
    });
</script>

<button
    onclick={() => (showModal = !showModal)}
    class="flex flex-row gap-2 items-center px-4 py-3 border-b border-gray-700 hover:bg-gray-700"
>
    <GroupAvatar
        groupType={invite.group_type}
        groupName={invite.group_name}
        counterpartyPubkey={invite.inviter}
        enrichedCounterparty={enrichedInviter}
        pxSize={40}
    />
    <div class="flex flex-col gap-1">
        <span class="text-lg font-semibold">{invite.group_name}</span>
    </div>
</button>

{#if showModal}
    <Modal initialComponent={InviteDetail} props={{ invite, enrichedInviter }} bind:showModal />
{/if}
