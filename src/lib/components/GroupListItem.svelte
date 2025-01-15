<script lang="ts">
import { activeAccount } from "$lib/stores/accounts";
import { type NostrMlsGroup, NostrMlsGroupType } from "$lib/types/nostr";
import type { EnrichedContact } from "$lib/types/nostr";
import { hexMlsGroupId } from "$lib/utils/group";
import { invoke } from "@tauri-apps/api/core";
import { Checks, LockKey } from "phosphor-svelte";
import { nameFromMetadata } from "../utils/nostr";
import GroupAvatar from "./GroupAvatar.svelte";

let { group } = $props<{
    group: NostrMlsGroup;
}>();

let counterpartyPubkey: string | undefined = $derived(
    group.group_type === NostrMlsGroupType.DirectMessage
        ? group.admin_pubkeys.filter((pubkey: string) => pubkey !== $activeAccount?.pubkey)[0]
        : undefined
);

let enrichedCounterparty: EnrichedContact | undefined = $state(undefined);
let groupName = $state("");

$effect(() => {
    if (counterpartyPubkey) {
        invoke("query_enriched_contact", {
            pubkey: counterpartyPubkey,
            updateAccount: false,
        }).then((value) => {
            enrichedCounterparty = value as EnrichedContact;
        });
    }
});

$effect(() => {
    if (
        group.group_type === NostrMlsGroupType.DirectMessage &&
        counterpartyPubkey &&
        enrichedCounterparty
    ) {
        groupName = nameFromMetadata(
            (enrichedCounterparty as EnrichedContact).metadata,
            counterpartyPubkey
        );
    } else {
        groupName = group.name;
    }
});
</script>

<a
    href={`/chats/${hexMlsGroupId(group.mls_group_id)}/`}
    class="flex flex-row gap-2 items-center px-4 py-3 border-b border-gray-700 hover:bg-gray-700"
>
    <GroupAvatar groupType={group.group_type} {groupName} {counterpartyPubkey} {enrichedCounterparty} pxSize={40} />
    <div class="flex flex-col gap-1">
        <span class="text-lg font-semibold">{groupName}</span>
    </div>
</a>
