<script lang="ts">
    import { XCircle } from "phosphor-svelte";

    type Props = {
        modalVisible: boolean;
        title: string;
    };
    let { modalVisible = false, title }: Props = $props();
</script>

{#if modalVisible}
    <div
        aria-modal="true"
        class="modal z-20 absolute top-20 left-1/2 -translate-x-1/2 p-4 rounded-lg shadow-lg bg-gray-800 ring-1 ring-gray-700 flex flex-col pb-10 w-1/2"
    >
        <h2 class="text-lg font-semibold mb-6">{title}</h2>
        <button
            onclick={() => (modalVisible = !modalVisible)}
            class="absolute top-4 right-4 p-2 rounded-md hover:text-white"
        >
            <XCircle size="1.5rem" weight="thin" />
        </button>
        <slot />
    </div>

    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
        class="w-screen h-screen absolute bg-gray-950/50"
        onclick={() => (modalVisible = !modalVisible)}
    ></div>
{/if}
