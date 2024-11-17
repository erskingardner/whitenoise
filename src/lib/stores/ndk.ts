import type { NDKCacheAdapter } from "@nostr-dev-kit/ndk";
import NDKCacheAdapterDexie from "@nostr-dev-kit/ndk-cache-dexie";
import { writable } from "svelte/store";
import NDK from "@nostr-dev-kit/ndk";

let cacheAdapter: NDKCacheAdapter | undefined = undefined;
cacheAdapter = new NDKCacheAdapterDexie({ dbName: "whitenoise" });

export const ndkStore = new NDK({
    explicitRelayUrls: [
        "wss://relay.snort.social",
        "wss://relay.damus.io",
        "wss://relay.primal.net",
        "wss://nos.lol",
        "wss://purplepag.es",
        "wss://relay.nostr.band",
    ],
    outboxRelayUrls: ["wss://purplepag.es"],
    autoConnectUserRelays: false,
    autoFetchUserMutelist: false,
    enableOutboxModel: false,
    cacheAdapter: cacheAdapter,
    clientName: "White Noise",
});

ndkStore.connect().then(() => console.log("NDK Connected"));

// Create a singleton instance that is the default export
const ndk = writable(ndkStore);

export default ndk;
