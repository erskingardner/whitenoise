<script lang="ts">
    import {
        Page,
        Navbar,
        Link,
        f7,
        BlockTitle,
        List,
        ListItem,
        Button,
        Input,
    } from "framework7-svelte";
    import {
        Binoculars,
        Key,
        Skull,
        Users,
        SignIn,
        PlusCircle,
        Trash,
        UserPlus,
    } from "phosphor-svelte";
    import Avatar from "../components/Avatar.svelte";
    import {
        identities,
        currentIdentity,
        logout,
        login,
        createIdentity,
        switchIdentity,
    } from "../stores/accounts";
    import { invoke } from "@tauri-apps/api/core";
    import { nameFromMetadata, npubFromPubkey } from "../utils/nostr";
    import ndk from "../stores/ndk";
    import type { NDKEvent } from "@nostr-dev-kit/ndk";

    let keyPackages: unknown[] = $state([]);
    let showAccounts = $state(false);
    let showLogin = $state(false);
    let nsecOrHex = $state("");
    let welcomeMessages: unknown[] = $state([]);

    $effect(() => {
        // Do something when $currentIdentity changes
        if ($currentIdentity) {
            keyPackages = [];
            showAccounts = false;
        }
    });

    let keyPackageTitle = $derived(
        `${keyPackages.length} Key Package${keyPackages.length === 1 ? "" : "s"}`
    );

    async function handleLogin() {
        await login(nsecOrHex);
        nsecOrHex = ""; // Clear the input field
        showLogin = false;
    }

    async function handleCreateIdentity() {
        await createIdentity();
        showLogin = false;
    }

    async function nukeAll() {
        f7.dialog.confirm(
            "Are you sure you want to delete all app data and settings?",
            "Delete all data",
            async () => {
                await invoke("delete_app_data");
                $identities = {};
                $currentIdentity = "";
            }
        );
    }

    async function generateAndPublishKeyPackage() {
        f7.dialog.confirm(
            "Are you sure you want to create and publish a new key package?",
            "Publish a Key Package",
            async () => {
                invoke("generate_and_publish_key_package", { pubkey: $currentIdentity })
                    .then(() => {
                        console.log("Key package published");
                        const toast = f7.toast.create({
                            text: "Key package published",
                            closeTimeout: 2500,
                            position: "top",
                        });
                        toast.open();
                    })
                    .catch((error) => {
                        console.error(error);
                        const toast = f7.toast.create({
                            text: error,
                            closeTimeout: 2500,
                            position: "top",
                        });
                        toast.open();
                    });
            }
        );
    }

    // Fetch key packages from Nostr and parse them
    async function fetchKeyPackages(): Promise<void> {
        const keyPackageEvents = await $ndk.fetchEvents({
            kinds: [443 as number],
            authors: [$currentIdentity],
        });
        keyPackages = [];
        keyPackageEvents.forEach(async (event: NDKEvent) => {
            console.log(event);
            const keyPackage = await invoke("parse_key_package", { keyPackageHex: event.content });
            keyPackages.push(keyPackage);
        });
    }

    // Publish delete requests for all key package events on relays
    async function deleteKeyPackages() {
        f7.dialog.confirm(
            "Are you sure you want to delete all key packages from relays?",
            "Delete key packages",
            async () => {
                await invoke("delete_key_packages")
                    .then(() => (keyPackages = []))
                    .catch((error) => {
                        f7.toast.create({
                            text: error.message,
                            closeTimeout: 2500,
                            position: "top",
                        });
                        console.error(error);
                    });
            }
        );
    }

    async function fetchWelcomeMessages() {
        welcomeMessages = await invoke("fetch_welcome_messages_for_user", {
            pubkey: $currentIdentity,
        });
    }
</script>

<Page class="settings-page bg-gray-900">
    <Navbar title="Settings" large transparent>
        <Link slot="left" iconF7="bars" onClick={() => f7.panel.toggle("#menu-panel-left")} />
    </Navbar>
    <BlockTitle>Profiles</BlockTitle>
    <List dividers outline mediaList class="profile-settings-list">
        {#each Object.entries($identities) as [pubkey, identity] (pubkey)}
            <ListItem
                mediaItem
                noChevron
                class={$currentIdentity === pubkey ? "bg-gray-800" : ""}
                on:click={() => switchIdentity(pubkey)}
            >
                <Avatar
                    slot="media"
                    {pubkey}
                    picture={identity.metadata?.picture}
                    showRing={$currentIdentity === pubkey}
                />
                <div slot="title" class="truncate">
                    {nameFromMetadata(identity.metadata, pubkey)}
                </div>
                <div slot="footer" class="font-mono truncate">
                    {npubFromPubkey(pubkey)}
                </div>
                <Button slot="after" tonal onClick={() => logout(pubkey)}>Log out</Button>
            </ListItem>
        {/each}
    </List>

    <Button tonal on:click={() => (showLogin = !showLogin)} class="flex md:w-1/2 md:mx-auto"
        >Log in or create new account</Button
    >
    <div
        class="{showLogin
            ? 'flex'
            : 'hidden'} flex-col gap-12 items-start md:w-1/2 md:mx-auto mt-10 bg-gray-800 ring-1 ring-gray-700 p-4 md:rounded-md"
    >
        <div class="flex flex-col gap-4 items-start w-full">
            <label for="nsec" class="flex flex-col gap-2 text-lg items-start font-medium w-full">
                Log in with your nsec
                <Input
                    type="password"
                    clearButton
                    id="nsec"
                    bind:value={nsecOrHex}
                    placeholder="nsec1&hellip;"
                    autocapitalize="off"
                    autocorrect="off"
                    inputStyle="padding: 0.875rem 1rem; width: 100%;"
                    class="text-lg w-full"
                />
            </label>
            <button
                type="submit"
                onclick={handleLogin}
                class="px-3 py-2 flex flex-row shrink items-center justify-start gap-2 font-semibold bg-blue-700 hover:bg-blue-600 rounded-md ring-1 ring-blue-500"
            >
                <SignIn size="2rem" weight="thin" />
                Log In
            </button>
        </div>
        <button
            onclick={handleCreateIdentity}
            class="px-3 py-2 text-center flex flex-row items-center gap-2 rounded-md bg-gray-700 hover:bg-gray-600 ring-1 ring-gray-500"
        >
            <PlusCircle size="2rem" weight="thin" />
            Create New Nostr Identity
        </button>
    </div>

    <BlockTitle>Privacy</BlockTitle>
    <List dividers outline mediaList class="privacy-settings-list">
        <ListItem link title="Delete all app data" onClick={nukeAll}>
            <Skull slot="media" size={24} />
        </ListItem>
    </List>

    <BlockTitle>Developer Settings</BlockTitle>
    <List dividers outline mediaList class="developer-settings-list">
        <ListItem link title="Inspect Account" onClick={() => (showAccounts = !showAccounts)}>
            <Users slot="media" size={24} />
        </ListItem>
        <ListItem link title="Fetch Prekey Events" onClick={fetchKeyPackages}>
            <Binoculars slot="media" size={24} />
        </ListItem>
        <ListItem link title="Publish Prekey Event" onClick={generateAndPublishKeyPackage}>
            <Key slot="media" size={24} />
        </ListItem>
        <ListItem link title="Delete all Prekey Events" onClick={deleteKeyPackages}>
            <Trash slot="media" size={24} />
        </ListItem>
        <ListItem link title="Fetch Welcome Messages" onClick={fetchWelcomeMessages}>
            <UserPlus slot="media" size={24} />
        </ListItem>
    </List>

    {#if showAccounts}
        <BlockTitle>Accounts</BlockTitle>
        <div class="p-4 rounded-md bg-gray-800 ring-1 ring-gray-700 mx-4">
            <pre class="overflow-x-scroll">
                <code class="language-json whitespace-pre font-mono">
{JSON.stringify($identities[$currentIdentity], null, 4)}
                </code>
            </pre>
        </div>
    {/if}

    {#if keyPackages.length > 0}
        <BlockTitle>{keyPackageTitle}</BlockTitle>
        {#each keyPackages as keyPackage}
            <div class="p-4 rounded-md bg-gray-800 ring-1 ring-gray-700 mx-4">
                <pre class="overflow-x-scroll">
                    <code class="language-json whitespace-pre break-words font-mono">
{JSON.stringify(keyPackage, null, 4)}
                    </code>
                </pre>
            </div>
        {/each}
    {/if}

    {#if welcomeMessages.length > 0}
        <BlockTitle>Welcome Messages</BlockTitle>
        {#each welcomeMessages as welcomeMessage}
            <div class="p-4 rounded-md bg-gray-800 ring-1 ring-gray-700 mx-4">
                <pre class="overflow-x-scroll">
                <code class="language-json whitespace-pre break-words font-mono">
{JSON.stringify(welcomeMessage, null, 4)}
                </code>
            </pre>
            </div>
        {/each}
    {/if}
</Page>
