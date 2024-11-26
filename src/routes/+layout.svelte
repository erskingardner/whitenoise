<script lang="ts">
import Toaster from "$lib/components/Toaster.svelte";
import { setToastState } from "$lib/stores/toast-state.svelte";
import type { Snippet } from "svelte";
import "../app.css";

let { children }: { children: Snippet } = $props();

setToastState();
</script>

<Toaster />
<div>
    {@render children()}
</div>
