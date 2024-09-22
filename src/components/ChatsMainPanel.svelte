<script lang="ts">
    import ChatHeader from "./ChatHeader.svelte";
    import ChatMessage from "./ChatMessage.svelte";
    import RespondPanel from "./RespondPanel.svelte";

    let {
        selectedChat = $bindable(),
        chats = $bindable(),
    }: { selectedChat: string | undefined; chats: Record<string, Chat> } = $props();
</script>

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
