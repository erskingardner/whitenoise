<script lang="ts">
import type { NEvent, NostrMlsGroup } from "$lib/types/nostr";
import { invoke } from "@tauri-apps/api/core";
import { PaperPlaneTilt } from "phosphor-svelte";
import Loader from "./Loader.svelte";

let {
    group,
    handleNewMessage,
}: { group: NostrMlsGroup; handleNewMessage: (message: NEvent) => void } = $props();

let message = $state("");
let textarea: HTMLTextAreaElement;
let sendingMessage: boolean = $state(false);

function adjustTextareaHeight() {
    textarea.style.height = "auto";
    textarea.style.height = `${textarea.scrollHeight}px`;
}

function handleInput() {
    adjustTextareaHeight();
}

async function sendMessage() {
    if (message.length === 0) return;
    sendingMessage = true;
    await invoke("send_mls_message", {
        group,
        message: message,
    })
        .then((messageEvent) => {
            handleNewMessage(messageEvent as NEvent);
            // Clear the message input and adjust the height of the textarea
            message = "";
            setTimeout(adjustTextareaHeight, 0);
        })
        .finally(() => {
            sendingMessage = false;
        });
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
        class="p-3 bg-blue-700 rounded-full text-white ring-1 ring-blue-500 hover:bg-blue-600 disabled:hidden"
        onclick={sendMessage}
        disabled={sendingMessage}
    >
        <PaperPlaneTilt size={24} />
    </button>
    <div
        class="p-3 bg-blue-700 rounded-full text-white ring-1 ring-blue-500"
        class:hidden={!sendingMessage}
    >
        <Loader fullscreen={false} size={24} />
    </div>
</div>
