<script lang="ts">
    import OnboardingNumbers from "./OnboardingNumbers.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { getToastState } from "$lib/stores/toast-state.svelte";
    import type { PushView } from "$lib/types/modal";
    import PostOnboard from "./PostOnboard.svelte";
    import { accounts } from "$lib/stores/accounts";

    let toastState = getToastState();

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

    async function publishKeyPackage(): Promise<void> {
        await invoke("publish_key_package", {})
            .then(async () => {
                keyPackagePublished = true;
                await invoke("update_account_onboarding", {
                    pubkey: $accounts.activeAccount,
                    inboxRelays: !!inboxRelaysPublished,
                    keyPackageRelays: !!keyPackageRelaysPublished,
                    publishKeyPackage: true,
                });
                goToPostOnboard();
            })
            .catch((e) => {
                toastState.add("Couldn't publish key package", e, "error");
                console.error(e);
            });
    }

    function goToPostOnboard(): void {
        pushView(PostOnboard, {
            inboxRelaysPublished,
            keyPackageRelaysPublished,
            keyPackagePublished,
        });
    }
</script>

<div class="flex flex-col gap-10 mt-10 items-center w-full md:w-2/3 lg:w-1/2 mx-auto">
    <OnboardingNumbers currentStep={3} {inboxRelaysPublished} {keyPackageRelaysPublished} {keyPackagePublished} />
    <p class="mt-4">
        Finally, we'll need to publish a key package event. This key package event will be used by other users to add
        you to DMs and groups.
    </p>
    <button class="button-primary" onclick={publishKeyPackage}> Publish a key package event </button>
    <button class="button-outline" onclick={goToPostOnboard}> Skip this step </button>
</div>
