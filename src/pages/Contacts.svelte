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
    } from "framework7-svelte";
    import { isValidHexPubkey } from "../types/nostr";
    import { UsersThree, User } from "phosphor-svelte";
    import { npubFromPubkey } from "../utils/nostr";
    import Avatar from "../components/Avatar.svelte";
    import Loader from "../components/Loader.svelte";
    import ndk from "../stores/ndk";
    import type { NDKUser, NDKUserProfile } from "@nostr-dev-kit/ndk";

    interface Props {
        modalTitle?: string;
        onContactSelect: (pubkey: string, profile: NDKUserProfile | undefined) => void;
    }

    let { modalTitle = "Contacts", onContactSelect }: Props = $props();

    let isLoading = $state(true);
    let contacts: NDKUser[] = $state([]);

    $effect(() => {
        if ($ndk.activeUser) {
            $ndk.activeUser
                .follows()
                .then(async (follows) => {
                    isLoading = true;
                    const cleanedContacts = Array.from(follows).filter((follow) =>
                        isValidHexPubkey(follow.pubkey)
                    );
                    await Promise.all(
                        cleanedContacts.map((follow: NDKUser) => {
                            follow.fetchProfile();
                        })
                    );
                    contacts = cleanedContacts;
                })
                .catch((error) => {
                    console.error("Error fetching NDK contacts:", error);
                })
                .finally(() => {
                    console.log("Contacts loaded");
                    isLoading = false;
                });
        }
    });

    let sortedContacts = $derived(getSortedContacts(contacts));
    let groups = $derived(getGroups(sortedContacts));

    function getSortedContacts(contacts: NDKUser[]): NDKUser[] {
        return contacts.sort((a, b) => {
            const nameA = (a.profile?.displayName || a.profile?.name || "").toLowerCase();
            const nameB = (b.profile?.displayName || b.profile?.name || "").toLowerCase();
            return nameA.localeCompare(nameB);
        });
    }

    function getGroups(sortedContacts: NDKUser[]): { [key: string]: NDKUser[] } {
        const groupedContacts: { [key: string]: NDKUser[] } = {};

        sortedContacts.forEach((contact) => {
            const name = contact.profile?.name || contact.profile?.displayName || "";
            const key = name ? name[0].toUpperCase() : "#";

            if (!groupedContacts[key]) {
                groupedContacts[key] = [];
            }
            groupedContacts[key].push(contact);
        });

        // Sort the keys alphabetically, but ensure '#' is at the end if it exists
        return Object.keys(groupedContacts)
            .sort((a, b) => {
                if (a === "#") return 1;
                if (b === "#") return -1;
                return a.localeCompare(b);
            })
            .reduce(
                (acc, key) => {
                    acc[key] = groupedContacts[key];
                    return acc;
                },
                {} as { [key: string]: NDKUser[] }
            );
    }

    // Create a link in the empty sidebar to do a NIP-50 and primal cache search
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

                    {#each Object.entries(groups) as [groupKey, contacts]}
                        <ListGroup>
                            <ListItem groupTitle title={groupKey} class="p-0 w-full" />
                            {#each contacts as contact (contact.pubkey)}
                                <ListItem
                                    link
                                    title={contact.profile?.displayName ||
                                        contact.profile?.name ||
                                        npubFromPubkey(contact.pubkey)}
                                    popupClose
                                    on:click={() =>
                                        onContactSelect(contact.pubkey, contact.profile)}
                                >
                                    <Avatar
                                        slot="media"
                                        picture={contact.profile?.image}
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
