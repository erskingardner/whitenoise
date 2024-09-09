<script lang="ts">
    import SidebarChatListItem from "./SidebarChatListItem.svelte";

    let names = [
        "Max Hillebrand",
        "PabloF7Z",
        "L0la L33tz",
        "Laura",
        "Hodlbod",
        "Satoshi Nakamoto",
        "Rikki ☠️",
        "Fiatjaf",
        "NVK",
        "Mads",
        "Miljan",
        "Amiti",
        "Jane Doe",
        "Thomas Paine",
    ];
</script>

<div class="chat-list flex flex-col gap-0 overflow-y-scroll">
    {#each names as name, i}
        <SidebarChatListItem active={i === 0 ? true : false} {name} index={i} />
    {/each}
</div>
