<script lang="ts">
    import { NotePencil, MagnifyingGlass } from "phosphor-svelte";
    import type { ComponentType } from "svelte";

    export let title: string;
    export let showSearch: boolean = true;
    export let showNewIcon: boolean = true;
    export let newIcon: ComponentType = NotePencil;
</script>

<div class="sticky top-0 p-4 pb-6 flex flex-col gap-6 bg-gray-900 border-b border-gray-700">
    <div class="flex flex-row gap-4 items-center">
        <h2 class="text-xl font-bold grow">{title}</h2>
        {#if showNewIcon}
            <div class=""><svelte:component this={newIcon} size="1.5rem" weight="thin" /></div>
        {/if}
    </div>
    {#if showSearch}
        <div class="search-container flex flex-row items-center relative text-sm">
            <input
                type="text"
                class="w-full p-2 rounded-md border border-gray-700 bg-transparent"
                placeholder="Search..."
            />
            <button class="absolute right-3">
                <MagnifyingGlass size="1.5rem" weight="thin" />
            </button>
        </div>
    {/if}
</div>
