<script lang="ts">
import { getToastState } from "$lib/stores/toast-state.svelte";
import Toast from "./Toast.svelte";

let toastState = getToastState();
</script>

<div class="absolute top-2 left-2 right-2 z-50 flex flex-col gap-2">
    {#each toastState.toasts as toast}
        <Toast {toast} />
    {/each}
</div>
