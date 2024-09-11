import { get, writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { ndkStore } from "./ndk";

export type Identity = {
    pubkey: string;
};

/** This is an array of hexpubkeys of all signed in identities */
export const identities = writable([] as Identity[]);

/** This is the hexpubkey of the currently selected identity */
export const currentIdentity = writable("");

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
    await invoke("set_current_identity", { pubkey });
    const user = ndkStore.getUser({ pubkey });
    ndkStore.activeUser = user;
    currentIdentity.set(pubkey);
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
    const pubkey = await invoke("create_identity");
    identities.update(currentIdentities => [...currentIdentities, { pubkey } as Identity]);
    currentIdentity.set(await invoke("get_current_identity"));
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
    const ids: string[] = await invoke("get_identities");
    const newIdentities = ids ? ids.map((id: string) => ({ pubkey: id }) as Identity) : [];
    identities.set(newIdentities);
    currentIdentity.set(await invoke("get_current_identity"));
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
export async function login(nsecOrHex: string): Promise<void> {
    if (!nsecOrHex || (!hexPattern.test(nsecOrHex) && !nsecPattern.test(nsecOrHex))) {
        throw new LoginError("Invalid private key");
    }

    const pubkey = await invoke("login", { nsecOrHex });
    if (!pubkey || typeof pubkey !== "string" || pubkey.trim() === "") {
        throw new LoginError("Invalid pubkey returned from login");
    }
    identities.update(currentIdentities => [...currentIdentities, { pubkey } as Identity]);
    currentIdentity.set(await invoke("get_current_identity"));
}

/**
 * Decrypts NIP04 message payload from/for recipient
 *
 * This function invokes the "nip04_decrypt" command on the backend
 * to decrypt a nip04 message and returns the plaintext (or undefined).
 *
 * @param counterparty - The public key of the message counterparty.
 * @param message - The message to decrypt.
 * @returns A promise that resolves with the message plaintext.
 */
export async function nip04Decrypt(counterparty: string, message: string): Promise<string> {
  return invoke('nip04_decrypt', {counterparty, message})
}
