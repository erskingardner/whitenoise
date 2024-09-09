import { browser } from "$app/environment";
import type { NDKCacheAdapter } from "@nostr-dev-kit/ndk";
import NDKCacheAdapterDexie from "@nostr-dev-kit/ndk-cache-dexie";
import NDK from "@nostr-dev-kit/ndk";
import { writable } from "svelte/store";

let cacheAdapter: NDKCacheAdapter | undefined;

if (browser) {
    cacheAdapter = new NDKCacheAdapterDexie({ dbName: "whitenoise" });
}

export const ndkStore = new NDK({
    explicitRelayUrls: [
        "wss://purplepag.es",
        "wss://relay.nostr.band",
        "wss://nos.lol",
        "wss://relay.snort.social",
        "wss://relay.damus.io",
        "wss://relay.primal.net",
    ],
    outboxRelayUrls: ["wss://purplepag.es", "wss://relay.primal.net"],
    autoConnectUserRelays: true,
    autoFetchUserMutelist: true,
    enableOutboxModel: true,
    cacheAdapter: cacheAdapter,
    clientName: "White Noise",
});

ndkStore.connect().then(() => console.log("NDK Connected"));

// Create a singleton instance that is the default export
const ndk = writable(ndkStore);

export default ndk;
