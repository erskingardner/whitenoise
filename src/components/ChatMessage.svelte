<script lang="ts">
    import { User } from "phosphor-svelte";

    export let user: number;
</script>

<div class="px-4 flex flex-row gap-4 items-end {user === 2 ? 'flex-row-reverse' : ''}">
    <div
        class="avatar border border-gray-400 rounded-full w-6 h-6 flex flex-col items-center justify-center shrink-0"
        style="background-color: purple;"
    >
        <User size="1.5rem" weight="thin" />
    </div>
    <div
        class="rounded-full {user === 1 ? 'bg-gray-700' : 'bg-blue-700'} {user === 2
            ? 'ml-auto'
            : ''} px-4 py-2 max-w-3/4"
    >
        This is a chat message
    </div>
</div>
