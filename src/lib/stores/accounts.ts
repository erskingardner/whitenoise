import { invoke } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";
import { type Writable, get, writable } from "svelte/store";
import type { NMetadata } from "../types/nostr";

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
    active: boolean;
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

type RelaysData = Record<string, string>;

export const accounts: Writable<Account[]> = writable([]);
export const activeAccount: Writable<Account | null> = writable(null);
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

/** Custom error class for signup-related errors */
export class SignupError extends Error {
    constructor(message: string) {
        super(message);
        this.name = "SignupError";
    }
}

export async function setActiveAccount(pubkey: string): Promise<void> {
    if (
        !get(accounts)
            .map((account) => account.pubkey)
            .includes(pubkey) ||
        pubkey === get(activeAccount)?.pubkey
    )
        return;
    emit("account_changing", pubkey);
    return invoke("set_active_account", { hexPubkey: pubkey }).then(async (account) => {
        activeAccount.set(account as Account);
        await fetchRelays();
    });
}

export async function createAccount(name: string): Promise<void> {
    const account = await invoke("create_identity", { name }).catch((e) => {
        throw new SignupError(e);
    });
    activeAccount.set(account as Account);
    await fetchRelays();
}

export async function logout(pubkey: string): Promise<void> {
    await invoke("logout", { hexPubkey: pubkey }).catch((e) => {
        if (e === "No account found") {
            throw new LogoutError("No account found");
        }
        throw new LogoutError(e);
    });
    await updateAccountsStore();
    await fetchRelays();
}

export async function login(nsecOrHex: string): Promise<void> {
    if (!nsecOrHex || (!hexPattern.test(nsecOrHex) && !nsecPattern.test(nsecOrHex))) {
        throw new LoginError("Invalid private key");
    }
    await invoke("login", { nsecOrHexPrivkey: nsecOrHex });
    await updateAccountsStore();
    await fetchRelays();
}

export async function updateAccountsStore(): Promise<void> {
    return invoke("get_accounts")
        .then((accountsResp) => {
            const sortedAccounts = (accountsResp as Account[]).sort((a, b) =>
                a.pubkey.localeCompare(b.pubkey)
            );
            accounts.set(sortedAccounts);
            const currentActiveAccount = sortedAccounts.find((account) => account.active) || null;
            activeAccount.set(currentActiveAccount);
        })
        .catch((_) => {
            accounts.set([]);
            activeAccount.set(null);
        });
}

export async function fetchRelays(): Promise<void> {
    const fetchedRelays: Record<string, string> = await invoke("fetch_relays");
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
