<script lang="ts">
    import { page } from "$app/stores";
    import Sidebar from "$lib/components/Sidebar.svelte";
    import Tabbar from "$lib/components/Tabbar.svelte";
    import { onMount } from "svelte";
    import { updateAccountsStore, accounts, type Account } from "$lib/stores/accounts";
    import { goto } from "$app/navigation";
    import Modal from "$lib/components/Modal.svelte";
    import {
        NumberCircleOne,
        NumberCircleTwo,
        NumberCircleThree,
        Trash,
        Plus,
    } from "phosphor-svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { getToastState } from "$lib/stores/toast-state.svelte";

    let { children } = $props();

    let toastState = getToastState();

    let activeTab = $derived($page.url.pathname.split("/")[1] || "chats");
    let isLoadingAccounts = $state(true);
    let showPreflightModal = $state(false);

    // svelte-ignore non_reactive_update
    let modalComponent: any;

    let inboxRelays = $state(["wss://auth.nostr1.com"]);
    let newInboxRelay = $state("");
    let keyPackageRelays = $state([
        "wss://relay.damus.io",
        "wss://relay.primal.net",
        "wss://nos.lol",
    ]);
    let newKeyPackageRelay = $state("");

    let keyPackagePublished = $state(false);
    let keyPackageRelaysPublished = $state(false);
    let inboxRelaysPublished = $state(false);

    let preflightStages = $state({
        welcome: { order: 0, completed: false },
        inboxRelays: { order: 1, completed: false },
        keyPackageRelays: { order: 2, completed: false },
        keyPackage: { order: 3, completed: false },
    });

    // Update the stage tracking
    let preflightStage = $derived(
        Object.entries(preflightStages)
            .filter(([_, stage]) => !stage.completed)
            .sort((a, b) => a[1].order - b[1].order)[0]?.[0] || "complete"
    );

    // Update the completion status based on the published states
    $effect(() => {
        preflightStages.inboxRelays.completed = inboxRelaysPublished;
        preflightStages.keyPackageRelays.completed = keyPackageRelaysPublished;
        preflightStages.keyPackage.completed = keyPackagePublished;
        if (Object.values(preflightStages).every((stage) => stage.completed)) {
            showPreflightModal = false;
        }
    });

    function startPreflight() {
        preflightStages.welcome.completed = true;
    }

    onMount(async () => {
        await updateAccountsStore();
        isLoadingAccounts = false;

        if (!!!$accounts.activeAccount) {
            goto("/");
        }

        if ($accounts.activeAccount) {
            let activeAccount: Account | undefined = $accounts.accounts.filter(
                (account: Account) => account.pubkey === $accounts.activeAccount
            )[0];
            if (activeAccount) {
                if (activeAccount.inbox_relays.length > 0) {
                    startPreflight();
                    inboxRelaysPublished = true;
                }
                if (activeAccount.key_package_relays.length > 0) {
                    startPreflight();
                    keyPackageRelaysPublished = true;
                }

                invoke("valid_key_package_exists_for_user", {
                    pubkey: activeAccount.pubkey,
                }).then((exists) => {
                    console.log("keyPackageExists", exists);
                    if (exists) {
                        startPreflight();
                        keyPackagePublished = true;
                    }
                });
            }
        }
    });

    async function publishInboxRelays() {
        await invoke("publish_relay_list", {
            relays: inboxRelays,
            kind: 10050,
        })
            .then(() => {
                inboxRelaysPublished = true;
            })
            .catch((e) => {
                toastState.add("Couldn't publish inbox relays", e, "error");
                console.error(e);
            });
    }

    async function publishKeyPackageRelays() {
        await invoke("publish_relay_list", {
            relays: keyPackageRelays,
            kind: 10051,
        })
            .then(() => {
                keyPackageRelaysPublished = true;
            })
            .catch((e) => {
                toastState.add("Couldn't publish key package relays", e, "error");
                console.error(e);
            });
    }

    async function publishKeyPackage() {
        await invoke("publish_key_package", {})
            .then(() => {
                keyPackagePublished = true;
            })
            .catch((e) => {
                toastState.add("Couldn't publish key package", e, "error");
                console.error(e);
            });
    }
</script>

<main class="flex flex-col md:flex-row min-w-96">
    <Sidebar {activeTab} />
    <Tabbar {activeTab} />
    <div class="flex flex-col grow">
        {@render children()}
    </div>
</main>

{#if showPreflightModal}
    <Modal bind:this={modalComponent} bind:showModal={showPreflightModal} title="Let's go">
        <div class="flex flex-col gap-4 justify-start items-center w-2/3 md:w-1/2 mx-auto h-full">
            <div class="flex flex-col gap-4 justify-start items-center">
                {#if preflightStage === "welcome"}
                    <div class="text-7xl">💬</div>
                    <h1 class="text-2xl font-bold text-white">Welcome to White Noise</h1>
                    <h2 class="text-xl font-medium text-gray-300">
                        The most secure chat app on Nostr
                    </h2>
                    <p class=" text-gray-400">
                        Under the hood, we use the Messaging Layer Security protocol (MLS) to ensure
                        end-to-end encryption of your messages; and with no servers to compromise,
                        you can rest assured that your messages are private and secure.
                    </p>
                    <p class="text-gray-400">
                        Let's get you set up so that you can start messaging with friends. This will
                        take less than one minute.
                    </p>
                    <button class="button-primary" onclick={startPreflight}>
                        🤝 Let's get started
                    </button>
                {:else if preflightStage === "inboxRelays"}
                    <div class="flex flex-col gap-4 mt-10 items-center">
                        <div class="flex flex-row gap-4">
                            <NumberCircleOne size={40} class="text-white" />
                            <NumberCircleTwo size={40} class="text-gray-500" />
                            <NumberCircleThree size={40} class="text-gray-500" />
                        </div>
                        First, we'll need to specify your inbox relays. These are the relays where other
                        users can send you messages and only you can read events meant for you.
                    </div>
                    <div class="w-full">
                        <h3 class="text-lg border-b border-gray-700 mb-2 font-medium text-white">
                            Inbox relays
                        </h3>
                        {#each inboxRelays as relay}
                            <div class="flex flex-row gap-2">
                                <div class="text-white">{relay}</div>
                                <button
                                    class="button-secondary"
                                    onclick={() =>
                                        (inboxRelays = inboxRelays.filter((r) => r !== relay))}
                                >
                                    <Trash size={20} />
                                </button>
                            </div>
                        {/each}
                        <div class="flex flex-row gap-2 mt-8">
                            <input
                                type="text"
                                bind:value={newInboxRelay}
                                class="w-full bg-transparent border-gray-700 rounded-md"
                            />
                            <button
                                class="button-secondary"
                                onclick={() => (inboxRelays = [...inboxRelays, newInboxRelay])}
                            >
                                <Plus size={20} />
                            </button>
                        </div>
                    </div>
                    <button class="button-primary w-full" onclick={publishInboxRelays}>
                        Publish a new inbox relays event
                    </button>
                {:else if preflightStage === "keyPackageRelays"}
                    <div class="flex flex-col gap-4 mt-10 items-center">
                        <div class="flex flex-row gap-4">
                            <NumberCircleOne size={40} class="text-green-500" />
                            <NumberCircleTwo size={40} class="text-white" />
                            <NumberCircleThree size={40} class="text-gray-500" />
                        </div>
                        Next, we'll need to specify your key package relays. These are the relays where
                        your key packages will be published. Unlike inbox relays, key package relays
                        must be readable by everyone.
                    </div>
                    <div class="w-full">
                        <h3 class="text-lg border-b border-gray-700 mb-2 font-medium text-white">
                            Key package relays
                        </h3>
                        {#each keyPackageRelays as relay}
                            <div class="flex flex-row gap-2">
                                <div class="text-white">{relay}</div>
                                <button
                                    class="button-secondary"
                                    onclick={() =>
                                        (keyPackageRelays = keyPackageRelays.filter(
                                            (r) => r !== relay
                                        ))}
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
                                onclick={() =>
                                    (keyPackageRelays = [...keyPackageRelays, newKeyPackageRelay])}
                            >
                                <Plus size={20} />
                            </button>
                        </div>
                    </div>
                    <button class="button-primary w-full" onclick={publishKeyPackageRelays}>
                        Publish a new key package relays event
                    </button>
                {:else if preflightStage === "keyPackage"}
                    <div class="flex flex-col gap-4 mt-10 items-center">
                        <div class="flex flex-row gap-4">
                            <NumberCircleOne size={40} class="text-green-500" />
                            <NumberCircleTwo size={40} class="text-green-500" />
                            <NumberCircleThree size={40} class="text-white" />
                        </div>
                        Finally, we'll need to publish a key package event. This key package event will
                        be used by other users to add you to DMs and groups.
                    </div>
                    <button class="button-primary" onclick={publishKeyPackage}>
                        Publish a key package event
                    </button>
                {:else}
                    <div class="flex flex-col gap-4 mt-10 items-center">
                        <div class="text-7xl">🎉</div>
                        Nice work! You're all set up and ready to chat.
                    </div>
                    <button class="button-primary" onclick={() => (showPreflightModal = false)}>
                        Let's go 🤙
                    </button>
                {/if}
            </div>
        </div>
    </Modal>
{/if}
