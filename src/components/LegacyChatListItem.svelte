<script lang="ts">
    import { currentIdentity } from "../stores/accounts";
    import type { NChat } from "../types/nostr";
    import { nameFromMetadata } from "../utils/nostr";
    import { formatMessageTime } from "../utils/time";
    import { f7, ListItem, SwipeoutActions, SwipeoutButton, Icon } from "framework7-svelte";
    import Avatar from "./Avatar.svelte";
    import { Checks, LockKey, Warning } from "phosphor-svelte";

    interface Props {
        pubkey: string;
        chat: NChat;
    }

    let { pubkey, chat }: Props = $props();

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
    link="/chats/{pubkey}/"
    swipeout
    class="hover:bg-gray-800 transition-colors duration-200"
    routeProps={{
        chat,
    }}
>
    <Avatar slot="media" picture={chat.metadata.picture} {pubkey} pxSize={48} />
    <div slot="title" class="flex flex-col items-start justify-start gap-0">
        <span class="flex flex-row items-center gap-2">
            <span class="z-50">
                <Warning
                    weight="light"
                    size={18}
                    class="warning-tooltip {chat.events[0].kind === 4
                        ? 'text-red-500'
                        : 'text-yellow-400'}"
                />
            </span>
            {nameFromMetadata(chat.metadata, pubkey)}
        </span>
        <span class="text-gray-400 font-normal text-base flex flex-row items-center gap-2">
            <div class="w-[18px] h-[18px]">
                {#if chat.events[chat.events.length - 1].pubkey === $currentIdentity}
                    <Checks size={18} class="text-green-500 shrink-0" />
                {/if}
            </div>
            {chat.events[chat.events.length - 1].content}
        </span>
    </div>
    <span slot="text" class=""> </span>
    <span slot="after">{formatMessageTime(chat.latest)}</span>
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
