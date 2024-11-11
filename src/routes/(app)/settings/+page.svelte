<script lang="ts">
    import Alert from "$lib/components/Alert.svelte";
    import Header from "$lib/components/Header.svelte";
    import HeaderToolbar from "$lib/components/HeaderToolbar.svelte";
    import {
        accounts,
        setActiveAccount,
        fetchRelays,
        updateAccountsStore,
        createAccount,
        logout,
        LogoutError,
        login,
    } from "$lib/stores/accounts";
    import Avatar from "$lib/components/Avatar.svelte";
    import { npubFromPubkey, nameFromMetadata } from "$lib/utils/nostr";
    import { isValidNsec, isValidHexPubkey } from "$lib/types/nostr";
    import { goto } from "$app/navigation";
    import { onMount, onDestroy } from "svelte";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import { invoke } from "@tauri-apps/api/core";
    import { getToastState } from "$lib/stores/toast-state.svelte";
    import {
        SignIn,
        PlusCircle,
        Skull,
        Lock,
        CaretRight,
        CaretDown,
        Key,
        Binoculars,
        UserPlus,
        HardDrives,
        UserFocus,
        Envelope,
    } from "phosphor-svelte";

    let showDeleteAlert = $state(false);
    let showKeyPackageAlert = $state(false);
    let showLogin = $state(false);
    let nsecOrHex = $state("");
    let showLoginError = $state(false);
    let loginError = $state("");

    let unlisten: UnlistenFn;

    let toastState = getToastState();

    onMount(async () => {
        if (!unlisten) {
            unlisten = await listen<string>("account_changed", (_event) => {
                updateAccountsStore().then(() => {
                    console.log("account_changed & updateAccountStore from settings page.");
                    fetchRelays();
                });
            });
        }

        fetchRelays();
    });

    onDestroy(() => {
        unlisten?.();
        toastState.cleanup();
    });

    async function handleLogin() {
        if (isValidNsec(nsecOrHex) || isValidHexPubkey(nsecOrHex)) {
            showLoginError = false;
            login(nsecOrHex)
                .then(() => {
                    toastState.add("Logged in", "Successfully logged in", "success");
                    nsecOrHex = "";
                })
                .catch((e) => {
                    console.error(e);
                    showLoginError = true;
                    loginError = "Failed to log in";
                });
        } else {
            showLoginError = true;
            loginError = "Invalid nsec or private key";
        }
    }

    async function handleCreateAccount() {
        showLoginError = false;
        createAccount()
            .then(() => {
                toastState.add("Created new account", "Successfully created new account", "success");
            })
            .catch((e) => {
                toastState.add("Error creating account", `Failed to create a new account: ${e.message}`, "error");
                console.error(e);
            });
    }

    async function handleLogout(pubkey: string): Promise<void> {
        showLoginError = false;
        logout(pubkey)
            .then(() => {
                toastState.add("Logged out", "Successfully logged out", "success");
            })
            .catch((e) => {
                if (e instanceof LogoutError) {
                    goto("/");
                } else {
                    toastState.add("Logout Error", `Failed to log out: ${e.message}`, "error");
                    console.error(e);
                }
            });
    }

    async function deleteAll() {
        showDeleteAlert = true;
    }

    function launchKeyPackage() {
        showKeyPackageAlert = true;
    }

    function publishKeyPackage() {
        invoke("publish_key_package", {})
            .then(() => {
                toastState.add("Key Package Published", "Key Package published successfully", "success");
                showKeyPackageAlert = false;
            })
            .catch((e) => {
                toastState.add(
                    "Error Publishing Key Package",
                    `Failed to publish key package: ${e.toString()}`,
                    "error"
                );
                console.error(e);
            });
    }

    let showAccountsState = $state(false);
    let accountsState = $state("");
    async function toggleInspectAccounts() {
        showAccountsState = !showAccountsState;
        if (showAccountsState) {
            invoke("get_accounts_state").then((accounts) => {
                accountsState = JSON.stringify(accounts, null, 2);
            });
        }
        console.log(accountsState);
    }
</script>

{#if showDeleteAlert}
    <Alert
        title="Delete everything?"
        body="Are you sure you want to delete all data? This cannot be undone."
        acceptFn={() => console.log("delete")}
        acceptText="Burn it all down"
        acceptStyle="warning"
        cancelText="Cancel"
        bind:showAlert={showDeleteAlert}
    />
{/if}

{#if showKeyPackageAlert}
    <Alert
        title="Publish Key Package?"
        body="Are you sure you want to publish a new Key Package event to relays?"
        acceptFn={publishKeyPackage}
        acceptText="Publish New Key Package"
        acceptStyle="primary"
        cancelText="Cancel"
        bind:showAlert={showKeyPackageAlert}
    />
{/if}

<HeaderToolbar>
    {#snippet center()}
        <h1>Settings</h1>
    {/snippet}
</HeaderToolbar>

<Header title="Settings" />
<main class="px-4 flex flex-col pb-40">
    <h2 class="section-title">Accounts</h2>
    <div class="section w-full">
        {#each $accounts.accounts as account (account.pubkey)}
            <div class="flex flex-row gap-4 items-center border-b border-gray-700 py-3 min-w-0 w-full">
                <button
                    class="flex flex-row items-center flex-1 min-w-0"
                    onclick={() => setActiveAccount(account.pubkey)}
                >
                    <Avatar
                        pubkey={account.pubkey}
                        picture={account.metadata?.picture}
                        pxSize={40}
                        showRing={$accounts.activeAccount === account.pubkey}
                    />

                    <div class="flex flex-col gap-1 min-w-0 justify-start text-left pl-4 truncate">
                        <div class="truncate">
                            {nameFromMetadata(account.metadata, account.pubkey)}
                        </div>
                        <div class="font-mono truncate">
                            {npubFromPubkey(account.pubkey)}
                        </div>
                    </div>
                </button>
                <button class="min-w-fit button-outline shrink-0" onclick={() => handleLogout(account.pubkey)}>
                    Log out
                </button>
            </div>
        {/each}
        <div class="section-list-item !mt-6">
            <button onclick={() => (showLogin = !showLogin)} class="button-primary">
                <UserPlus size={24} />
                Log in or create new account
            </button>
        </div>
        <div class="{showLogin ? 'flex' : 'hidden'} flex-col gap-8 items-start w-full mt-4 p-4">
            <div class="flex flex-col gap-4 items-start w-full">
                <label for="nsec" class="flex flex-col gap-2 text-lg items-start font-medium w-full">
                    Log in with your nsec
                    <input
                        type="password"
                        id="nsec"
                        bind:value={nsecOrHex}
                        placeholder="nsec1&hellip;"
                        autocapitalize="off"
                        autocorrect="off"
                        class="w-full px-3 py-2 bg-transparent ring-1 ring-gray-700 rounded-md"
                    />
                </label>
                {#if showLoginError}
                    <span class="text-red-500 text-sm">
                        {loginError}
                    </span>
                {/if}
                <button type="submit" onclick={handleLogin} class="button-primary w-full !justify-start">
                    <SignIn size="20" />
                    Log In
                </button>
            </div>
            <button onclick={handleCreateAccount} class="button-outline w-full !justify-start">
                <PlusCircle size="20" />
                Create New Nostr Identity
            </button>
        </div>
    </div>
    <h2 class="section-title">Privacy & Security</h2>
    <div class="section">
        <ul class="section-list">
            <li class="section-list-item">
                <button onclick={() => goto("/settings/lockdown/")} class="row-button">
                    <Lock size={24} />
                    <span>Lockdown Mode</span>
                    <CaretRight size={24} class="ml-auto mr-0" />
                </button>
            </li>
            <li class="section-list-item">
                <button onclick={deleteAll} class="row-button">
                    <Skull size={24} />
                    <span>Delete all data</span>
                </button>
            </li>
        </ul>
    </div>

    <h2 class="section-title">Developer Settings</h2>
    <div class="section">
        <ul class="section-list">
            <li class="section-list-item">
                <button onclick={() => goto("/settings/network/")} class="row-button">
                    <HardDrives size={24} />
                    <span>Network</span>
                    <CaretRight size={24} class="ml-auto mr-0" />
                </button>
            </li>
            <li class="section-list-item">
                <button onclick={launchKeyPackage} class="row-button">
                    <Key size={24} />
                    <span>Publish Key Package Events</span>
                </button>
            </li>
            <li class="section-list-item">
                <button onclick={() => goto("/settings/key_packages/")} class="row-button">
                    <Binoculars size={24} />
                    <span>Inspect Key Package Events</span>
                    <CaretRight size={24} class="ml-auto mr-0" />
                </button>
            </li>
            <li class="section-list-item">
                <button onclick={() => goto("/settings/invites/")} class="row-button">
                    <Envelope size={24} />
                    <span>Inspect Invites</span>
                    <CaretRight size={24} class="ml-auto mr-0" />
                </button>
            </li>
            <li class="section-list-item">
                <button onclick={toggleInspectAccounts} class="row-button">
                    <UserFocus size={24} />
                    <span>Inspect Accounts</span>
                    {#if showAccountsState}
                        <CaretDown size={24} class="ml-auto mr-0" />
                    {:else}
                        <CaretRight size={24} class="ml-auto mr-0" />
                    {/if}
                </button>
            </li>
        </ul>
        {#if showAccountsState}
            <div class="flex flex-col gap-4 items-start w-full mt-4 p-4">
                <pre>{accountsState}</pre>
            </div>
        {/if}
    </div>
</main>
