<script lang="ts">
    import { PaperPlaneTilt } from "phosphor-svelte";
    let { group } = $props();

    let message = $state("");
    let textarea: HTMLTextAreaElement;

    function adjustHeight() {
        textarea.style.height = "auto";
        textarea.style.height = textarea.scrollHeight + "px";
    }

    function handleInput() {
        adjustHeight();
    }

    function sendMessage() {
        console.log("sendMessage", message);
        message = "";
        setTimeout(adjustHeight, 0);
    }

    function handleKeydown(event: KeyboardEvent) {
        if ((event.metaKey || event.ctrlKey) && event.key === "Enter") {
            sendMessage();
        }
    }
</script>

<div
    class="flex flex-row px-8 py-4 gap-4 items-center border-t border-gray-700 sticky bottom-0 left-0 right-0 bg-gray-900 drop-shadow-message-bar"
>
    <textarea
        bind:this={textarea}
        class="px-4 py-2 w-full bg-transparent ring-1 ring-gray-700 rounded-lg min-h-[2.5rem] max-h-[12rem] resize-none overflow-y-auto"
        rows="1"
        bind:value={message}
        oninput={handleInput}
        onkeydown={handleKeydown}
    ></textarea>
    <button
        class="p-3 bg-blue-700 rounded-full text-white ring-1 ring-blue-500 hover:bg-blue-600"
        onclick={sendMessage}
    >
        <PaperPlaneTilt size={24} />
    </button>
</div>
