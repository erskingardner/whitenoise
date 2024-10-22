<script lang="ts">
    import { f7, Navbar, Page, Popup, View, Block, Button, Link } from "framework7-svelte";
    import Avatar from "../components/Avatar.svelte";
    import InviteSubheader from "../components/InviteSubheader.svelte";
    import type { EnrichedContact, Invite } from "../types/nostr";
    import { invoke } from "@tauri-apps/api/core";
    import { X } from "phosphor-svelte";

    let {
        invite,
        enrichedInvitee = $bindable(),
    }: {
        invite: Invite;
        enrichedInvitee: EnrichedContact;
    } = $props();

    function acceptInvite() {
        invoke("accept_invite", { invite });
        f7.popup.close("#group-invite-popup");
        const event = new CustomEvent("inviteAccepted", { detail: invite.mls_group_id });
        window.dispatchEvent(event);
    }
    function declineInvite() {
        invoke("decline_invite", { invite });
    }
    function closePopup() {
        f7.popup.close("#group-invite-popup");
    }
</script>

<Popup push closeByBackdropClick closeOnEscape id="group-invite-popup">
    <View tab tabActive id="group-info-popup-view">
        <Page class="group-info-page bg-gray-900" noToolbar>
            <Navbar class="group-info-navbar justify-start py-8">
                <div slot="title" class="title-profile-link flex flex-row gap-2 items-center">
                    <div class="flex flex-col">Accept Invite?</div>
                </div>
                <Button slot="right" on:click={closePopup}>
                    <X size={24} />
                </Button>
            </Navbar>
            <Block>
                <div class="flex flex-col justify-start items-center gap-4">
                    <Avatar
                        pubkey={invite.invitee}
                        picture={enrichedInvitee?.metadata?.picture}
                        pxSize={64}
                    />
                    <InviteSubheader {invite} {enrichedInvitee} />
                </div>
                <div class="flex flex-col justify-center items-center gap-4 my-6">
                    <button
                        class="px-3 py-2 flex flex-row shrink items-center justify-center text-center gap-2 font-semibold bg-blue-700 hover:bg-blue-600 rounded-md ring-1 ring-blue-500"
                        onclick={acceptInvite}>Accept</button
                    >
                    <button
                        class="px-3 py-2 text-center flex flex-row items-center justify-center gap-2 rounded-md bg-gray-700 hover:bg-gray-600 ring-1 ring-gray-500"
                        onclick={declineInvite}>Decline</button
                    >
                </div>
            </Block>
        </Page>
    </View>
</Popup>
