<script lang="ts">
import { createAccount, login } from "$lib/stores/accounts";
import { getToastState } from "$lib/stores/toast-state.svelte";
import { isValidHexPubkey, isValidNsec } from "$lib/types/nostr";
import { PlusCircle, SignIn, UserPlus } from "phosphor-svelte";

let { closeModal } = $props<{
    closeModal: () => void;
}>();

let toastState = getToastState();

let nsecOrHex: string = $state("");
let showLoginError: boolean = $state(false);
let loginError: string = $state("");
let name: string = $state("");

async function handleLogin() {
    if (isValidNsec(nsecOrHex) || isValidHexPubkey(nsecOrHex)) {
        showLoginError = false;
        login(nsecOrHex)
            .then(() => {
                toastState.add("Logged in", "Successfully logged in", "success");
                nsecOrHex = "";
                setTimeout(() => {
                    closeModal();
                }, 100);
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
    createAccount(name)
        .then(() => {
            toastState.add("Created new account", "Successfully created new account", "success");
            setTimeout(() => {
                closeModal();
            }, 100);
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
</script>

<div>
    <div class="flex flex-col gap-8 items-start w-full mt-4 py-4">
        <div class="flex flex-col gap-4 items-start w-full bg-gray-900/50 ring-1 ring-gray-600 shadow-lg p-4 rounded-md">
            <h2 class="text-lg font-medium flex flex-row gap-3 items-center">
                <SignIn size="32" weight="fill" />
                Log in with an existing Nostr identity
            </h2>
            <label for="name" class="flex flex-col gap-2 items-start font-medium w-full text-gray-400">
                <span>Nostr Private Key</span>
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
            <button type="submit" onclick={handleLogin} class="button-primary w-full !justify-start !w-auto">
                <SignIn size="20" />
                Log In
            </button>
        </div>
        <div class="flex flex-col gap-4 items-start w-full bg-gray-900/50 ring-1 ring-gray-600 shadow-lg p-4 rounded-md">
            <h2 class="text-lg font-medium flex flex-row gap-3 items-center">
                <UserPlus size="32" weight="fill" />
                Create a new Nostr identity
            </h2>
            <label for="name" class="flex flex-col gap-2 items-start font-medium w-full text-gray-400">
                <span>Display Name</span>
                <input
                    type="text"
                    id="name"
                    bind:value={name}
                    placeholder="What should we call you?"
                    autocapitalize="off"
                    autocorrect="off"
                    class="w-full px-3 py-2 bg-transparent ring-1 ring-gray-700 rounded-md"
                />
            </label>
            <button onclick={handleCreateAccount} class="px-3 py-1.5 text-center flex flex-row shrink items-center justify-center gap-2 ring-1 bg-indigo-700 hover:bg-indigo-600 rounded-md ring-1 ring-indigo-500">
                <PlusCircle size="20" />
                Create New Nostr Identity
            </button>
        </div>
    </div>
</div>
