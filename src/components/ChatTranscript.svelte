<script lang="ts">
    import ChatMessage from "./ChatMessage.svelte";
    import RespondPanel from "./RespondPanel.svelte";
</script>

<div class="flex flex-col gap-10 pt-10">
    <ChatMessage user={1} />
    <ChatMessage user={1} />
    <ChatMessage user={1} />
    <ChatMessage user={2} />
    <ChatMessage user={2} />
    <ChatMessage user={2} />
    <ChatMessage user={2} />
    <ChatMessage user={2} />
    <ChatMessage user={1} />
    <ChatMessage user={1} />
    <ChatMessage user={2} />
    <ChatMessage user={1} />
    <ChatMessage user={1} />
    <ChatMessage user={1} />
    <ChatMessage user={2} />
    <ChatMessage user={2} />
    <ChatMessage user={1} />
    <ChatMessage user={1} />
    <ChatMessage user={1} />
</div>
<RespondPanel />
