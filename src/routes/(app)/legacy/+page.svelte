<script lang="ts">
import Header from "$lib/components/Header.svelte";
import HeaderToolbar from "$lib/components/HeaderToolbar.svelte";
import { PlusCircle } from "phosphor-svelte";

let isLoading = $state(false);
let loadingError = $state<string | null>(null);

// Load nip17 chats
// decrypt content
// load in stages? infinite scroll style?
// show loading state
// show error state
</script>

<HeaderToolbar>
    {#snippet right()}
        <div>
            <button onclick={() => console.log("plus clicked")} class="p-2 -mr-2">
                <PlusCircle size={30} />
            </button>
        </div>
    {/snippet}
    {#snippet center()}
        <h1>Legacy DMs</h1>
    {/snippet}
</HeaderToolbar>

<Header title="Legacy DMs" />
<main class="px-4">
    <div
        class="calls-page-content flex flex-col justify-start items-center text-gray-500 mt-64 text-center"
    >
        Two Weeks™ <br />
        ...or, maybe never.
    </div>
</main>
