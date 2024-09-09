<script lang="ts">
    import { MagnifyingGlass, DotsThreeCircle, User } from "phosphor-svelte";
</script>

<div class="sticky top-0 p-4 border-b border-gray-700 bg-gray-900">
    <div class="flex justify-between items-center w-full grow">
        <div class="flex flex-row gap-4 items-center">
            <div
                class="avatar border border-gray-400 rounded-full w-10 h-10 flex flex-col items-center justify-center shrink-0"
                style="background-color: purple;"
            >
                <User size="1.5rem" weight="thin" />
            </div>
            <div class="name font-medium grow">Max Hillebrand</div>
        </div>
        <div class="icons flex flex-row gap-4 items-center">
            <MagnifyingGlass size="1.5rem" weight="thin" />
            <DotsThreeCircle size="1.5rem" weight="thin" />
        </div>
    </div>
</div>
