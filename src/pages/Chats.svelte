<script lang="ts">
    import { Page, Navbar, Link, List, f7, ListGroup, ListItem } from "framework7-svelte";
    import Loader from "../components/Loader.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { currentIdentity } from "../stores/accounts";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import type { NChats, NEvent, NMetadata, NostrMlsGroup } from "../types/nostr";
    import type { Router as F7Router } from "framework7/types";
    import LegacyChatListItem from "../components/LegacyChatListItem.svelte";
    import GroupListItem from "../components/GroupListItem.svelte";

    let isLoading = $state(true);
    let selectedChat: string | undefined = $state(undefined);
    let chats: NChats = $state({});

    let groups: NostrMlsGroup[] = $state([]);
    let welcomes: NEvent[] = $state([]);

    let unlisten: UnlistenFn;

    let { f7router }: { f7router: F7Router.Router } = $props();

    async function getLegacyChats(): Promise<void> {
        isLoading = true;
        chats = {};
        selectedChat = undefined;

        let retryCount = 0;
        while (!$currentIdentity && retryCount < 15) {
            console.log("No current identity, retrying in 500ms...");
            await new Promise((resolve) => setTimeout(resolve, 500));
            retryCount++;
        }

        if (!$currentIdentity) {
            isLoading = false;
            return;
        } else {
            try {
                const fetchedChats = (await invoke("get_legacy_chats", {
                    pubkey: $currentIdentity,
                })) as NChats;
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
    }

    async function getWelcomeMessages() {
        if (!$currentIdentity) {
            welcomes = [];
        } else {
            welcomes = await invoke("fetch_welcome_messages_for_user", {
                pubkey: $currentIdentity,
            });
        }
    }

    async function getGroups() {
        if (!$currentIdentity) {
            groups = [];
        } else {
            groups = await invoke("get_groups");
        }
    }

    async function getEvents() {
        console.log("getEvents");
        await getLegacyChats();
        await getWelcomeMessages();
        await getGroups();
    }

    const onContactSelect = (pubkey: string, metadata: NMetadata) => {
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
    on:pageInit={async () => {
        console.log("pageInit: Chats");
        if (!unlisten) {
            unlisten = await listen<string>("identity_change", (_event) => {
                setTimeout(getEvents, 100);
            });
        }
        warningTooltip = f7.tooltip.create({
            targetEl: ".warning-tooltip",
            text: "This is a NIP-04 encrypted message.<br /><em>All metadata is publicly visible.</em>",
        });
        await getEvents();
    }}
    on:pageTabShow={async () => {
        console.log("pageTabShow: Chats");
        await getEvents();
    }}
    on:pageBeforeRemove={() => {
        console.log("pageBeforeRemove: Chats");
        if (warningTooltip) f7.tooltip.destroy(warningTooltip);
        if (unlisten) unlisten();
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
            <Link slot="left" iconF7="bars" on:click={() => f7.panel.toggle("#menu-panel-left")} />
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

    <List noChevron mediaList dividers ul={false}>
        <ListGroup>
            <ListItem groupTitle title="Secure Chats" class="list-group p-0 w-full" />
            {#each groups as group}
                <GroupListItem {group} />
            {/each}
        </ListGroup>
        <ListGroup>
            <ListItem groupTitle title="Legacy Chats" class="list-group p-0 w-full" />
            {#each Object.entries(chats) as [pubkey, chat] (pubkey)}
                <LegacyChatListItem {pubkey} {chat} />
            {/each}
        </ListGroup>
    </List>
</Page>
