<script lang="ts">
    import { Page, Navbar, Link, List, f7, ListGroup, ListItem } from "framework7-svelte";
    import Loader from "../components/Loader.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { currentIdentity } from "../stores/accounts";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import type { NChats, Invite, NMetadata, NostrMlsGroup } from "../types/nostr";
    import type { Router as F7Router } from "framework7/types";
    import LegacyChatListItem from "../components/LegacyChatListItem.svelte";
    import GroupListItem from "../components/GroupListItem.svelte";
    import InviteListItem from "../components/InviteListItem.svelte";
    import { hexMlsGroupId } from "../utils/group";

    let isLoading = $state(true);
    let selectedChat: string | undefined = $state(undefined);
    let chats: NChats = $state({});

    let groups: NostrMlsGroup[] = $state([]);
    let invites: Invite[] = $state([]);

    let unlisten: UnlistenFn;

    let { f7router }: { f7router: F7Router.Router } = $props();

    let inviteAcceptedListener: (event: CustomEvent) => void;

    inviteAcceptedListener = async (event: CustomEvent) => {
        const acceptedGroupId = event.detail;
        invites = invites.filter((invite) => invite.mls_group_id !== acceptedGroupId);
        await getEvents();
        f7router.navigate(`/groups/${acceptedGroupId}/`);
    };

    window.addEventListener("inviteAccepted", inviteAcceptedListener as EventListener);

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

    async function getInvites() {
        if (!$currentIdentity) {
            invites = [];
        } else {
            invites = await invoke("fetch_invites_for_user", {
                pubkey: $currentIdentity,
            });
        }
    }

    $inspect("groups", groups);
    async function getGroups() {
        if (!$currentIdentity) {
            groups = [];
        } else {
            groups = await invoke("get_groups");
            await invoke("fetch_and_process_mls_messages");
        }
    }

    async function getEvents() {
        await getLegacyChats();
        await getInvites();
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
                console.log("identity_change event received in Chats.svelte");
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
        {#if invites.length > 0}
            <ListGroup>
                <ListItem groupTitle title="Invites" class="list-group p-0 w-full" />
                {#each invites as invite (invite.event.id)}
                    <InviteListItem {invite} />
                {/each}
            </ListGroup>
        {/if}
        <ListGroup>
            <ListItem groupTitle title="Secure Chats" class="list-group p-0 w-full" />
            {#each groups as group (hexMlsGroupId(group.mls_group_id))}
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
