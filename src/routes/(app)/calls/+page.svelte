<script lang="ts">
import Header from "$lib/components/Header.svelte";
import HeaderToolbar from "$lib/components/HeaderToolbar.svelte";
</script>

<HeaderToolbar>
    {#snippet center()}
        <h1>Calls</h1>
    {/snippet}
</HeaderToolbar>

<Header title="Calls" />
<main class="px-4">
    <div class="calls-page-content flex flex-col justify-start items-center text-gray-500 mt-40">
        Two Weeks™
    </div>
</main>
