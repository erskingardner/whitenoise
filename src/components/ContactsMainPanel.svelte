<script lang="ts">
    let { selectedContact = $bindable() } = $props();
</script>

{#if selectedContact === undefined}
    <div class="flex items-center justify-center w-full text-gray-500 h-screen grow">
        Select a contact
    </div>
{:else}
    <div class="flex items-center justify-center w-full text-gray-500 h-screen grow">
        The selected contact is {selectedContact}
    </div>
{/if}
