<script lang="ts">
import { accounts } from "$lib/stores/accounts";
import { getToastState } from "$lib/stores/toast-state.svelte";
import type { PushView } from "$lib/types/modal";
import { invoke } from "@tauri-apps/api/core";
import { Plus, Trash } from "phosphor-svelte";
import OnboardingNumbers from "./OnboardingNumbers.svelte";
import PublishKeyPackage from "./PublishKeyPackage.svelte";

let toastState = getToastState();

let {
    pushView,
    keyPackageRelaysPublished = $bindable(),
    inboxRelaysPublished = $bindable(),
    keyPackagePublished = $bindable(),
} = $props<{
    pushView: PushView;
    keyPackageRelaysPublished: boolean;
    inboxRelaysPublished: boolean;
    keyPackagePublished: boolean;
}>();

let keyPackageRelays: string[] = $state([
    "wss://relay.damus.io",
    "wss://relay.primal.net",
    "wss://nos.lol",
]);
let newKeyPackageRelay: string = $state("");

function goToKeyPackagePublish(): void {
    pushView(PublishKeyPackage, {
        inboxRelaysPublished,
        keyPackageRelaysPublished,
        keyPackagePublished,
    });
}

async function publishKeyPackageRelays() {
    await invoke("publish_relay_list", {
        relays: keyPackageRelays,
        kind: 10051,
    })
        .then(async () => {
            keyPackageRelaysPublished = true;
            await invoke("update_account_onboarding", {
                pubkey: $accounts.activeAccount,
                inboxRelays: !!inboxRelaysPublished,
                keyPackageRelays: true,
                publishKeyPackage: !!keyPackagePublished,
            });
            goToKeyPackagePublish();
        })
        .catch((e) => {
            toastState.add("Couldn't publish key package relays", e, "error");
            console.error(e);
        });
}
</script>

<div class="flex flex-col gap-10 mt-10 items-center w-full md:w-2/3 lg:w-1/2 mx-auto">
    <OnboardingNumbers currentStep={2} {inboxRelaysPublished} {keyPackageRelaysPublished} {keyPackagePublished} />
    <p class="mt-4">
        Next, we'll need to specify your key package relays. These are the relays where your key packages will be
        published. Unlike inbox relays, key package relays must be readable by everyone.
    </p>
    <div class="w-full">
        <h3 class="text-lg border-b border-gray-700 mb-2 font-medium text-white">Key package relays</h3>
        {#each keyPackageRelays as relay}
            <div class="flex flex-row gap-2">
                <div class="text-white">{relay}</div>
                <button
                    class="button-secondary"
                    onclick={() => (keyPackageRelays = keyPackageRelays.filter((r) => r !== relay))}
                >
                    <Trash size={20} />
                </button>
            </div>
        {/each}
        <div class="flex flex-row gap-2 mt-8">
            <input
                type="text"
                bind:value={newKeyPackageRelay}
                class="w-full bg-transparent border-gray-700 rounded-md"
            />
            <button
                class="button-secondary"
                onclick={() => (keyPackageRelays = [...keyPackageRelays, newKeyPackageRelay])}
            >
                <Plus size={20} />
            </button>
        </div>
    </div>
    <button class="button-primary" onclick={publishKeyPackageRelays}> Publish a new key package relays event </button>
    <button class="button-outline" onclick={goToKeyPackagePublish}> Skip this step </button>
</div>
