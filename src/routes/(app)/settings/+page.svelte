<script lang="ts">
import { goto } from "$app/navigation";
import Alert from "$lib/components/Alert.svelte";
import Avatar from "$lib/components/Avatar.svelte";
import Header from "$lib/components/Header.svelte";
import HeaderToolbar from "$lib/components/HeaderToolbar.svelte";
import SettingsHeader from "$lib/components/SettingsHeader.svelte";
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
    SignOut,
    Skull,
    Trash,
    User,
    UserPlus,
} from "phosphor-svelte";
import { onDestroy, onMount } from "svelte";

let showDeleteAlert = $state(false);
let showKeyPackageAlert = $state(false);
let showDeleteKeyPackagesAlert = $state(false);
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

async function testNotification() {
    let permissionGranted = await isPermissionGranted();

    if (!permissionGranted) {
        permissionGranted = "granted" === (await requestPermission());
    }
    if (permissionGranted) {
        sendNotification({
            title: "White Noise",
            body: "Notification test successful!",
        });
    }
}

async function deleteAll() {
    showDeleteAlert = true;
}

function launchKeyPackage() {
    showKeyPackageAlert = true;
}

function deleteAllKeyPackages() {
    showDeleteKeyPackagesAlert = true;
}

function publishKeyPackage() {
    invoke("publish_new_key_package", {})
        .then(() => {
            toastState.add(
                "Key Package Published",
                "Key Package published successfully",
                "success"
            );
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
}

let showGroupsState = $state(false);
let groupsState = $state("");
async function toggleInspectGroups() {
    showGroupsState = !showGroupsState;
    if (showGroupsState) {
        invoke("get_groups").then((groups) => {
            groupsState = JSON.stringify(groups, null, 2);
        });
    }
}

let showInvitesState = $state(false);
let invitesState = $state("");
async function toggleInspectInvites() {
    showInvitesState = !showInvitesState;
    if (showInvitesState) {
        invoke("get_invites").then((invites) => {
            invitesState = JSON.stringify(invites, null, 2);
        });
    }
}

async function handleLogout(): Promise<void> {
    if (!$activeAccount) {
        return;
    }
    showLoginError = false;
    logout($activeAccount.pubkey)
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
</script>

{#if showDeleteAlert}
    <Alert
        title="Delete everything?"
        body="This will delete all group and message data, and sign you out of all accounts. This will not delete your nostr keys or any other events you've published to relays. Are you sure you want to delete all data from White Noise? This cannot be undone."
        acceptFn={async () => {
            invoke("delete_all_data")
                .then(() => {
                    toastState.add("Data deleted", "All accounts, groups, and messages have been deleted.", "info");
                    showDeleteAlert = false;
                    goto("/login");
                })
                .catch((e) => {
                    toastState.add("Error deleting data", `Failed to delete data: ${e.toString()}`, "error");
                    console.error(e);
                });
        }}
        acceptText="Yes, delete everything"
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
        acceptText="Publish Key Package"
        acceptStyle="primary"
        cancelText="Cancel"
        bind:showAlert={showKeyPackageAlert}
    />
{/if}

{#if showDeleteKeyPackagesAlert}
    <Alert
        title="Delete All Key Packages?"
        body="Are you sure you want to send delete requests to all relays where your key packages are found?"
        acceptFn={async () => {
            invoke("delete_all_key_packages")
                .then(() => {
                    toastState.add("Key Packages Deleted", "All key packages have been deleted.", "success");
                    showDeleteKeyPackagesAlert = false;
                })
                .catch((e) => {
                    toastState.add("Error Deleting Key Packages", `Failed to delete key packages: ${e.toString()}`, "error");
                    console.error(e);
                });
        }}
        acceptText="Yes, delete all key packages"
        acceptStyle="warning"
        cancelText="Cancel"
        bind:showAlert={showDeleteKeyPackagesAlert}
    />
{/if}

<HeaderToolbar>
    {#snippet center()}
        <h1>Settings</h1>
    {/snippet}
</HeaderToolbar>

<SettingsHeader />
<main class="px-4 flex flex-col pb-32">
    <div class="section">
        <ul class="section-list">
            <li class="section-list-item">
                <button onclick={() => goto("/settings/profile/")} class="row-button">
                    <User size={24} class="shrink-0" />
                    <span>Profile</span>
                    <CaretRight size={24} class="ml-auto mr-0 shrink-0" />
                </button>
            </li>
            <li class="section-list-item">
                <button onclick={() => goto("/settings/network/")} class="row-button">
                    <HardDrives size={24} class="shrink-0" />
                    <span>Network</span>
                    <CaretRight size={24} class="ml-auto mr-0 shrink-0" />
                </button>
            </li>
            <li class="section-list-item">
                <button onclick={handleLogout} class="row-button">
                    <SignOut size={24} class="shrink-0" />
                    <span class="truncate">Sign out</span>
                </button>
            </li>
        </ul>
    </div>
    <h2 class="section-title">Privacy & Security</h2>
    <div class="section">
        <ul class="section-list">
            <!-- <li class="section-list-item">
                <button onclick={() => goto("/settings/lockdown/")} class="row-button">
                    <Lock size={24} />
                    <span>Lockdown Mode</span>
                    <CaretRight size={24} class="ml-auto mr-0" />
                </button>
            </li> -->
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
                <button onclick={launchKeyPackage} class="row-button">
                    <Key size={24} />
                    <span>Publish a Key Package</span>
                </button>
            </li>
            <li class="section-list-item">
                <button onclick={deleteAllKeyPackages} class="row-button">
                    <Trash size={24} class="shrink-0" />
                    <span class="truncate">Send delete requests for all key packages</span>
                </button>
            </li>

            <li class="section-list-item">
                <button onclick={testNotification} class="row-button">
                    <Bell size={24} class="shrink-0" />
                    <span class="truncate">Test Notification</span>
                </button>
            </li>

            <!-- <li class="section-list-item">
                <button onclick={toggleInspectAccounts} class="row-button">
                    <UserFocus size={24} />
                    <span>Inspect Accounts</span>
                    {#if showAccountsState}
                        <CaretDown size={24} class="ml-auto mr-0" />
                    {:else}
                        <CaretRight size={24} class="ml-auto mr-0" />
                    {/if}
                </button>
                {#if showAccountsState}
                    <div class="flex flex-col gap-4 items-start w-full mt-4 p-4">
                        <pre class="whitespace-pre overflow-x-auto w-full">{accountsState}</pre>
                    </div>
                {/if}
            </li>
            <li class="section-list-item">
                <button onclick={toggleInspectInvites} class="row-button">
                    <Envelope size={24} />
                    <span>Inspect Invites</span>
                    {#if showInvitesState}
                        <CaretDown size={24} class="ml-auto mr-0" />
                    {:else}
                        <CaretRight size={24} class="ml-auto mr-0" />
                    {/if}
                </button>
                {#if showInvitesState}
                    <div class="flex flex-col gap-4 items-start w-full mt-4 p-4">
                        <pre class="whitespace-pre overflow-x-auto w-full">{invitesState}</pre>
                    </div>
                {/if}
            </li>
            <li class="section-list-item">
                <button onclick={toggleInspectGroups} class="row-button">
                    <Users size={24} />
                    <span>Inspect Groups</span>
                    {#if showGroupsState}
                        <CaretDown size={24} class="ml-auto mr-0" />
                    {:else}
                        <CaretRight size={24} class="ml-auto mr-0" />
                    {/if}
                </button>
                {#if showGroupsState}
                    <div class="flex flex-col gap-4 items-start w-full mt-4 p-4">
                        <pre class="whitespace-pre overflow-x-auto w-full">{groupsState}</pre>
                    </div>
                {/if}
            </li> -->
        </ul>
    </div>
</main>
