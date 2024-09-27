<script lang="ts">
    import {
        Page,
        Navbar,
        Link,
        List,
        ListItem,
        SwipeoutActions,
        SwipeoutButton,
        Icon,
        f7,
    } from "framework7-svelte";
    import Loader from "../components/Loader.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { currentIdentity } from "../stores/accounts";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import type { NChat, NMetadata } from "../types/nostr";
    import { nameFromMetadata } from "../utils/nostr";
    import { formatMessageTime } from "../utils/time";
    import Avatar from "../components/Avatar.svelte";
    import { Checks, LockKey, Warning } from "phosphor-svelte";

    let isLoading = $state(true);
    let selectedChat: string | undefined = $state(undefined);
    let chats: NChat = $state({});

    let unlisten: UnlistenFn;

    let { f7router } = $props();

    async function getLegacyChats(): Promise<void> {
        isLoading = true;
        chats = {};
        selectedChat = undefined;

        while (!$currentIdentity) {
            console.log("No current identity, retrying in 500ms...");
            await new Promise((resolve) => setTimeout(resolve, 500));
        }

        try {
            const fetchedChats = (await invoke("get_legacy_chats", {
                pubkey: $currentIdentity,
            })) as NChat;
            const sortedChats = Object.entries(fetchedChats)
                .sort(([, a], [, b]) => b.latest - a.latest)
                .reduce((acc, [key, value]) => ({ ...acc, [key]: value }), {});
            chats = sortedChats;
        } catch (error) {
            console.error("Error fetching contacts:", error);
        } finally {
            isLoading = false;
        }
    }

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
    const onContactSelect = (pubkey: string, metadata: NMetadata) => {
        console.log("start new chat with", pubkey);
        if (chats[pubkey]) {
            setTimeout(() => {
                f7router.navigate(`/chats/${pubkey}/`, {
                    props: {
                        chat: chats[pubkey],
                    },
                });
            }, 300);
        } else {
            setTimeout(() => {
                f7router.navigate(`/chats/${pubkey}/`, {
                    props: {
                        chat: { latest: undefined, metadata: metadata, events: [] },
                    },
                });
            }, 300);
        }
    };

    let warningTooltip: any;
</script>

<Page
    class="chats-page bg-gray-900"
    on:pageAfterIn={async () => {
        warningTooltip = f7.tooltip.create({
            targetEl: ".warning-tooltip",
            text: "This is a NIP-04 encrypted message.<br /><em>All metadata is publicly visible.</em>",
        });
        console.log("pageAfterIn");
        await getLegacyChats();
        unlisten = await listen<string>("identity_change", (_event) => getLegacyChats());
    }}
    on:pageReinit={async () => {
        console.log("reinit");
        await getLegacyChats();
    }}
    on:pageBeforeRemove={() => {
        console.log("pageBeforeRemove");
        if (warningTooltip) f7.tooltip.destroy(warningTooltip);
        unlisten();
    }}
>
    {#if isLoading}
        <div
            class="text-center absolute left-1/2 -translate-x-1/2 top-4 flex flex-row items-center gap-2 text-gray-400"
        >
            <Loader size={24} fullscreen={false} />
            <span>Loading&hellip;</span>
        </div>
    {/if}
    {#if f7.device.desktop}
        <Navbar title="Chats" large transparent class="relative">
            <Link slot="left" iconF7="bars" onClick={() => f7.panel.toggle("#menu-panel-left")} />
            <Link
                slot="right"
                iconF7="plus_circle"
                href="/contacts/"
                routeProps={{
                    modalTitle: "New Chat",
                    onContactSelect,
                }}
            />
        </Navbar>
    {:else}
        <Navbar title="Chats" large transparent class="relative">
            <Link
                slot="right"
                iconF7="plus_circle"
                href="/contacts/"
                routeProps={{
                    modalTitle: "New Chat",
                }}
            />
        </Navbar>
    {/if}
    <List noChevron mediaList class="chats-list">
        {#each Object.entries(chats) as [pubkey, chat] (pubkey)}
            <ListItem
                link="/chats/{pubkey}/"
                swipeout
                class="hover:bg-gray-800 {selectedChat === pubkey
                    ? 'bg-gray-800'
                    : ''} transition-colors duration-200"
                routeProps={{
                    chat,
                }}
            >
                <Avatar slot="media" picture={chat.metadata.picture} {pubkey} pxSize={48} />
                <div slot="title" class="flex flex-col items-start justify-start gap-0">
                    <span class="">{nameFromMetadata(chat.metadata, pubkey)}</span>
                    <span
                        class="text-gray-400 font-normal text-base flex flex-row items-center gap-2"
                    >
                        {#if chat.events[chat.events.length - 1].pubkey === $currentIdentity}
                            <Checks class="text-green-500 w-4 h-4 shrink-0" />
                        {/if}
                        {chat.events[chat.events.length - 1].content}
                    </span>
                </div>
                <span slot="text" class=""> </span>
                <div slot="after" class="flex flex-col items-end justify-start gap-0">
                    <span>{formatMessageTime(chat.latest)}</span>
                    <span class="z-50">
                        {#if [4, 14].includes(chat.events[0].kind)}
                            <Warning
                                weight="fill"
                                size={18}
                                class="warning-tooltip {chat.events[0].kind === 4
                                    ? 'text-red-500'
                                    : 'text-yellow-400'}"
                            />
                        {:else}
                            <LockKey weight="fill" size={18} class="text-green-500" />
                        {/if}
                    </span>
                </div>
                <SwipeoutActions left>
                    <SwipeoutButton close overswipe color="blue" onClick={swipeoutUnread}>
                        <Icon f7="chat_bubble_fill" />
                        <span>Unread</span>
                    </SwipeoutButton>
                    <SwipeoutButton close color="gray" onClick={swipeoutPin}>
                        <Icon f7="pin_fill" />
                        <span>Pin</span>
                    </SwipeoutButton>
                </SwipeoutActions>
                <SwipeoutActions right>
                    <SwipeoutButton close color="gray" onClick={swipeoutMore}>
                        <Icon f7="ellipsis" />
                        <span>More</span>
                    </SwipeoutButton>
                    <SwipeoutButton close overswipe color="light-blue" onClick={swipeoutArchive}>
                        <Icon f7="archivebox_fill" />
                        <span>Archive</span>
                    </SwipeoutButton>
                </SwipeoutActions>
            </ListItem>
        {/each}
    </List>
</Page>
