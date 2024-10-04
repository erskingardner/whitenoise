import type { NDKCacheAdapter } from "@nostr-dev-kit/ndk";
import NDKCacheAdapterDexie from "@nostr-dev-kit/ndk-cache-dexie";
import { writable } from "svelte/store";
import NDKSvelte from "@nostr-dev-kit/ndk-svelte";

let cacheAdapter: NDKCacheAdapter | undefined = undefined;
cacheAdapter = new NDKCacheAdapterDexie({ dbName: "whitenoise" });

export const ndkStore = new NDKSvelte({
    explicitRelayUrls: [
        // "wss://relay.snort.social",
        // "wss://relay.damus.io",
        // "wss://relay.primal.net",
        // "wss://nos.lol",
        "wss://purplepag.es",
        "ws://localhost:8080",
    ],
    outboxRelayUrls: ["wss://purplepag.es", "ws://localhost:8080"],
    autoConnectUserRelays: true,
    autoFetchUserMutelist: true,
    enableOutboxModel: false,
    cacheAdapter: cacheAdapter,
    clientName: "White Noise",
});

ndkStore.connect().then(() => console.log("NDK Connected"));

// Create a singleton instance that is the default export
const ndk = writable(ndkStore);

export default ndk;
