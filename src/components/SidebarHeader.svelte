<script lang="ts">
    import { NotePencil, MagnifyingGlass } from "phosphor-svelte";
    import type { ComponentType } from "svelte";
    import { createEventDispatcher } from "svelte";

    const dispatch = createEventDispatcher();

    // type Props = {
    //     title: string;
    //     showSearch: boolean;
    //     showNewIcon: boolean;
    //     newIcon: ComponentType;
    // };

    // let {
    //     title,
    //     showSearch = true,
    //     showNewIcon = true,
    //     newIcon = NotePencil,
    // }: Props = $props();

    export let title: string;
    export let showSearch: boolean = true;
    export let showNewIcon: boolean = true;
    export let newIcon: ComponentType = NotePencil;

    let searchTerm: string = "";
</script>

<div
    class="sticky top-0 p-4 pb-6 flex flex-col gap-6 bg-gray-900 border-b border-gray-700 relative"
>
    <div class="flex flex-row gap-4 items-center">
        <h2 class="text-xl py-2 font-bold grow">{title}</h2>
        {#if showNewIcon}
            <button
                onclick={() => dispatch("newIconClicked")}
                class="p-2 rounded-md hover:bg-gray-800"
            >
                <!-- {@render this=newIcon size="1.5rem" weight="" } -->
                <svelte:component this={newIcon} size="1.5rem" weight="thin" />
            </button>
        {/if}
    </div>
    {#if showSearch}
        <div class="search-container flex flex-row items-center relative text-sm">
            <input
                type="text"
                bind:value={searchTerm}
                class="w-full p-2 rounded-md border border-gray-700 bg-transparent"
                placeholder="Search&hellip;"
                oninput={() => dispatch("search", searchTerm)}
            />
            <button class="absolute right-3">
                <MagnifyingGlass size="1.5rem" weight="thin" />
            </button>
        </div>
    {/if}
    <slot />
</div>
