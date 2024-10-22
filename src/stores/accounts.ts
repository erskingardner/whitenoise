import { writable, type Writable, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import type { NUsers } from "../types/nostr";
import { ndkStore } from "../stores/ndk";

export type Accounts = {
    accounts: NUsers;
    current_identity: string | null;
};

/** This is an object containing the hexpubkeys and metadata of all signed in identities */
export const identities: Writable<NUsers> = writable({} as NUsers);

/** This is the hexpubkey of the currently selected identity */
export const currentIdentity: Writable<string | null> = writable(null);

/** Basic matching patterns for hex and nsec keys */
export const hexPattern = /^[a-fA-F0-9]{64}$/;
export const nsecPattern = /^nsec1[a-zA-Z0-9]{58}$/;

/** Custom error class for login-related errors */
export class LoginError extends Error {
    constructor(message: string) {
        super(message);
        this.name = "LoginError";
    }
}

/**
 * Fetches the accounts from the backend and updates the stores.
 *
 * This function performs the following steps:
 * 1. Invokes the "get_accounts" command on the backend to retrieve the accounts.
 * 2. Updates the identities store with the fetched identities.
 * 3. Updates the currentIdentity store with the fetched current identity.
 *
 * If the backend returns empty values, it defaults to an empty object for identities
 * and an empty string for currentIdentity.
 *
 * @returns A promise that resolves when the accounts are fetched and stores are updated.
 */
export async function fetchAccounts(): Promise<void> {
    const accounts: Accounts = await invoke("get_accounts");
    updateIdentities(accounts);
}

/**
 * Switches the current identity to the specified public key.
 *
 * This function invokes the "set_current_identity" command on the backend
 * to update the current identity, and then updates the currentIdentity store
 * in the frontend to reflect this change.
 *
 * @param pubkey - The public key of the identity to switch to.
 * @returns A promise that resolves when the identity switch is complete.
 */
export async function switchIdentity(pubkey: string): Promise<void> {
    if (pubkey === get(currentIdentity)) return;
    const accounts: Accounts = await invoke("set_current_identity", { pubkey });
    console.log("switchIdentity - accounts", accounts);
    updateIdentities(accounts);
}

/**
 * Creates a new identity and updates the stores.
 *
 * This function performs the following steps:
 * 1. Invokes the "create_identity" command on the backend to generate a new identity.
 * 2. Updates the identities store by adding the newly created identity.
 * 3. Retrieves the current identity from the backend and updates the currentIdentity store.
 *
 * @returns A promise that resolves when the new identity is created and stores are updated.
 */
export async function createIdentity(): Promise<void> {
    const accounts: Accounts = await invoke("create_identity");
    updateIdentities(accounts);
}

/**
 * Logs out the specified identity and updates the stores.
 *
 * This function performs the following steps:
 * 1. Invokes the "logout" command on the backend to log out the specified identity.
 * 2. Retrieves the updated list of identities from the backend.
 * 3. Updates the identities store with the new list of identities.
 * 4. Updates the currentIdentity store with the new current identity from the backend.
 *
 * @param pubkey - The public key of the identity to log out.
 * @returns A promise that resolves when the logout process is complete and stores are updated.
 */
export async function logout(pubkey: string): Promise<void> {
    await invoke("logout", { pubkey });
    const accounts: Accounts = await invoke("get_accounts");
    updateIdentities(accounts);
}

/**
 * Logs in with a private key and updates the stores.
 *
 * This function performs the following steps:
 * 1. Validates the input private key (nsec or hex format).
 * 2. Invokes the "login" command on the backend with the provided private key.
 * 3. Validates the returned public key.
 * 4. Updates the identities store by adding the newly logged-in identity.
 * 5. Updates the currentIdentity store with the new current identity from the backend.
 *
 * @param nsecOrHex - The private key in nsec or hex format.
 * @throws {LoginError} If the private key is invalid or the login process fails.
 * @returns A promise that resolves when the login process is complete and stores are updated.
 */
export async function login(nsecOrHex: string, source: string): Promise<void> {
    if (!nsecOrHex || (!hexPattern.test(nsecOrHex) && !nsecPattern.test(nsecOrHex))) {
        throw new LoginError("Invalid private key");
    }

    const accounts: Accounts = await invoke("login", { nsecOrHex, source });
    updateIdentities(accounts);
}

/**
 * Updates the identities and currentIdentity stores based on the provided Accounts object.
 *
 * This function performs the following actions:
 * 1. Sets the identities store with the identities from the provided Accounts object,
 *    or an empty object if no identities are present.
 * 2. Sets the currentIdentity store with the currentIdentity from the provided Accounts object,
 *    or an empty string if no current identity is set.
 *
 * @param accounts - The Accounts object containing updated identity information.
 */
export function updateIdentities(accounts: Accounts): void {
    identities.set(accounts.accounts || {});
    currentIdentity.set(accounts.current_identity);

    console.log("updateIdentities: accounts", get(identities));
    console.log("updateIdentities: currentIdentity", get(currentIdentity));

    if (accounts.current_identity) {
        ndkStore.activeUser = ndkStore.getUser({ pubkey: accounts.current_identity });
    }
}
