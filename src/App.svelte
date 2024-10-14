<script lang="ts">
    import { fetchAccounts, identities, currentIdentity, switchIdentity } from "./stores/accounts";
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
        LoginScreen,
        Input,
        Button,
        Page,
        Panel,
        Toolbar,
        View,
        Views,
    } from "framework7-svelte";
    import routes from "./js/routes";
    import { Dom7 } from "framework7";
    import { npubFromPubkey } from "./utils/nostr";
    import { invoke } from "@tauri-apps/api/core";
    import { login } from "./stores/accounts";

    const f7params = {
        routes,
        theme: "auto",
        name: "White Noise",
        darkMode: "dark",
        themeColor: "#1d4ed8", // blue-600
        // on: {
        //     routeChanged: (
        //         newRoute: Router.Route,
        //         oldRoute: Router.Route,
        //         router: F7Router.Router
        //     ) => {
        //         console.log("<===== routeChanged =====>");
        //         console.log("newRoute: ", newRoute.path);
        //         console.log("oldRoute: ", oldRoute.path);
        //         console.log("history: ", router.history);
        //         console.log("<===== routeChanged =====>");
        //     },
        // },
    };

    let isLoadingAccounts = $state(true);
    let activeTab = $state("chats");
    let previousTab = $state("chats");
    let loginLoading = $state(false);
    let nsecOrHex: string = $state("");

    async function handleCreateIdentity() {
        if (loginLoading) return;
        loginLoading = true;
        await invoke("create_identity");
        if (!!$currentIdentity) {
            f7.loginScreen.get("#login-screen").close();
        }
        loginLoading = false;
    }

    async function handleLogin(e: Event) {
        e.preventDefault();
        if (loginLoading) return;
        loginLoading = true;
        await login(nsecOrHex, `login-${Date.now()}`); // TODO: ERROR HANDLING
        if (!!$currentIdentity) {
            f7.loginScreen.get("#login-screen").close();
        }
        nsecOrHex = "";
        loginLoading = false;
    }

    f7ready(async () => {
        console.log("f7ready");
        await updateAccounts();
        isLoadingAccounts = false;
    });

    function onTabLinkClick(tab: string) {
        console.log("onTabLinkClick", tab);
        if (previousTab !== activeTab) {
            previousTab = activeTab;
            setTimeout(() => {
                f7.panel.close("#menu-panel-left");
            }, 100);
            return;
        }
        if (activeTab === tab) {
            Dom7(`#view-${tab}`)[0].f7View.router.back();
        }
        previousTab = tab;
    }

    async function updateAccounts() {
        await fetchAccounts();
    }
</script>

<App {...f7params}>
    <LoginScreen class="login-screen" id="login-screen" opened={!!!$currentIdentity}>
        <Page loginScreen>
            <div class="flex flex-col items-center justify-center w-screen h-screen bg-gray-800">
                <div
                    class="bg-gray-800 w-full md:w-1/2 h-2/3 flex flex-col items-center justify-center gap-6 py-12 px-6"
                >
                    <h1 class="text-5xl font-extrabold text-center">White Noise</h1>
                    <h2 class="text-3xl font-medium text-center">
                        Secure. Distributed. Uncensorable.
                    </h2>
                    <form onsubmit={handleLogin} class="w-full md:w-4/5 flex flex-col gap-4 mt-12">
                        <Input
                            bind:value={nsecOrHex}
                            type="password"
                            clearButton
                            placeholder="nsec1&hellip;"
                            autocorrect="off"
                            autocapitalize="off"
                            inputStyle="padding: 0.875rem 1rem;"
                            class="text-lg"
                        ></Input>
                        <Button
                            type="submit"
                            class="p-3 font-semibold bg-blue-700 hover:bg-blue-600 rounded-md ring-1 ring-blue-500"
                        >
                            Log In
                        </Button>
                    </form>

                    <h3 class="font-semibold text-gray-400">OR</h3>
                    <button
                        class="p-3 w-full md:w-4/5 font-semibold bg-indigo-700 hover:bg-indigo-600 rounded-md ring-1 ring-indigo-500"
                        onclick={handleCreateIdentity}>Create a new Nostr identity</button
                    >
                </div>
                <div class="flex flex-row gap-1 items-end mt-20">
                    Powered by
                    <img src="../images/nostr.webp" alt="nostr" class="w-20" />
                </div>
            </div>
        </Page>
    </LoginScreen>
    <Page>
        <Views tabs>
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
                <Toolbar tabbar icons bottom>
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

            <View
                id="view-chats"
                main
                tabActive
                onTabShow={() => (activeTab = "chats")}
                tab
                url="/chats/"
                animate={false}
            />
            <View
                id="view-calls"
                onTabShow={() => (activeTab = "calls")}
                tab
                url="/calls/"
                animate={false}
            />
            <View
                id="view-settings"
                onTabShow={() => (activeTab = "settings")}
                tab
                url="/settings/"
                animate={false}
            />
        </Views>
    </Page>
</App>
