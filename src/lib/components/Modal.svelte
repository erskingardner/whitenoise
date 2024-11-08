<script lang="ts">
    import { fly, fade } from "svelte/transition";
    import { X, CaretLeft } from "phosphor-svelte";
    import type { Snippet } from "svelte";
    import { onDestroy } from "svelte";
    import HeaderToolbar from "./HeaderToolbar.svelte";

    type ModalViewProps = Record<string, (...args: any[]) => any>;

    let {
        title,
        showModal = $bindable(),
        children,
    }: { title: string; showModal: boolean; children: Snippet } = $props();

    // Stack to keep track of views/pages
    let viewStack: { title: string; content: Snippet; props?: Record<string, any> }[] = $state([]);
    let currentView: { title: string; content: Snippet; props?: Record<string, any> } = $state({
        title,
        content: children,
        props: {},
    });

    // Navigation methods
    export function pushView(
        newTitle: string,
        newContent: Snippet,
        modalViewProps?: ModalViewProps
    ) {
        viewStack = [...viewStack, currentView];
        currentView = { title: newTitle, content: newContent, props: modalViewProps || {} };
    }

    export function popView() {
        if (viewStack.length > 0) {
            currentView = viewStack[viewStack.length - 1];
            viewStack = viewStack.slice(0, -1);
        }
    }

    $inspect(viewStack);

    // Lock/unlock scroll when modal opens/closes
    $effect(() => {
        if (showModal) {
            document.body.style.overflow = "hidden";
        } else {
            document.body.style.overflow = "auto";
        }
    });

    // Cleanup on component destroy
    onDestroy(() => {
        document.body.style.overflow = "auto";
    });

    // We can pass in subpages by creating snippets in parent views like this:
    //
    // const InboxRelays = createRawSnippet(() => {
    //     return {
    //         render: () => `
    //     <div class="flex flex-col gap-4 justify-start items-center w-full h-full">
    //         <h1>Specify your inbox relays</h1>
    //         <button
    //             class="button-primary"
    //             id="inbox-relays-button"
    //         >
    //             Specify your inbox relays
    //         </button>
    //     </div>
    //     `,
    //         setup: (element: Element) => {
    //             const button = element.querySelector("#inbox-relays-button");
    //             button?.addEventListener("click", () => {
    //                 console.log("Inbox relays button clicked");
    //             });
    //         },
    //     };
    // });
</script>

<div
    class="fixed inset-0 -bottom-10 md:p-8 bg-black bg-opacity-50 flex items-end z-20"
    transition:fade={{ duration: 100 }}
>
    <div
        class="flex flex-col gap-4 bg-gray-800 ring-2 ring-gray-700 rounded-t-xl w-full h-[90vh] z-30 pb-40 overflow-y-scroll"
        transition:fly={{ y: 800, duration: 300 }}
    >
        <HeaderToolbar>
            {#snippet left()}
                {#if viewStack.length > 0}
                    <button class="flex flex-row gap-0.5 items-end" onclick={popView}>
                        <CaretLeft size={24} />
                        <span class="text-xl font-medium">Back</span>
                    </button>
                {/if}
            {/snippet}

            {#snippet center()}<h1>{currentView.title}</h1>{/snippet}

            {#snippet right()}
                <div>
                    <button onclick={() => (showModal = false)}>
                        <X size={30} />
                    </button>
                </div>
            {/snippet}
        </HeaderToolbar>
        <div class="p-4">
            {@render currentView.content()}
        </div>
    </div>
</div>
