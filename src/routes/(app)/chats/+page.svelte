<script lang="ts">
    import RespondPanel from "../../../components/RespondPanel.svelte";
    import Sidebar from "../../../components/Sidebar.svelte";
    import SidebarHeader from "../../../components/SidebarHeader.svelte";
    import MainPanel from "../../../components/MainPanel.svelte";
    import { MagnifyingGlass } from "phosphor-svelte";
    import ContactPanel from "../../../components/ContactPanel.svelte";
    import { slide } from "svelte/transition";
    import { invoke } from "@tauri-apps/api/core";
    import { identities, currentIdentity } from "../../../stores/accounts";
    import { tick } from "svelte";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import type { NEvent, NMetadata, NChat } from "../../../types/nostr";
    import { onMount, onDestroy } from "svelte";
    import Loader from "../../../components/Loader.svelte";
    import Contact from "../../../components/Contact.svelte";
    import ChatHeader from "../../../components/ChatHeader.svelte";
    import ChatMessage from "../../../components/ChatMessage.svelte";

    let contactPanelVisible = $state(false);
    let contactSearch = $state("");
    let searchTerm = $state("");

    function handleSearch(event: CustomEvent<string>) {
        searchTerm = event.detail;
    }

    function toggleContactPanel() {
        contactPanelVisible = !contactPanelVisible;
    }

    async function submitContactsSearch(event: KeyboardEvent | MouseEvent) {
        // TODO: implement contacts search
        if (event instanceof KeyboardEvent) {
            const { key } = event;
            if (key === "Enter") console.log("Submitted by keyboard");
        } else {
            console.log("Submitted by mouse");
        }
    }

    let selectedChat: string | undefined = $state(undefined);
    let chats: NChat = $state({});
    let isLoading = $state(true);

    async function getLegacyChats(): Promise<void> {
        isLoading = true;
        chats = {};
        selectedChat = undefined;
        try {
            const fetchedChats = (await invoke("get_legacy_chats", {
                pubkey: $currentIdentity,
            })) as NChat;
            console.log("fetchedChats", fetchedChats);
            const sortedChats = Object.entries(fetchedChats)
                .sort(([, a], [, b]) => b.latest - a.latest)
                .reduce((acc, [key, value]) => ({ ...acc, [key]: value }), {});
            chats = sortedChats;
            await tick();
            scrollToBottom();
        } catch (error) {
            console.error("Error fetching contacts:", error);
        } finally {
            isLoading = false;
        }
    }

    let unlisten: UnlistenFn;

    onMount(async () => {
        getLegacyChats();
        unlisten = await listen<string>("identity_change", (_event) => getLegacyChats());
    });

    onDestroy(() => {
        unlisten();
    });

    async function selectConversation(pubkey: string) {
        selectedChat = pubkey;
        isLoading = false;
        await tick();
        scrollToBottom();
    }

    function scrollToBottom() {
        const node = document.getElementById("main-panel") as HTMLElement;
        if (node) {
            node.scrollTo({
                top: node.scrollHeight + 1000,
                behavior: "smooth",
            });
        } else {
            console.error("Element with id 'main-panel' not found");
        }
    }

    function metadataForEvent(event: NEvent): NMetadata {
        if (event.pubkey === $currentIdentity) {
            return $identities[$currentIdentity] as NMetadata;
        }
        return chats[event.pubkey].metadata;
    }
</script>

<Sidebar>
    {#if contactPanelVisible}
        <div in:slide={{ delay: 100, duration: 100, axis: "x" }}>
            <ContactPanel on:backIconClicked={toggleContactPanel} />
        </div>
    {:else}
        <SidebarHeader
            title="Chats"
            on:search={handleSearch}
            on:newIconClicked={toggleContactPanel}
        />
        {#if isLoading}
            <div class="w-full h-10 mt-4 flex items-center justify-center">
                <Loader size={40} />
            </div>
        {/if}
        {#if !isLoading && Object.keys(chats).length === 0}
            <div class="text-gray-500 w-full p-4 text-center">No chats found</div>
        {/if}
        {#each Object.entries(chats) as [pubkey, chat]}
            <button onclick={() => selectConversation(pubkey)} class="w-full">
                <Contact
                    {pubkey}
                    metadata={chat.metadata}
                    active={pubkey === selectedChat}
                    lastMessageAt={Number(chat.latest)}
                />
            </button>
        {/each}
    {/if}
</Sidebar>
<MainPanel>
    {#if selectedChat === undefined}
        <div class="flex items-center justify-center w-full text-gray-500 h-screen grow">
            Select a conversation
        </div>
    {:else}
        <ChatHeader
            pubkey={selectedChat}
            metadata={selectedChat ? chats[selectedChat].metadata : undefined}
        />
        <div class="flex flex-col gap-10 py-10">
            {#each chats[selectedChat].events as event (event.id)}
                <ChatMessage {event} metadata={metadataForEvent(event)} />
            {/each}
        </div>
        <RespondPanel pubkey={selectedChat} />
    {/if}
</MainPanel>
