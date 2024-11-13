import { writable, type Writable, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";
import type { NMetadata } from "../types/nostr";

export type Accounts = {
    accounts: Account[];
    activeAccount: string | null;
};

export type Account = {
    pubkey: string;
    metadata: NMetadata;
    nostr_relays: string[];
    inbox_relays: string[];
    key_package_relays: string[];
    mls_group_ids: Uint8Array[];
    settings: AccountSettings;
    onboarding: AccountOnboarding;
    last_used: number;
};

export type AccountSettings = {
    darkTheme: boolean;
    devMode: boolean;
    lockdownMode: boolean;
};

export type AccountOnboarding = {
    inbox_relays: boolean;
    key_package_relays: boolean;
    publish_key_package: boolean;
};

type AccountsData = {
    accounts: Record<string, Account>;
    active_account: string;
};

type RelaysData = Record<string, string>;

/** This is an object containing all the logged in accounts and the currently active one */
export const accounts: Writable<Accounts> = writable({} as Accounts);
export const relays: Writable<RelaysData> = writable({} as RelaysData);

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

/** Custom error class for logout-related errors */
export class LogoutError extends Error {
    constructor(message: string) {
        super(message);
        this.name = "LogoutError";
    }
}

export async function setActiveAccount(pubkey: string): Promise<void> {
    if (pubkey === get(accounts).activeAccount) return;
    emit("account_changing", pubkey);
    invoke("set_active_account", { hexPubkey: pubkey }).then(async (accountState) => {
        await updateAccountsStore(accountState as AccountsData);
        await fetchRelays();
    });
}

export async function createAccount(): Promise<void> {
    invoke("create_identity").then(async (accountState) => {
        await updateAccountsStore(accountState as AccountsData);
        await fetchRelays();
    });
}

export async function logout(pubkey: string): Promise<void> {
    const accountState = await invoke("logout", { hexPubkey: pubkey }).catch((e) => {
        if (e === "No accounts exist") {
            throw new LogoutError("No accounts exist");
        } else {
            throw new LogoutError(e);
        }
    });
    await updateAccountsStore(accountState as AccountsData);
    await fetchRelays();
}

export async function login(nsecOrHex: string): Promise<void> {
    if (!nsecOrHex || (!hexPattern.test(nsecOrHex) && !nsecPattern.test(nsecOrHex))) {
        throw new LoginError("Invalid private key");
    }
    const accountState = await invoke("login", { nsecOrHexPrivkey: nsecOrHex });
    await updateAccountsStore(accountState as AccountsData);
    await fetchRelays();
}

export async function updateAccountsStore(accountState?: AccountsData): Promise<void> {
    if (!accountState) {
        accountState = await invoke("get_accounts_state");
    }
    accounts.set({
        accounts: Object.values(accountState!.accounts),
        activeAccount: accountState!.active_account,
    });
}

export async function fetchRelays(): Promise<void> {
    let fetchedRelays: Record<string, string> = await invoke("fetch_relays");
    relays.set(fetchedRelays);
}

export function colorForRelayStatus(status: string): string {
    switch (status) {
        case "Pending":
        case "Initialized":
        case "Connecting":
            return "text-yellow-500";
        case "Connected":
            return "text-green-500";
        case "Disconnected":
        case "Terminated":
            return "text-red-500";
        default:
            return "";
    }
}
