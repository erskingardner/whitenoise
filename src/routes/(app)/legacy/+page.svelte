<script lang="ts">
    import MainPanel from "../../../components/MainPanel.svelte";
    import Contact from "../../../components/Contact.svelte";
    import Sidebar from "../../../components/Sidebar.svelte";
    import SidebarHeader from "../../../components/SidebarHeader.svelte";
    import RespondPanel from "../../../components/RespondPanel.svelte";
    import ChatHeader from "../../../components/ChatHeader.svelte";
    import Loader from "../../../components/Loader.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { type NChat } from "../../../types/nostr";
    import { onMount, onDestroy } from "svelte";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import { currentIdentity } from "../../../stores/identities";
    import ChatMessage from "../../../components/ChatMessage.svelte";
    import { tick } from "svelte";

    let selectedChat: string | undefined = $state(undefined);
    let chats: NChat[] = $state([]);
    let isLoading = $state(true);

    async function getLegacyChats(): Promise<void> {
        isLoading = true;
        chats = [];
        selectedChat = undefined;
        try {
            const fetchedChats = (await invoke("get_legacy_chats", {
                pubkey: $currentIdentity,
            })) as NChat;
            const sortedChats = Object.entries(fetchedChats)
                .sort(([, a], [, b]) => b.latest - a.latest)
                .reduce((acc, [key, value]) => ({ ...acc, [key]: value }), {});
            chats = sortedChats as NChat[];
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
        unlisten = await listen<string>("identity_change", (event) => {
            console.log("identity_change on legacy chats", event);
            getLegacyChats();
        });
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
        console.log("scrollToBottom", node);
        if (node) {
            node.scrollTo({
                top: node.scrollHeight + 1000,
                behavior: "smooth",
            });
        } else {
            console.error("Element with id 'main-panel' not found");
        }
    }
</script>

<Sidebar>
    <SidebarHeader title="Legacy Chats" />
    {#if isLoading}
        <div class="w-full h-10 mt-4 flex items-center justify-center">
            <Loader size={40} />
        </div>
    {/if}
    {#if !isLoading && chats.length === 0}
        <div class="text-gray-500 w-full p-4 text-center">No chats found</div>
    {/if}
    {#each Object.entries(chats) as [pubkey, chat]}
        <button onclick={() => selectConversation(pubkey)} class="w-full">
            <Contact
                {pubkey}
                active={pubkey === selectedChat}
                lastMessageAt={Number(chat.latest)}
            />
        </button>
    {/each}
</Sidebar>
<MainPanel>
    {#if selectedChat === undefined}
        <div class="flex items-center justify-center w-full text-gray-500 h-screen grow">
            Select a conversation
        </div>
    {:else}
        <ChatHeader pubkey={selectedChat} />
        <div class="flex flex-col gap-10 py-10">
            {#each chats[selectedChat].events as event (event.id)}
                <ChatMessage {event} />
            {/each}
        </div>
        <RespondPanel />
    {/if}
</MainPanel>
