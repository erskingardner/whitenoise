<script lang="ts">
import { goto } from "$app/navigation";
import Header from "$lib/components/Header.svelte";
import HeaderToolbar from "$lib/components/HeaderToolbar.svelte";
import { activeAccount } from "$lib/stores/accounts";
import type { EnrichedContact, NMetadata } from "$lib/types/nostr";
import { invoke } from "@tauri-apps/api/core";
import { CaretLeft } from "phosphor-svelte";
import { get } from "svelte/store";

function goBack() {
    goto("/settings");
}

let metadata: NMetadata = $state({});

// Load the current user's metadata
$effect(() => {
    const account = get(activeAccount);
    if (account?.pubkey) {
        invoke<EnrichedContact>("query_enriched_contact", {
            pubkey: account.pubkey,
            updateAccount: false,
        }).then((response) => {
            metadata = response?.metadata ?? {};
        });
    }
});

async function handleImageUpload(event: Event, field: "picture" | "banner") {
    const input = event.target as HTMLInputElement;
    if (input.files?.[0]) {
        const file = input.files[0];
        // TODO: Implement image upload to a hosting service
        // For now, we'll just use a local URL
        metadata[field] = URL.createObjectURL(file);
    }
}

async function handleSubmit(event: Event) {
    event.preventDefault();
    // TODO: Implement saving profile changes via Nostr
    console.log("Saving profile:", metadata);
}
</script>

<HeaderToolbar>
    {#snippet left()}
        <button class="flex flex-row gap-0.5 items-center" onclick={goBack}>
            <CaretLeft size={24} weight="bold" />
            <span class="font-medium text-lg">Back</span>
        </button>
    {/snippet}
    {#snippet center()}
        <h1>Profile</h1>
    {/snippet}
</HeaderToolbar>

<Header title="Profile" />

<main class="px-4 flex flex-col pb-32 gap-6">
    <!-- Banner Image -->
    <div class="relative w-full h-48 bg-gray-100 dark:bg-gray-800 rounded-lg overflow-hidden">
        {#if metadata.banner}
            <img src={metadata.banner} alt="Profile banner" class="w-full h-full object-cover" />
        {/if}
        <label class="absolute bottom-4 right-4 bg-white dark:bg-gray-700 px-4 py-2 rounded-lg cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors">
            <input type="file" accept="image/*" class="hidden" onchange={(e) => handleImageUpload(e, 'banner')} />
            Change Banner
        </label>
    </div>

    <!-- Avatar Image -->
    <div class="relative w-32 h-32 mx-auto -mt-16">
        <div class="w-full h-full rounded-full bg-gray-100 dark:bg-gray-800 overflow-hidden border-4 border-white dark:border-gray-900">
            {#if metadata.picture}
                <img src={metadata.picture} alt="profile" class="w-full h-full object-cover" />
            {:else}
                <div class="w-full h-full flex items-center justify-center text-gray-400">
                    <span class="text-4xl">?</span>
                </div>
            {/if}
        </div>
        <label class="absolute bottom-0 right-0 bg-white dark:bg-gray-700 p-2 rounded-full cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors shadow-lg">
            <input type="file" accept="image/*" class="hidden" onchange={(e) => handleImageUpload(e, 'picture')} />
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                <path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z" />
            </svg>
        </label>
    </div>

    <!-- Profile Form -->
    <form class="space-y-6 max-w-2xl mx-auto w-full" onsubmit={handleSubmit}>
        <div class="space-y-4">
            <!-- Display Name -->
            <div>
                <label for="display_name" class="block text-sm font-medium mb-1">Display Name</label>
                <input
                    type="text"
                    id="display_name"
                    bind:value={metadata.display_name}
                    class="w-full px-4 py-2 rounded-lg border dark:border-gray-700 bg-white dark:bg-gray-800 focus:ring-2 focus:ring-blue-500 dark:focus:ring-blue-400"
                    placeholder="Your display name"
                />
            </div>

            <!-- Username -->
            <div>
                <label for="name" class="block text-sm font-medium mb-1">Name</label>
                <input
                    type="text"
                    id="name"
                    bind:value={metadata.name}
                    class="w-full px-4 py-2 rounded-lg border dark:border-gray-700 bg-white dark:bg-gray-800 focus:ring-2 focus:ring-blue-500 dark:focus:ring-blue-400"
                    placeholder="Your username"
                />
            </div>

            <!-- About -->
            <div>
                <label for="about" class="block text-sm font-medium mb-1">About</label>
                <textarea
                    id="about"
                    bind:value={metadata.about}
                    class="w-full px-4 py-2 rounded-lg border dark:border-gray-700 bg-white dark:bg-gray-800 focus:ring-2 focus:ring-blue-500 dark:focus:ring-blue-400 min-h-[100px]"
                    placeholder="Tell us about yourself"
                ></textarea>
            </div>

            <!-- Website -->
            <div>
                <label for="website" class="block text-sm font-medium mb-1">Website</label>
                <input
                    type="url"
                    id="website"
                    bind:value={metadata.website}
                    class="w-full px-4 py-2 rounded-lg border dark:border-gray-700 bg-white dark:bg-gray-800 focus:ring-2 focus:ring-blue-500 dark:focus:ring-blue-400"
                    placeholder="https://your-website.com"
                />
            </div>

            <!-- NIP-05 Verification -->
            <div>
                <label for="nip05" class="block text-sm font-medium mb-1">NIP-05 Verification</label>
                <input
                    type="text"
                    id="nip05"
                    bind:value={metadata.nip05}
                    class="w-full px-4 py-2 rounded-lg border dark:border-gray-700 bg-white dark:bg-gray-800 focus:ring-2 focus:ring-blue-500 dark:focus:ring-blue-400"
                    placeholder="you@domain.com"
                />
            </div>

            <!-- Lightning Address -->
            <div>
                <label for="lud16" class="block text-sm font-medium mb-1">Lightning Address</label>
                <input
                    type="text"
                    id="lud16"
                    bind:value={metadata.lud16}
                    class="w-full px-4 py-2 rounded-lg border dark:border-gray-700 bg-white dark:bg-gray-800 focus:ring-2 focus:ring-blue-500 dark:focus:ring-blue-400"
                    placeholder="your@lightning.address"
                />
            </div>
        </div>

        <!-- Save Button -->
        <div class="flex justify-end">
            <button
                type="submit"
                class="button-primary"
            >
                Save Changes
            </button>
        </div>
    </form>
</main>
