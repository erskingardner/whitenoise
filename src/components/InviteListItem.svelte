<script lang="ts">
    import type { Invite } from "../types/nostr";
    import { ListItem, SwipeoutButton, SwipeoutActions, Icon, f7 } from "framework7-svelte";
    import Avatar from "./Avatar.svelte";
    import Name from "./Name.svelte";
    import InviteSubheader from "./InviteSubheader.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import type { EnrichedContact } from "../types/nostr";
    import { LockKey } from "phosphor-svelte";
    import { formatMessageTime } from "../utils/time";

    let { invite }: { invite: Invite } = $props();
    let enrichedInvitee: EnrichedContact | undefined = $state(undefined);

    $effect(() => {
        if (invite.invitee) {
            invoke("get_contact", { pubkey: invite.invitee }).then((value) => {
                enrichedInvitee = value as EnrichedContact;
            });
        }
    });

    function swipeoutAccept() {
        f7.dialog.alert("Accept");
    }
    function swipeoutArchive() {
        f7.dialog.alert("Archive");
    }
</script>

<ListItem
    link="/groups/{invite.mls_group_id}/invite/"
    swipeout
    class="hover:bg-gray-800 transition-colors duration-200"
    routeProps={{
        invite,
        enrichedInvitee,
    }}
>
    <div slot="media">
        <Avatar pubkey={invite.invitee} picture={enrichedInvitee?.metadata?.picture} pxSize={48} />
    </div>
    <div slot="title" class="flex flex-col items-start justify-start gap-0">
        <span class="flex flex-row items-center gap-2">
            <span class="z-50">
                <LockKey weight="light" size={18} class="text-green-500" />
            </span>
            <Name pubkey={invite.invitee} metadata={enrichedInvitee?.metadata} unstyled={true} />
        </span>
        <span class="text-gray-400 font-normal text-base flex flex-row items-center gap-2">
            <div class="w-[18px] h-[18px]"></div>
            <InviteSubheader {invite} {enrichedInvitee} />
        </span>
    </div>
    <span slot="text" class=""> </span>
    <span slot="after">{formatMessageTime(invite.event.created_at)}</span>
    <!-- <SwipeoutActions left>
        <SwipeoutButton close overswipe color="blue" on:click={swipeoutAccept}>
            <Icon f7="hand_thumbsup_fill" />
            <span>Accept</span>
        </SwipeoutButton>
    </SwipeoutActions>
    <SwipeoutActions right>
        <SwipeoutButton close overswipe color="light-blue" on:click={swipeoutArchive}>
            <Icon f7="archivebox_fill" />
            <span>Archive</span>
        </SwipeoutButton>
    </SwipeoutActions> -->
</ListItem>
