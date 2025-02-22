<script lang="ts">
import { goto } from "$app/navigation";
import Header from "$lib/components/Header.svelte";
import HeaderToolbar from "$lib/components/HeaderToolbar.svelte";
import { CaretLeft } from "phosphor-svelte";

function goBack() {
    goto("/settings");
}
</script>

<HeaderToolbar>
    {#snippet left()}
        <button class="flex flex-row gap-0.5 items-center" onclick={goBack}>
            <CaretLeft size={24} weight="bold" />
            <span class="font-medium text-lg">Back</span>
        </button>
    {/snippet}
    {#snippet center()}
        <h1>Profile</h1>
    {/snippet}
</HeaderToolbar>

<Header title="Profile" />
<main class="px-4 flex flex-col pb-32">

</main>
