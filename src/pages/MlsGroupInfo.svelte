<script lang="ts">
    import { Page, Navbar, Block, List, ListItem, Popup, View } from "framework7-svelte";
    import type { NostrMlsGroup, EnrichedContact } from "../types/nostr";
    import { NostrMlsGroupType } from "../types/nostr";
    import { currentIdentity } from "../stores/accounts";
    import { invoke } from "@tauri-apps/api/core";
    import { nameFromMetadata } from "../utils/nostr";
    import GroupAvatar from "../components/GroupAvatar.svelte";
    import Avatar from "../components/Avatar.svelte";
    import Name from "../components/Name.svelte";
    import { npubFromPubkey } from "../utils/nostr";

    let { group }: { group: NostrMlsGroup } = $props();

    let counterpartyPubkey =
        group.group_type === NostrMlsGroupType.DirectMessage
            ? group.admin_pubkeys.filter((pubkey) => pubkey !== $currentIdentity)[0]
            : undefined;

    let enrichedCounterparty: EnrichedContact | undefined = $state(undefined);
    let groupName = $state("");
    let members: EnrichedContact[] = $state([]);
    let memberPubkeys: string[] = $state([]);

    $effect(() => {
        if (
            group.group_type === NostrMlsGroupType.DirectMessage &&
            counterpartyPubkey &&
            enrichedCounterparty
        ) {
            groupName = nameFromMetadata((enrichedCounterparty as EnrichedContact).metadata);
        } else {
            groupName = group.group_name;
        }
    });

    $effect(() => {
        if (counterpartyPubkey) {
            invoke("get_contact", { pubkey: counterpartyPubkey }).then((value) => {
                enrichedCounterparty = value as EnrichedContact;
            });
        }
    });

    async function getMembers() {
        memberPubkeys = await invoke("get_group_member_pubkeys", {
            mlsGroupId: group.mls_group_id,
        });
    }

    $inspect(memberPubkeys);
</script>

<Popup push>
    <View tab tabActive id="group-info-popup-view">
        <Page class="group-info-page bg-gray-900" noToolbar on:pageInit={getMembers}>
            <Navbar class="group-info-navbar justify-start py-8">
                <div slot="title" class="title-profile-link flex flex-row gap-2 items-center">
                    <div class="flex flex-col">Group Info</div>
                </div>
            </Navbar>
            <Block strong inset outline>
                <div class="flex flex-col justify-start items-center gap-4">
                    <GroupAvatar
                        groupType={group.group_type}
                        {groupName}
                        {counterpartyPubkey}
                        {enrichedCounterparty}
                        pxSize={96}
                    />
                    <h1 class="text-3xl font-bold">{groupName}</h1>
                    <h2 class="text-xl font-bold">{group.description}</h2>
                    {#if group.group_type === NostrMlsGroupType.DirectMessage}
                        <p>
                            Secure Direct Message with {groupName}
                        </p>
                        <div class="flex flex-col justify-start items-center gap-4">
                            <p class="text-gray-500 font-light flex flex-row gap-2 items-center">
                                {counterpartyPubkey
                                    ? npubFromPubkey(counterpartyPubkey)
                                    : "Unknown"}
                            </p>
                            <p class="px-8">{enrichedCounterparty?.metadata?.about}</p>
                            <p class="flex flex-row items-center justify-center gap-4">
                                <span>{enrichedCounterparty?.metadata?.nip05}</span>
                                <span class="text-gray-500 font-black">⋅</span>
                                <span>{enrichedCounterparty?.metadata?.website}</span>
                            </p>
                        </div>
                    {:else}
                        <p>Secure group chat with {memberPubkeys.length} members</p>
                    {/if}
                </div>
            </Block>
            {#if group.group_type === NostrMlsGroupType.Group}
                <Block strong inset outline>
                    <List title="Members" strong outline dividers mediaList>
                        {#each memberPubkeys as pubkey}
                            <ListItem mediaItem>
                                <div slot="media">
                                    <Avatar {pubkey} pxSize={32} />
                                </div>
                                <div slot="title">
                                    <Name {pubkey} />
                                </div>
                            </ListItem>
                        {/each}
                    </List>
                </Block>
            {/if}
        </Page>
    </View>
</Popup>
