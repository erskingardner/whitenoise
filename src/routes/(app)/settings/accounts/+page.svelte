<script lang="ts">
import { goto } from "$app/navigation";
import Alert from "$lib/components/Alert.svelte";
import Avatar from "$lib/components/Avatar.svelte";
import Header from "$lib/components/Header.svelte";
import HeaderToolbar from "$lib/components/HeaderToolbar.svelte";
import {
    type Account,
    LogoutError,
    accounts,
    activeAccount,
    createAccount,
    fetchRelays,
    login,
    logout,
    setActiveAccount,
    updateAccountsStore,
} from "$lib/stores/accounts";
import { getToastState } from "$lib/stores/toast-state.svelte";
import { isValidHexPubkey, isValidNsec } from "$lib/types/nostr";
import { copyToClipboard } from "$lib/utils/clipboard";
import { nameFromMetadata, npubFromPubkey } from "$lib/utils/nostr";
import { invoke } from "@tauri-apps/api/core";
import { type UnlistenFn, listen } from "@tauri-apps/api/event";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import {
    isPermissionGranted,
    requestPermission,
    sendNotification,
} from "@tauri-apps/plugin-notification";
import {
    Bell,
    CaretRight,
    CopySimple,
    HardDrives,
    Key,
    PlusCircle,
    SignIn,
    Skull,
    Trash,
    UserPlus,
    Users,
} from "phosphor-svelte";
import { CaretLeft } from "phosphor-svelte";
import { onDestroy, onMount } from "svelte";

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

function goBack() {
    goto("/settings");
}

async function refetchAccount() {
    await invoke("query_enriched_contact", {
        pubkey: $activeAccount?.pubkey,
        updateAccount: true,
    });
}

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
    createAccount("New Account")
        .then(() => {
            toastState.add("Created new account", "Successfully created new account", "success");
        })
        .catch((e) => {
            toastState.add(
                "Error creating account",
                `Failed to create a new account: ${e.message}`,
                "error"
            );
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

async function copyNsec(account: Account) {
    try {
        const nsec = await invoke("export_nsec", { pubkey: account.pubkey });
        if (await copyToClipboard(nsec as string, "nsec")) {
            highlightButton(`[id="nsec-copy-${account.pubkey}"]`);
        } else {
            toastCopyErrorMessage("nsec");
        }
    } catch (e) {
        console.error(e);
        toastState.add(
            "Error exporting nsec",
            "There was an error exporting your nsec, please try again.",
            "error"
        );
    }
}

async function copyNpub(account: Account) {
    const npub = npubFromPubkey(account.pubkey);
    if (await copyToClipboard(npub, "npub")) {
        highlightButton(`[id="npub-copy-${account.pubkey}"]`);
    } else {
        toastCopyErrorMessage("npub");
    }
}

function highlightButton(selector: string) {
    const button = document.querySelector(selector);
    if (!button) return;

    button.classList.add("text-green-500");
    setTimeout(() => {
        button.classList.remove("text-green-500");
    }, 1000);
}

function toastCopyErrorMessage(errorMessage: string) {
    toastState.add(
        `Error copying ${errorMessage}`,
        `There was an error copying your ${errorMessage}, please try again.`,
        "error"
    );
}
</script>

<HeaderToolbar>
    {#snippet left()}
        <button class="flex flex-row gap-0.5 items-end" onclick={goBack}>
            <CaretLeft size={24} />
            <span class="text-xl font-medium">Back</span>
        </button>
    {/snippet}
    {#snippet center()}
        <h1>Accounts</h1>
    {/snippet}
</HeaderToolbar>

<Header title="Accounts" />
<main class="px-4 flex flex-col">
    <div class="section w-full">
        {#each $accounts as account (account.pubkey)}
            <div class="flex flex-row gap-4 items-center border-b border-gray-700 py-3 min-w-0 w-full">
                <div class="flex flex-row items-center flex-1 min-w-0">
                    <button
                      onclick={() => setActiveAccount(account.pubkey)}
                    >
                      <Avatar
                          pubkey={account.pubkey}
                          picture={account.metadata?.picture}
                          pxSize={40}
                          showRing={$activeAccount?.pubkey === account.pubkey}
                      />
                    </button>
                    <div class="flex flex-col gap-1 min-w-0 justify-start text-left pl-4 truncate">
                        <div class="truncate">
                            {nameFromMetadata(account.metadata, account.pubkey)}
                        </div>
                        <div class="flex gap-2 items-center">
                          <p class="font-mono truncate">
                            {npubFromPubkey(account.pubkey)}
                          </p>
                          <button
                            class="transition-colors duration-200"
                            id={`npub-copy-${account.pubkey}`}
                            onclick={() => copyNpub(account)}
                          >
                            <CopySimple size={24} />
                          </button>
                        </div>
                    </div>
                </div>
                <button
                  class="export-nsec-butto min-w-fit text-sm button-outline shrink-0 transition-colors duration-200"
                  id={`nsec-copy-${account.pubkey}`}
                  onclick={() => copyNsec(account)}
                >
                  Copy nsec
                </button>
                <button class="min-w-fit text-sm button-outline shrink-0" onclick={() => handleLogout(account.pubkey)}>
                    Log out
                </button>
            </div>
        {/each}
        <div class="section-list-item !mt-6">
            <button onclick={() => (showLogin = !showLogin)} class="button-primary">
                <UserPlus size={24} />
                Add another account
            </button>
        </div>
        <div class="{showLogin ? 'flex' : 'hidden'} flex-col gap-8 items-start w-full mt-4 py-4">
            <div class="flex flex-col gap-4 items-start w-full">
                <label for="nsec" class="flex flex-col gap-2 text-lg items-start font-medium w-full">
                    Add another account?
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
    <button class="button-primary" onclick={refetchAccount}>Refetch Account</button>
</main>
