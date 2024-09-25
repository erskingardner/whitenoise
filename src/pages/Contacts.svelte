<script lang="ts">
    import {
        View,
        Link,
        List,
        ListGroup,
        ListItem,
        Navbar,
        Page,
        Popup,
        Searchbar,
        Subnavbar,
        ListIndex,
        Icon,
        f7ready,
    } from "framework7-svelte";
    import type { NUsers } from "../types/nostr";
    import { invoke } from "@tauri-apps/api/core";
    import { UsersThree, User } from "phosphor-svelte";
    import { npubFromPubkey } from "../utils/nostr";
    import Avatar from "../components/Avatar.svelte";
    import Loader from "../components/Loader.svelte";

    let { modalTitle = "Contacts", onContactSelect } = $props();

    let contacts: NUsers = $state({});
    let isLoading = $state(true);

    async function getContacts(): Promise<void> {
        isLoading = true;
        contacts = {};
        try {
            contacts = await invoke("get_contacts");
        } catch (error) {
            console.error("Error fetching contacts:", error);
        } finally {
            isLoading = false;
        }
    }

    $inspect(contacts);

    f7ready(async () => {
        await getContacts();
    });

    // Transform and sort contacts object into an array
    let sortedContacts = $derived(
        Object.entries(contacts)
            .map(([pubkey, userData]) => ({
                pubkey,
                ...userData,
                ...userData.metadata,
            }))
            .sort((a, b) => {
                const nameA = (a.name || a.display_name || "").toLowerCase();
                const nameB = (b.name || b.display_name || "").toLowerCase();
                return nameA.localeCompare(nameB);
            })
    );

    function getGroups() {
        const groupedContacts: { [key: string]: typeof sortedContacts } = {};

        sortedContacts.forEach((contact) => {
            const name = contact.name || contact.display_name || "";
            const key = name ? name[0].toUpperCase() : "-";

            if (!groupedContacts[key]) {
                groupedContacts[key] = [];
            }
            groupedContacts[key].push(contact);
        });

        // Sort the keys alphabetically, but ensure '-' is at the end if it exists
        return Object.keys(groupedContacts)
            .sort((a, b) => {
                if (a === "-") return 1;
                if (b === "-") return -1;
                return a.localeCompare(b);
            })
            .reduce(
                (acc, key) => {
                    acc[key] = groupedContacts[key];
                    return acc;
                },
                {} as { [key: string]: typeof sortedContacts }
            );
    }

    // Group sorted contacts by first letter of name
    let groups = $derived(getGroups());

    // Create a link in the empty sidebar to do a NIP-50 and primal cache search
    // This method will handle that
    async function submitContactsSearch(event: KeyboardEvent | MouseEvent) {
        // TODO: implement contacts search
        if (event instanceof KeyboardEvent) {
            const { key } = event;
            if (key === "Enter") console.log("Submitted by keyboard");
        } else {
            console.log("Submitted by mouse");
        }
    }
</script>

<Popup push>
    <View>
        <Page class="contacts-page bg-gray-900">
            <Navbar title={modalTitle}>
                <Link slot="right" popupClose>Cancel</Link>
                <Subnavbar>
                    <Searchbar searchContainer=".contacts-list" disableButton={false} />
                </Subnavbar>
            </Navbar>
            <List strongIos outlineIos dividersIos class="searchbar-not-found">
                <ListItem title="Nothing found" />
            </List>
            {#if isLoading}
                <div class="flex items-start justify-center bg-gray-900 h-full pt-10">
                    <Loader fullscreen={false} size={32} />
                </div>
            {:else}
                <ListIndex
                    indexes={Object.keys(groups)}
                    listEl=".contacts-list"
                    class="bg-transparent"
                />
                <List
                    contactsList
                    noChevron
                    dividers
                    ul={false}
                    class="searchbar-found bg-gray-900"
                >
                    <ListItem link noChevron class="list-none">
                        <UsersThree slot="media" size={40} class="text-blue-700" />
                        <span slot="title" class="text-color-primary"> New Group </span>
                    </ListItem>
                    <ListItem link noChevron class="list-none">
                        <User slot="media" size={40} class="text-blue-700" />
                        <span slot="title" class="text-color-primary"> New Contact </span>
                    </ListItem>

                    {#each Object.keys(groups) as groupKey}
                        <ListGroup key={groupKey}>
                            <ListItem groupTitle title={groupKey} class="p-0 w-full" />
                            {#each groups[groupKey] as contact (contact.pubkey)}
                                <ListItem
                                    link
                                    title={contact.display_name ||
                                        contact.name ||
                                        npubFromPubkey(contact.pubkey)}
                                    popupClose
                                    on:click={() =>
                                        onContactSelect(
                                            contact.pubkey,
                                            contacts[contact.pubkey].metadata
                                        )}
                                >
                                    <Avatar
                                        slot="media"
                                        picture={contact.picture}
                                        pubkey={contact.pubkey}
                                    />
                                </ListItem>
                            {/each}
                        </ListGroup>
                    {/each}
                </List>
            {/if}
        </Page>
    </View>
</Popup>
