<script lang="ts">
    import type { Snippet } from "svelte";

    let { left, center, right } = $props<{
        left?: () => Snippet;
        center?: () => Snippet;
        right?: () => Snippet;
    }>();

    // Fade the center text in when scrolling down
    let centerOpacity = $state(0);
    window.addEventListener("scroll", () => {
        centerOpacity = Math.min(window.scrollY / 200, 1);
    });

    let headerOpacity = $derived(centerOpacity / 1.2);
</script>

<div
    class="flex flex-row justify-between items-center p-4 sticky h-16 top-0 left-0 right-0 backdrop-blur-sm"
    style="border-bottom: 1px solid rgba(55, 65, 81, {centerOpacity}); background-color: rgba(3, 7, 18, {headerOpacity})"
>
    {#if left}
        {@render left()}
    {:else}
        <span></span>
    {/if}

    {#if center}
        <div class="font-bold text-xl" style="opacity:{centerOpacity};">
            {@render center()}
        </div>
    {:else}
        <span></span>
    {/if}

    {#if right}
        {@render right()}
    {:else}
        <span></span>
    {/if}
</div>
