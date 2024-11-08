<script lang="ts">
    import { fly, fade } from "svelte/transition";

    let {
        title,
        body,
        acceptText,
        acceptStyle,
        cancelText,
        acceptFn,
        showAlert = $bindable(),
    } = $props<{
        title: string;
        body: string;
        acceptText: string;
        acceptStyle: "primary" | "outline" | "warning";
        cancelText: string;
        acceptFn: () => void;
        showAlert: boolean;
    }>();

    function toggleAlert(e: MouseEvent) {
        e.stopPropagation();
        showAlert = !showAlert;
    }
</script>

<div
    class="fixed inset-0 bg-black/50 flex flex-col items-center justify-center z-40"
    transition:fade={{ duration: 100 }}
>
    <div
        class="ring-1 ring-gray-700 bg-gray-900 flex flex-col gap-4 p-6 rounded-lg shadow-xl w-3/4 z-50"
        transition:fly={{ y: 200, duration: 300 }}
    >
        <h2 class="text-2xl font-bold">{title}</h2>
        <div class="whitespace-pre-wrap">{body}</div>
        <div class="flex flex-row gap-4 items-center justify-around">
            <button onclick={() => acceptFn()} class="button-{acceptStyle} w-full text-center">
                {acceptText}
            </button>
            <button onclick={toggleAlert} class="button-outline w-full text-center">
                {cancelText}
            </button>
        </div>
    </div>
</div>
