<script lang="ts">
    import { Page, Navbar, Link, f7, BlockTitle, List, ListItem, Button } from "framework7-svelte";
    import { Binoculars, Key, Skull, Users, SignIn, PlusCircle } from "phosphor-svelte";
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
    import { MLSCiphersuites } from "../types/mls";

    let keyPackages: unknown[] = $state([]);
    let showAccounts = $state(false);
    let showLogin = $state(false);
    let nsecOrHex = $state("");

    async function handleLogin() {
        await login(nsecOrHex);
        nsecOrHex = ""; // Clear the input field
    }

    async function nukeAll() {
        f7.dialog.confirm("Are you sure you want to delete all app data?", async () => {
            await invoke("delete_app_data");
            $identities = {};
            $currentIdentity = "";
        });
    }

    async function generateAndPublishKeyPackage() {
        f7.dialog.confirm(
            "Are you sure you want to generate and publish a key package?",
            async () => {
                await invoke("generate_and_publish_key_package", { pubkey: $currentIdentity });
            }
        );
    }

    async function fetchKeyPackages(): Promise<void> {
        const keyPackageEvents = await $ndk.fetchEvents({
            kinds: [443 as number],
            authors: [$currentIdentity],
        });
        keyPackages = [];
        keyPackageEvents.forEach(async (event: NDKEvent) => {
            console.log(event.rawEvent());
            const keyPackage = await invoke("parse_key_package", { keyPackageHex: event.content });
            keyPackages.push(keyPackage);
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

    <Button onClick={() => (showLogin = !showLogin)} class="flex md:w-1/2 md:mx-auto"
        >Login or create new account</Button
    >
    {#if showLogin}
        <div
            class="flex flex-col gap-12 items-start md:w-1/2 md:mx-auto mt-10 bg-gray-800 ring-1 ring-gray-700 p-4 md:rounded-md"
        >
            <div class="flex flex-col gap-4 items-start w-full">
                <form class="flex flex-col gap-2 w-full items-start">
                    <label
                        for="nsec"
                        class="mb-2 flex flex-col gap-2 text-lg items-start font-medium w-full"
                    >
                        Login in with your nsec
                        <input
                            type="password"
                            id="nsec"
                            bind:value={nsecOrHex}
                            placeholder="nsec1&hellip;"
                            autocapitalize="off"
                            autocorrect="off"
                            class="bg-transparent ring-1 ring-gray-700 rounded-md px-3 py-3 w-full"
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
                </form>
            </div>
            <button
                onclick={createIdentity}
                class="px-3 py-2 text-center flex flex-row items-center gap-2 rounded-md bg-gray-700 hover:bg-gray-600 ring-1 ring-gray-500"
            >
                <PlusCircle size="2rem" weight="thin" />
                Create New Nostr Identity
            </button>
        </div>
    {/if}

    <BlockTitle>Privacy</BlockTitle>
    <List dividers outline mediaList class="privacy-settings-list">
        <ListItem link title="Delete all app data" onClick={nukeAll}>
            <Skull slot="media" size={24} />
        </ListItem>
    </List>

    <BlockTitle>Developer Settings</BlockTitle>
    <List dividers outline mediaList class="developer-settings-list">
        <ListItem link title="Inspect Accounts" onClick={() => (showAccounts = !showAccounts)}>
            <Users slot="media" size={24} />
        </ListItem>
        <ListItem link title="Fetch Prekey Events" onClick={fetchKeyPackages}>
            <Binoculars slot="media" size={24} />
        </ListItem>
        <ListItem link title="Publish Prekey Event" onClick={generateAndPublishKeyPackage}>
            <Key slot="media" size={24} />
        </ListItem>
    </List>

    {#if showAccounts}
        <BlockTitle>Accounts</BlockTitle>
        <div class="p-4 rounded-md bg-gray-800 ring-1 ring-gray-700 mx-4">
            <pre><code class="language-json">{JSON.stringify($identities, null, 4)}</code></pre>
        </div>
    {/if}

    {#if keyPackages.length > 0}
        <BlockTitle>Key Packages</BlockTitle>
        {#each keyPackages as keyPackage}
            <div class="p-4 rounded-md bg-gray-800 ring-1 ring-gray-700 mx-4">
                <pre><code class="language-json">{JSON.stringify(keyPackage, null, 4)}</code></pre>
            </div>
        {/each}
    {/if}
</Page>
