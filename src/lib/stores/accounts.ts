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

export async function setActiveAccount(pubkey: string): Promise<void> {
    if (
        !get(accounts)
            .map((account) => account.pubkey)
            .includes(pubkey) ||
        pubkey === get(activeAccount)?.pubkey
    )
        return;
    emit("account_changing", pubkey);
    invoke("set_active_account", { hexPubkey: pubkey }).then(async (account) => {
        activeAccount.set(account as Account);
        await fetchRelays();
    });
}

export async function createAccount(): Promise<void> {
    invoke("create_identity").then(async (account) => {
        activeAccount.set(account as Account);
        await fetchRelays();
    });
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
    const [accountsResp, activeAccountResp] = await Promise.all([
        invoke("get_accounts").catch((_) => {
            return [];
        }),
        invoke("get_active_account").catch((_) => {
            return null;
        }),
    ]);
    accounts.set(accountsResp as Account[]);
    activeAccount.set(activeAccountResp as Account | null);
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
