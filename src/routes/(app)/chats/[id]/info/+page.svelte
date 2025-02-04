<script lang="ts">
import { goto } from "$app/navigation";
import { page } from "$app/state";
import Avatar from "$lib/components/Avatar.svelte";
import GroupAvatar from "$lib/components/GroupAvatar.svelte";
import HeaderToolbar from "$lib/components/HeaderToolbar.svelte";
import Name from "$lib/components/Name.svelte";
import { activeAccount } from "$lib/stores/accounts";
import { getToastState } from "$lib/stores/toast-state.svelte";
import { type NostrMlsGroup, NostrMlsGroupType } from "$lib/types/nostr";
import type { EnrichedContact, NEvent } from "$lib/types/nostr";
import { nameFromMetadata } from "$lib/utils/nostr";
import { invoke } from "@tauri-apps/api/core";
import { CaretLeft, Key, LockKey, SignOut, WarningOctagon } from "phosphor-svelte";
import { onDestroy, onMount } from "svelte";

let toastState = getToastState();

let group: NostrMlsGroup | undefined = $state(undefined);
let counterpartyPubkey: string | undefined = $state(undefined);
let enrichedCounterparty: EnrichedContact | undefined = $state(undefined);
let groupName = $state("");
let members: string[] = $state([]);
let admins: string[] = $state([]);
let rotatingKey = $state(false);

$effect(() => {
    if (group && !counterpartyPubkey && !enrichedCounterparty) {
        counterpartyPubkey =
            group.group_type === NostrMlsGroupType.DirectMessage
                ? group.admin_pubkeys.filter((pubkey) => pubkey !== $activeAccount?.pubkey)[0]
                : undefined;
        if (counterpartyPubkey) {
            invoke("query_enriched_contact", {
                pubkey: counterpartyPubkey,
                updateAccount: false,
            }).then((value) => {
                enrichedCounterparty = value as EnrichedContact;
            });
        }
    }

    if (
        group &&
        group.group_type === NostrMlsGroupType.DirectMessage &&
        counterpartyPubkey &&
        enrichedCounterparty
    ) {
        groupName = nameFromMetadata(enrichedCounterparty.metadata, counterpartyPubkey);
    } else if (group) {
        groupName = group.name;
    }
});

async function loadGroup() {
    let groupResponses = Promise.all([
        invoke("get_group", { groupId: page.params.id }),
        invoke("get_group_members", { groupId: page.params.id }),
        invoke("get_group_admins", { groupId: page.params.id }),
    ]);
    let [groupResponse, membersResponse, adminsResponse] = await groupResponses;
    group = groupResponse as NostrMlsGroup;
    members = membersResponse as string[];
    admins = adminsResponse as string[];
}

onMount(async () => {
    await loadGroup();
});

function leaveGroup() {
    console.log("leaveGroup not implemented");
}

function reportSpam() {
    console.log("reportSpam not implemented");
}

async function rotateKey() {
    console.log("rotateKey not implemented");
    // rotatingKey = true;
    // await invoke("rotate_key_in_group", { groupId: page.params.id })
    //     .then(() => {
    //         document.getElementById("rotate-key-icon")?.style.setProperty("color", "green");
    //         setTimeout(() => {
    //             document.getElementById("rotate-key-icon")?.style.setProperty("color", "white");
    //         }, 2000);
    //     })
    //     .catch((e) => {
    //         console.error(e);
    //         toastState.add("Error rotating key", e.split(": ")[2], "error");
    //         rotatingKey = false;
    //         document.getElementById("rotate-key-icon")?.style.setProperty("color", "red");
    //     })
    //     .finally(() => {
    //         rotatingKey = false;
    //     });
}

onDestroy(() => {
    toastState.cleanup();
});
</script>

{#if group}
    <HeaderToolbar>
        {#snippet left()}
            <button onclick={() => goto(`/chats/${page.params.id}`)} class="p-2 -mr-2">
                <CaretLeft size={30} />
            </button>
        {/snippet}
    </HeaderToolbar>
    <div class="flex flex-col items-center justify-center gap-2 p-4 mb-8">
        <GroupAvatar groupType={group.group_type} {groupName} {counterpartyPubkey} {enrichedCounterparty} pxSize={80} />
        <h1 class="text-2xl font-bold">{groupName}</h1>
        <p class="text-gray-500 flex flex-row items-center gap-2">
            <LockKey size={20} />
            {group.description || "A secure chat"}
        </p>
    </div>
    <div class="section mx-6">
        <h2 class="section-title">{members.length} Members</h2>
        <ul class="flex flex-col">
            {#each members as member}
                <li class="flex flex-row items-center gap-4 border-b border-gray-700 py-2 last:border-b-0">
                    <Avatar pubkey={member} />
                    <span class="text-base font-medium"><Name pubkey={member} unstyled={true} /></span>
                    {#if admins.includes(member)}
                        <span class="text-xs font-medium text-white bg-purple-600 outline outline-1 outline-purple-300/50 px-2 pt-0.5 rounded-full">Admin</span>
                    {/if}
                </li>
            {/each}
        </ul>
    </div>
    <div class="section mx-6">
        <div class="flex flex-col items-center gap-0">
            <button class="flex flex-row items-center gap-4 py-3 w-full border-b border-gray-700 last:border-b-0" onclick={rotateKey}><Key size={24} class="transition-all duration-300 ease-in-out {rotatingKey ? 'animate-spin': ''}" id="rotate-key-icon" />Rotate Your Key</button>
            <button class="text-red-500 flex flex-row items-center gap-4 py-3 w-full border-b border-gray-700 last:border-b-0" onclick={leaveGroup}><SignOut size={24} />Leave Group</button>
            <button class="text-red-500 flex flex-row items-center gap-4 py-3 w-full border-b border-gray-700 last:border-b-0" onclick={reportSpam}><WarningOctagon size={24} />Report Spam</button>
        </div>
    </div>
{/if}

