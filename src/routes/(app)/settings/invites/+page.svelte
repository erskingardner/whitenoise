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
        <button class="flex flex-row gap-0.5 items-end" onclick={goBack}>
            <CaretLeft size={24} />
            <span class="text-xl font-medium">Back</span>
        </button>
    {/snippet}
    {#snippet center()}
        <h1>Invites</h1>
    {/snippet}
</HeaderToolbar>

<Header title="Invites" />
<main class="px-4 flex flex-col"></main>
