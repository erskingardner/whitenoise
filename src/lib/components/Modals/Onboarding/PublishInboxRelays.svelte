<script lang="ts">
import { Trash, Plus } from "phosphor-svelte";
import OnboardingNumbers from "./OnboardingNumbers.svelte";
import { invoke } from "@tauri-apps/api/core";
import { getToastState } from "$lib/stores/toast-state.svelte";
import type { PushView } from "$lib/types/modal";
import PublishKeyPackageRelays from "./PublishKeyPackageRelays.svelte";
import { accounts } from "$lib/stores/accounts";

let toastState = getToastState();

let inboxRelays: string[] = $state(["wss://auth.nostr1.com"]);
let newInboxRelay: string = $state("");

let {
    pushView,
    inboxRelaysPublished = $bindable(),
    keyPackageRelaysPublished = $bindable(),
    keyPackagePublished = $bindable(),
} = $props<{
    pushView: PushView;
    inboxRelaysPublished: boolean;
    keyPackageRelaysPublished: boolean;
    keyPackagePublished: boolean;
}>();

function goToKeyPackageRelays(): void {
    pushView(PublishKeyPackageRelays, {
        inboxRelaysPublished,
        keyPackageRelaysPublished,
        keyPackagePublished,
    });
}

async function publishInboxRelays() {
    await invoke("publish_relay_list", {
        relays: inboxRelays,
        kind: 10050,
    })
        .then(async () => {
            inboxRelaysPublished = true;
            await invoke("update_account_onboarding", {
                pubkey: $accounts.activeAccount,
                inboxRelays: true,
                keyPackageRelays: !!keyPackageRelaysPublished,
                publishKeyPackage: !!keyPackagePublished,
            });
            goToKeyPackageRelays();
        })
        .catch((e) => {
            toastState.add("Couldn't publish inbox relays", e, "error");
            console.error(e);
        });
}
</script>

<div class="flex flex-col gap-10 mt-10 items-center w-full md:w-2/3 lg:w-1/2 mx-auto">
    <div>
        <OnboardingNumbers currentStep={1} {inboxRelaysPublished} {keyPackageRelaysPublished} {keyPackagePublished} />
        <p class="mt-4">
            First, we'll need to specify your inbox relays. These are the relays where other users can send you messages
            and only you can read events meant for you.
        </p>
    </div>
    <div class="w-full">
        <h3 class="text-lg border-b border-gray-700 mb-2 font-medium text-white">Inbox relays</h3>
        {#each inboxRelays as relay}
            <div class="flex flex-row gap-2">
                <div class="text-white">{relay}</div>
                <button class="button-secondary" onclick={() => (inboxRelays = inboxRelays.filter((r) => r !== relay))}>
                    <Trash size={20} />
                </button>
            </div>
        {/each}
        <div class="flex flex-row gap-2 mt-8">
            <input type="text" bind:value={newInboxRelay} class="w-full bg-transparent border-gray-700 rounded-md" />
            <button class="button-secondary" onclick={() => (inboxRelays = [...inboxRelays, newInboxRelay])}>
                <Plus size={20} />
            </button>
        </div>
    </div>
    <button class="button-primary" onclick={publishInboxRelays}> Publish a new inbox relays event </button>
    <button class="button-outline" onclick={goToKeyPackageRelays}> Skip this step </button>
</div>
