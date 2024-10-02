<script lang="ts">
    import { fetchAccounts, identities, currentIdentity, switchIdentity } from "./stores/accounts";
    import Login from "./pages/Login.svelte";
    import Name from "./components/Name.svelte";
    import Avatar from "./components/Avatar.svelte";
    import {
        App,
        BlockTitle,
        f7,
        f7ready,
        Link,
        List,
        ListItem,
        Panel,
        Toolbar,
        View,
        Views,
    } from "framework7-svelte";
    import routes from "./js/routes";
    import { Dom7 } from "framework7";
    import { npubFromPubkey } from "./utils/nostr";

    const f7params = {
        routes,
        theme: "auto",
        name: "White Noise",
        darkMode: "dark",
        themeColor: "#1d4ed8", // blue-600
    };

    let isLoadingAccounts = $state(true);
    let activeTab = $state("chats");
    let previousTab = $state("chats");
    let showLoginScreen = $state(false);

    f7ready(async () => {
        await updateAccounts();
        isLoadingAccounts = false;
    });

    function onTabLinkClick(tab: string) {
        if (previousTab !== activeTab) {
            previousTab = activeTab;
            return;
        }
        if (activeTab === tab) {
            Dom7(`#view-${tab}`)[0].f7View.router.back();
        }
        previousTab = tab;
    }

    async function updateAccounts() {
        await fetchAccounts();
        showLoginScreen = !$currentIdentity;
    }
</script>

<App {...f7params}>
    <Login showLoginScreen={!$currentIdentity} />
    <Views tabs class="safe-areas">
        {#if f7.device.desktop}
            <Panel left push id="menu-panel-left" class="bg-gray-900">
                <List>
                    <ListItem>
                        <Link
                            tabLink="#view-chats"
                            iconF7="chat_bubble_2"
                            tabLinkActive
                            text="Chats"
                            on:click={() => onTabLinkClick("chats")}
                        />
                    </ListItem>
                    <ListItem>
                        <Link
                            tabLink="#view-calls"
                            iconF7="phone"
                            text="Calls"
                            on:click={() => onTabLinkClick("calls")}
                        />
                    </ListItem>
                    <ListItem>
                        <Link
                            tabLink="#view-settings"
                            iconF7="gear"
                            text="Settings"
                            on:click={() => onTabLinkClick("settings")}
                        />
                    </ListItem>
                </List>
                <BlockTitle>Accounts</BlockTitle>
                <List mediaList>
                    {#each Object.entries($identities) as [pubkey, identity] (pubkey)}
                        <ListItem link noChevron on:click={() => switchIdentity(pubkey)}>
                            <Avatar
                                slot="media"
                                {pubkey}
                                picture={identity.metadata?.picture}
                                showRing={$currentIdentity === pubkey}
                            />
                            <Name slot="title" {pubkey} metadata={identity.metadata} />
                            <div slot="footer" class="font-mono text-sm truncate">
                                {npubFromPubkey(pubkey)}
                            </div>
                        </ListItem>
                    {/each}
                </List>
            </Panel>
        {:else}
            <Toolbar tabbar icons bottom class="py-10">
                <Link
                    tabLink="#view-chats"
                    iconF7="chat_bubble_2"
                    tabLinkActive
                    text="Chats"
                    on:click={() => onTabLinkClick("chats")}
                />
                <Link
                    tabLink="#view-calls"
                    iconF7="phone"
                    text="Calls"
                    on:click={() => onTabLinkClick("calls")}
                />
                <Link
                    tabLink="#view-settings"
                    iconF7="gear"
                    text="Settings"
                    on:click={() => onTabLinkClick("settings")}
                />
            </Toolbar>
        {/if}
        <View id="view-chats" onTabShow={() => (activeTab = "chats")} tabActive tab url="/chats/" />
        <View id="view-calls" onTabShow={() => (activeTab = "calls")} tab url="/calls/" />
        <View id="view-settings" onTabShow={() => (activeTab = "settings")} tab url="/settings/" />
    </Views>
</App>
