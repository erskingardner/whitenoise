<script lang="ts">
import type { Toast } from "$lib/stores/toast-state.svelte";
import { X } from "phosphor-svelte";
import { getToastState } from "$lib/stores/toast-state.svelte";
import { fly } from "svelte/transition";

let toastState = getToastState();
let { toast }: { toast: Toast } = $props();

let accentColor: string = $derived.by(() => {
    if (toast.type === "error") return "ring-red-700";
    if (toast.type === "success") return "ring-green-700";
    return "ring-gray-700";
});
</script>

<div
    transition:fly={{ duration: 200, y: -20 }}
    class="bg-gray-950/90 text-white ring-1 {accentColor} rounded-md p-4 shadow-md relative"
>
    <h3 class="text-lg font-bold">{toast.title}</h3>
    <p>{toast.message}</p>
    <X
        size={32}
        class="absolute top-2 right-2 cursor-pointer"
        onclick={() => toastState.remove(toast.id)}
    />
</div>
