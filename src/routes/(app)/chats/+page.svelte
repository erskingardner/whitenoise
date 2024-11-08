<script lang="ts">
    import Header from "$lib/components/Header.svelte";
    import HeaderToolbar from "$lib/components/HeaderToolbar.svelte";
    import { PlusCircle } from "phosphor-svelte";
    import Modal from "$lib/components/Modal.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import type { EnrichedContactsMap, NostrMlsGroup, Invite, NMetadata } from "$lib/types/nostr";
    import { onMount, onDestroy, createRawSnippet } from "svelte";
    import Name from "$lib/components/Name.svelte";
    import Avatar from "$lib/components/Avatar.svelte";
    import { nameFromMetadata, npubFromPubkey } from "$lib/utils/nostr";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import Loader from "$lib/components/Loader.svelte";
    import { accounts } from "$lib/stores/accounts";
    import { getToastState } from "$lib/stores/toast-state.svelte";
    import type { EnrichedContact } from "$lib/types/nostr";
    import { CaretLeft, CaretRight } from "phosphor-svelte";

    let unlistenAccountChanging: UnlistenFn;
    let unlistenAccountChanged: UnlistenFn;
    let unlistenNostrReady: UnlistenFn;

    // svelte-ignore non_reactive_update
    let modalComponent: any;

    let toastState = getToastState();

    let createGroupData = $state<{ pubkey: string; contact: EnrichedContact } | null>(null);
    let inviteeName = $derived.by(() => {
        if (createGroupData?.contact) {
            return nameFromMetadata(createGroupData?.contact.metadata, createGroupData?.pubkey);
        }
        return "";
    });

    let showModal = $state(false);
    let contacts = $state<EnrichedContactsMap>({});
    let search = $state("");
    let filteredContacts = $state<EnrichedContactsMap>({});

    let isLoading = $state(true);
    let loadingError = $state<string | null>(null);

    let groups = $state<NostrMlsGroup[]>([]);
    let invites = $state<Invite[]>([]);

    let modalViews = $state(["contacts"]); // Stack of views
    let currentView = $derived(modalViews[modalViews.length - 1]);

    function pushView(view: string, pubkey: string, contact: EnrichedContact) {
        createGroupData = { pubkey, contact };
        modalViews = [...modalViews, view];
    }

    function popView() {
        if (modalViews.length > 1) {
            modalViews = modalViews.slice(0, -1);
        }
    }

    async function loadEvents() {
        isLoading = true;
        try {
            const [groupsResponse, invitesResponse, contactsResponse] = await Promise.all([
                invoke("get_groups"),
                invoke("get_invites"),
                invoke("fetch_enriched_contacts"),
            ]);

            groups = groupsResponse as NostrMlsGroup[];
            invites = invitesResponse as Invite[];

            // Sort contacts by name
            contacts = Object.fromEntries(
                Object.entries(contactsResponse as EnrichedContactsMap).sort(
                    ([_keyA, contactA], [_keyB, contactB]) => {
                        const nameA =
                            contactA.metadata.display_name || contactA.metadata.name || "";
                        const nameB =
                            contactB.metadata.display_name || contactB.metadata.name || "";
                        return nameA.localeCompare(nameB);
                    }
                )
            );
        } catch (error) {
            loadingError = error as string;
        } finally {
            isLoading = false;
        }
    }

    $effect(() => {
        if (search === "") {
            filteredContacts = contacts;
        } else {
            filteredContacts = Object.fromEntries(
                Object.entries(contacts).filter(
                    ([pubkey, contact]) =>
                        contact.metadata.name?.toLowerCase().includes(search.toLowerCase()) ||
                        contact.metadata.display_name
                            ?.toLowerCase()
                            .includes(search.toLowerCase()) ||
                        pubkey.toLowerCase().includes(search.toLowerCase())
                )
            );
        }
    });

    async function createGroup() {
        if (!createGroupData || !createGroupData.contact.metadata || !createGroupData.pubkey)
            return;

        console.log("Creating group with", inviteeName);

        invoke("create_group", {
            creatorPubkey: $accounts.activeAccount,
            memberPubkeys: [createGroupData.pubkey],
            adminPubkeys: [$accounts.activeAccount, createGroupData.pubkey],
            groupName: "Secure DM",
            description: "",
        })
            .then((group) => {
                console.log("Group created", group);
                createGroupData = null;
                showModal = false;
            })
            .catch((e) => {
                toastState.add("Error creating group", e.toString(), "error");
                console.error("Error creating group", e);
            });
    }

    onMount(async () => {
        await loadEvents();

        if (!unlistenAccountChanging) {
            unlistenAccountChanging = await listen<string>("account_changing", async (_event) => {
                console.log("Event received on chats page: account_changing");
                isLoading = true;
                contacts = {};
                filteredContacts = {};
                groups = [];
                invites = [];
                search = "";
            });
        }

        if (!unlistenAccountChanged) {
            unlistenAccountChanged = await listen<string>("account_changed", async (_event) => {
                console.log("Event received on chats page: account_changed");
            });
        }

        if (!unlistenNostrReady) {
            unlistenNostrReady = await listen<string>("nostr_ready", async (_event) => {
                console.log("Event received on chats page: nostr_ready");
                await loadEvents();
            });
        }
    });

    onDestroy(() => {
        unlistenAccountChanging?.();
        unlistenAccountChanged?.();
        unlistenNostrReady?.();
        toastState.cleanup();
    });

    const contactDetailsView = createRawSnippet(() => {
        return {
            render: () => `
        <div class="p-4">
                            <p>
                                White Noise uses MLS (messaging layer security). Your messages are
                                end-to-end encrypted and can only be read by you and the other
                                participant.
                            </p>
                            <p class="mt-4">
                                Ready to invite {inviteeName} to start a secure chat?
                            </p>
                            <button
                                class="w-full py-2 px-4 bg-blue-600 rounded-md mt-4"
                                id="create-group-button"
                            >
                                Start secure chat
                            </button>
                        </div>
        `,
            setup: (element: Element) => {
                const button = element.querySelector("#create-group-button");
                button?.addEventListener("click", () => {
                    console.log("Create group button clicked");
                    // createGroup();
                });
            },
        };
    });
</script>

<HeaderToolbar>
    {#snippet right()}
        <div>
            <button onclick={() => (showModal = !showModal)} class="p-2 -mr-2">
                <PlusCircle size={30} />
            </button>
        </div>
    {/snippet}
    {#snippet center()}
        <h1>Chats</h1>
    {/snippet}
</HeaderToolbar>

<Header title="Chats" />
<main class="">
    {#if isLoading}
        <div class="flex justify-center items-center mt-20 w-full">
            <Loader size={40} fullscreen={false} />
        </div>
    {:else if loadingError}
        <div class="text-red-500 px-4 font-medium flex flex-col gap-2">
            <span>Sorry, we couldn't load your chats because of an error.</span>
            <pre class="font-mono p-2 rounded-md ring-1 ring-red-500/30">{loadingError}</pre>
        </div>
    {:else}
        <div class="px-4 py-2 bg-gray-800 text-lg font-bold border-t border-b border-gray-700">
            Invites
        </div>
        <div class="flex flex-col gap-2">
            {#each invites as invite}
                <div class="border-b border-gray-700 py-4">{invite.group_name}</div>
            {/each}
        </div>
        <div class="px-4 py-2 bg-gray-800 text-lg font-bold border-t border-b border-gray-700">
            Groups
        </div>
        <div class="flex flex-col gap-2">
            {#each groups as group}
                <div class="border-b border-gray-700 py-4">{group.name}</div>
            {/each}
        </div>
    {/if}
</main>

{#snippet contactsView("Contacts")}
    <div>Contacts view</div>
{/snippet}

{#if showModal}
    <Modal bind:this={modalComponent} title={"Contacts"} bind:showModal>
        <div class="relative h-[90vh] overflow-x-hidden">
            {#key currentView}
                <div class="w-full absolute inset-0">
                    {#if currentView === "contacts"}
                        <input
                            type="search"
                            placeholder="Search..."
                            bind:value={search}
                            class="bg-transparent ring-1 ring-gray-700 rounded-md px-3 py-2 w-full"
                        />
                        <div class="flex flex-col">
                            {#each Object.entries(filteredContacts) as [pubkey, contact]}
                                <button
                                    onclick={() =>
                                        modalComponent.pushView(
                                            "Start secure chat",
                                            contactDetailsView,
                                            { pubkey, contact }
                                        )}
                                    class="flex flex-row gap-2 items-center px-2 py-3 border-b border-gray-700 hover:bg-gray-700"
                                >
                                    <Avatar
                                        {pubkey}
                                        picture={contact.metadata.picture}
                                        pxSize={40}
                                    />
                                    <div class="flex flex-col items-start justify-start truncate">
                                        <Name {pubkey} metadata={contact.metadata} />
                                        <span class="text-gray-400 text-sm font-mono"
                                            >{npubFromPubkey(pubkey)}</span
                                        >
                                    </div>
                                    <CaretRight size={20} class="ml-auto" />
                                </button>
                            {/each}
                        </div>
                    {:else if currentView === "contact-details"}{/if}
                </div>
            {/key}
        </div>
    </Modal>
{/if}
