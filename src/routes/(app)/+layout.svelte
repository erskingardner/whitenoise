<script lang="ts">
    import "../../app.pcss";
    import Menu from "../../components/Menu.svelte";
</script>

<div class="flex flex-row gap-0">
    <Menu />
    <slot />
</div>
