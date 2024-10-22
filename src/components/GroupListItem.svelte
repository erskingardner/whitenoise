<script lang="ts">
    import { currentIdentity } from "../stores/accounts";
    import type { NostrMlsGroup, EnrichedContact } from "../types/nostr";
    import { NostrMlsGroupType } from "../types/nostr";
    import { hexMlsGroupId } from "../utils/group";
    import { formatMessageTime } from "../utils/time";
    import { f7, ListItem, SwipeoutActions, SwipeoutButton, Icon } from "framework7-svelte";
    import { Checks, LockKey } from "phosphor-svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { nameFromMetadata } from "../utils/nostr";
    import GroupAvatar from "./GroupAvatar.svelte";

    let { group }: { group: NostrMlsGroup } = $props();

    let counterpartyPubkey =
        group.group_type === NostrMlsGroupType.DirectMessage
            ? group.admin_pubkeys.filter((pubkey) => pubkey !== $currentIdentity)[0]
            : undefined;

    let enrichedCounterparty: EnrichedContact | undefined = $state(undefined);
    let groupName = $state("");

    $effect(() => {
        if (
            group.group_type === NostrMlsGroupType.DirectMessage &&
            counterpartyPubkey &&
            enrichedCounterparty
        ) {
            groupName = nameFromMetadata((enrichedCounterparty as EnrichedContact).metadata);
        } else {
            groupName = group.group_name;
        }
    });

    $effect(() => {
        if (counterpartyPubkey) {
            invoke("get_contact", { pubkey: counterpartyPubkey }).then((value) => {
                enrichedCounterparty = value as EnrichedContact;
            });
        }
    });

    function swipeoutUnread() {
        f7.dialog.alert("Unread");
    }
    function swipeoutPin() {
        f7.dialog.alert("Pin");
    }
    function swipeoutMore() {
        f7.dialog.alert("More");
    }
    function swipeoutArchive() {
        f7.dialog.alert("Archive");
    }
</script>

<ListItem
    id="group-list-item-{hexMlsGroupId(group.mls_group_id)}"
    link="/groups/{hexMlsGroupId(group.mls_group_id)}/"
    swipeout
    class="hover:bg-gray-800 transition-colors duration-200"
    routeProps={{
        group,
    }}
>
    <div slot="media">
        <GroupAvatar
            groupType={group.group_type}
            {groupName}
            {counterpartyPubkey}
            {enrichedCounterparty}
            pxSize={48}
        />
    </div>
    <div slot="title" class="flex flex-col items-start justify-start gap-0">
        <span class="flex flex-row items-center gap-2">
            <span class="z-50">
                <LockKey weight="light" size={18} class="text-green-500" />
            </span>
            {groupName}
        </span>
        <span class="text-gray-400 font-normal text-base flex flex-row items-center gap-2">
            {#if group.transcript.length > 0}
                <div class="w-[18px] h-[18px]">
                    {#if group.transcript[group.transcript.length - 1].pubkey === $currentIdentity}
                        <Checks size={18} class="text-green-500 shrink-0" />
                    {/if}
                </div>
                {group.transcript[group.transcript.length - 1].content}
            {:else}
                <div class="w-[18px] h-[18px]"></div>
                <span class="text-gray-400">No messages</span>
            {/if}
        </span>
    </div>
    <span slot="text" class=""> </span>
    <span slot="after">{formatMessageTime(group.last_message_at)}</span>
    <!-- <SwipeoutActions left>
        <SwipeoutButton close overswipe color="blue" on:click={swipeoutUnread}>
            <Icon f7="chat_bubble_fill" />
            <span>Unread</span>
        </SwipeoutButton>
        <SwipeoutButton close color="gray" on:click={swipeoutPin}>
            <Icon f7="pin_fill" />
            <span>Pin</span>
        </SwipeoutButton>
    </SwipeoutActions>
    <SwipeoutActions right>
        <SwipeoutButton close color="gray" on:click={swipeoutMore}>
            <Icon f7="ellipsis" />
            <span>More</span>
        </SwipeoutButton>
        <SwipeoutButton close overswipe color="light-blue" on:click={swipeoutArchive}>
            <Icon f7="archivebox_fill" />
            <span>Archive</span>
        </SwipeoutButton>
    </SwipeoutActions> -->
</ListItem>
