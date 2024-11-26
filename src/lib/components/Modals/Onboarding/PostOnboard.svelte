<script lang="ts">
import type { CloseModal } from "$lib/types/modal";

let { closeModal }: { closeModal: CloseModal } = $props();
</script>

<div class="flex flex-col gap-10 mt-10 items-center w-full md:w-2/3 lg:w-1/2 mx-auto">
    <div class="text-7xl">🎉</div>
    <p class="mt-4">Nice work! You're all set up and ready to chat.</p>
    <button class="button-primary" onclick={closeModal}>Let's go 🤙</button>
</div>
