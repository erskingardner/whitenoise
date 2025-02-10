<script lang="ts">
import Avatar from "$lib/components/Avatar.svelte";
import Name from "$lib/components/Name.svelte";
import { getToastState } from "$lib/stores/toast-state.svelte";
import type { EnrichedContact, Invite } from "$lib/types/nostr";
import { nameFromMetadata } from "$lib/utils/nostr";
import { invoke } from "@tauri-apps/api/core";
import { onDestroy } from "svelte";

let { invite, enrichedInviter, closeModal } = $props<{
    invite?: Invite;
    enrichedInviter?: EnrichedContact;
    closeModal?: () => void;
}>();

let toastState = getToastState();

let subhead = $derived(
    invite.member_count === 2
        ? "has invited you to join a secure chat."
        : `has invited you to join ${invite.group_name}, a group with ${invite.member_count} members.`
);

async function acceptInvite() {
    invoke("accept_invite", { invite })
        .then(() => {
            const event = new CustomEvent("inviteAccepted", { detail: invite.mls_group_id });
            window.dispatchEvent(event);
            toastState.add(
                "Accepted Invite",
                `You've accpeted an invite to join a secure chat with ${nameFromMetadata(enrichedInviter.metadata)}`,
                "success"
            );
        })
        .catch((e) => {
            toastState.add("Error accepting invite", e.split(": ")[2], "error");
            console.error(e);
        })
        .finally(() => {
            closeModal();
        });
}

async function declineInvite() {
    invoke("decline_invite", { invite })
        .then(() => {
            toastState.add(
                "Invite declined",
                `You've declined an invite to join a secure chat with ${nameFromMetadata(enrichedInviter.metadata)}`,
                "info"
            );
        })
        .finally(() => {
            closeModal();
        });
}

onDestroy(() => {
    toastState.cleanup();
});
</script>

<div class="flex flex-col justify-start items-center gap-4">
    <Avatar pubkey={invite.inviter} picture={enrichedInviter?.metadata?.picture} pxSize={64} />
    <span class="flex flex-row items-baseline gap-1">
        <Name pubkey={invite.inviter} metadata={enrichedInviter?.metadata} />
        <span>{subhead}</span>
    </span>
</div>
<div class="flex flex-row justify-center items-center gap-4 my-6">
    <button
        class="px-3 py-2 flex flex-row shrink items-center justify-center text-center gap-2 font-semibold bg-blue-700 hover:bg-blue-600 rounded-md ring-1 ring-blue-500"
        onclick={acceptInvite}>Accept</button
    >
    <button
        class="px-3 py-2 text-center flex flex-row items-center justify-center gap-2 rounded-md bg-gray-700 hover:bg-gray-600 ring-1 ring-gray-500"
        onclick={declineInvite}>Decline</button
    >
</div>
