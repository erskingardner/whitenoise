<script lang="ts">
    import type { WelcomeMessage } from "../types/nostr";
    import { ListItem, SwipeoutButton, SwipeoutActions, Icon, f7 } from "framework7-svelte";
    import Avatar from "./Avatar.svelte";
    import Name from "./Name.svelte";
    import { hexMlsGroupId } from "../utils/group";
    import { invoke } from "@tauri-apps/api/core";
    import type { EnrichedContact } from "../types/nostr";
    import { LockKey } from "phosphor-svelte";
    import { formatMessageTime } from "../utils/time";

    let { welcome }: { welcome: WelcomeMessage } = $props();

    let inviteePubkey = $derived(welcome.invitee);
    let enrichedInvitee: EnrichedContact | undefined = $state(undefined);
    let groupName = $derived(welcome.nostr_group_data.name);

    $effect(() => {
        if (inviteePubkey) {
            invoke("get_contact", { pubkey: inviteePubkey }).then((value) => {
                enrichedInvitee = value as EnrichedContact;
            });
        }
    });

    $inspect(enrichedInvitee);

    function swipeoutAccept() {
        f7.dialog.alert("Accept");
    }
    function swipeoutArchive() {
        f7.dialog.alert("Archive");
    }
</script>

<ListItem
    link="/groups/{hexMlsGroupId(welcome.nostr_group_data.mls_group_id)}/"
    swipeout
    class="hover:bg-gray-800 transition-colors duration-200"
    routeProps={{
        welcome,
    }}
>
    <div slot="media">
        <Avatar pubkey={inviteePubkey} picture={enrichedInvitee?.metadata?.picture} pxSize={48} />
    </div>
    <div slot="title" class="flex flex-col items-start justify-start gap-0">
        <span class="flex flex-row items-center gap-2">
            <span class="z-50">
                <LockKey weight="light" size={18} class="text-green-500" />
            </span>
            {groupName}
        </span>
        <span class="text-gray-400 font-normal text-base flex flex-row items-center gap-2">
            <Name pubkey={inviteePubkey} metadata={enrichedInvitee?.metadata} /> has invited you to join
            {groupName}.
        </span>
    </div>
    <span slot="text" class=""> </span>
    <span slot="after">{formatMessageTime(welcome.event.created_at)}</span>
    <SwipeoutActions left>
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
    </SwipeoutActions>
</ListItem>
