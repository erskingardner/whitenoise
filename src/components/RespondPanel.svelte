<script lang="ts">
    import { Microphone } from "phosphor-svelte";

    // TODO: Implement changing Microphone Icon to Send if input is not empty
    function comingSoon() {
        console.log("Coming soon!");
    }
</script>

<div class="sticky bottom-0 flex flex-row gap-4 p-4 bg-gray-800 items-center">
    <input
        type="text"
        placeholder="Type a message..."
        class="bg-transparent ring-1 ring-gray-700 rounded-md grow p-2"
    />
    <button on:click={comingSoon} class="rounded-full border border-gray-700 p-2">
        <Microphone size="1.5rem" weight="thin" />
    </button>
</div>
