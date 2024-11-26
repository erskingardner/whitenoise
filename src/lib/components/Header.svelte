<script lang="ts">
interface Props {
    title: string;
}
let { title }: Props = $props();
</script>

<div class="flex flex-row justify-between items-center px-4">
    <h1 class="text-4xl font-extrabold pb-6">{title}</h1>
</div>
