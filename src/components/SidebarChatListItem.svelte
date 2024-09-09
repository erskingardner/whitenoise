<script lang="ts">
    import { CheckCircle, User } from "phosphor-svelte";

    export let active: boolean = false;
    export let name: string;
    export let index: number;

    function getRandomColor() {
        const letters = "0123456789ABCDEF";
        let color = "rgba(";
        for (let i = 0; i < 3; i++) {
            color += Math.floor(Math.random() * 256) + ",";
        }
        color += "0.45)";
        return color;
    }
    let lorem =
        "Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book.";
</script>

<div
    class="chat-item flex flex-row gap-4 items-start justify-start px-4 py-2 first:border-t border-b border-gray-700 hover:bg-gray-800 {active
        ? 'bg-gray-800'
        : ''}"
>
    <div
        class="avatar border border-gray-400 rounded-full w-10 h-10 flex flex-col items-center justify-center shrink-0 mt-1"
        style="background-color: {index === 0 ? 'purple' : getRandomColor()}"
    >
        <User size="1.5rem" weight="thin" />
    </div>
    <div class="message-preview grow">
        <div class="name font-medium">{name}</div>
        <div class="preview font-extralight line-clamp-2">
            {lorem}
        </div>
    </div>
    <div class="metadata flex flex-col gap-1 items-center justify-center ml-auto">
        <div class="timestamp">Fri</div>
        <div class="read-state">
            <CheckCircle size="1rem" weight="thin" />
        </div>
    </div>
</div>
