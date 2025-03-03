<script lang="ts">
import { goto } from "$app/navigation";
import Header from "$lib/components/Header.svelte";
import HeaderToolbar from "$lib/components/HeaderToolbar.svelte";
import { activeAccount } from "$lib/stores/accounts";
import { getToastState } from "$lib/stores/toast-state.svelte";
import type { EnrichedContact, NMetadata } from "$lib/types/nostr";
import { invoke } from "@tauri-apps/api/core";
import { CaretLeft } from "phosphor-svelte";
import { get } from "svelte/store";

function goBack() {
    goto("/settings");
}

let metadata: NMetadata = $state({});
let toastState = getToastState();

let displayName = $state("");
let name = $state("");
let about = $state("");
let website = $state("");
let nip05 = $state("");
let lud16 = $state("");
let banner = $state("");
let picture = $state("");

let pictureLoading = $state(false);
let bannerLoading = $state(false);

// Load the current user's metadata
$effect(() => {
    const account = get(activeAccount);
    if (account?.pubkey) {
        invoke<EnrichedContact>("query_enriched_contact", {
            pubkey: account.pubkey,
            updateAccount: false,
        }).then((response) => {
            metadata = response?.metadata ?? {};
            displayName = metadata.display_name ?? "";
            name = metadata.name ?? "";
            about = metadata.about ?? "";
            website = metadata.website ?? "";
            nip05 = metadata.nip05 ?? "";
            lud16 = metadata.lud16 ?? "";
            banner = metadata.banner ?? "";
            picture = metadata.picture ?? "";
        });
    }
});

$inspect(metadata);

async function uploadImageToNip96(file: File): Promise<string> {
    // Fetch the NIP-96 endpoint info
    const response = await fetch("https://nostr.build/.well-known/nostr/nip96.json");
    if (!response.ok) {
        throw new Error("Failed to fetch NIP-96 endpoint info");
    }
    const nip96Info = await response.json();

    // Get the API URL from the NIP-96 info
    const apiUrl = nip96Info.api_url;
    if (!apiUrl) {
        throw new Error("No API URL found in NIP-96 info");
    }

    // Get the download URL
    const baseDownloadUrl = nip96Info.download_url;

    // Create form data with a specific boundary for reproducible hashing
    const boundary = "whitenoise-upload-boundary";

    // Manually construct the multipart form-data body to ensure consistent hashing
    const encoder = new TextEncoder();
    const fileArrayBuffer = await file.arrayBuffer();
    const fileBytes = new Uint8Array(fileArrayBuffer);

    // Construct the multipart form-data manually
    const bodyParts = [
        `--${boundary}\r\n`,
        `Content-Disposition: form-data; name="file"; filename="${file.name}"\r\n`,
        `Content-Type: ${file.type}\r\n\r\n`,
    ];

    // Convert text parts to Uint8Arrays
    const bodyPartsBytes = bodyParts.map((part) => encoder.encode(part));

    // Calculate total length
    const totalLength =
        bodyPartsBytes.reduce((acc, part) => acc + part.length, 0) +
        fileBytes.length +
        encoder.encode(`\r\n--${boundary}--\r\n`).length;

    // Create a single Uint8Array for the entire body
    const bodyBytes = new Uint8Array(totalLength);
    let offset = 0;

    // Copy all parts into the final array
    for (const part of bodyPartsBytes) {
        bodyBytes.set(part, offset);
        offset += part.length;
    }
    bodyBytes.set(fileBytes, offset);
    offset += fileBytes.length;
    bodyBytes.set(encoder.encode(`\r\n--${boundary}--\r\n`), offset);

    // Hash the complete body
    const hashBuffer = await crypto.subtle.digest("SHA-256", bodyBytes);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    const hashHex = hashArray.map((b) => b.toString(16).padStart(2, "0")).join("");

    // Make sure the URL is properly formatted
    const fullUrl = new URL(apiUrl);

    // Generate NIP-98 authorization token with the complete body hash
    const authEvent = await invoke<string>("generate_nip98_auth_token", {
        url: fullUrl.toString(),
        method: "POST",
        payload: hashHex,
    });

    // Base64 encode the auth event
    const authEventBase64 = btoa(authEvent);

    // Upload the file using the exact same body we hashed
    const uploadResponse = await fetch(fullUrl.toString(), {
        method: "POST",
        headers: {
            Authorization: `Nostr ${authEventBase64}`,
            Accept: "application/json",
            "Content-Type": `multipart/form-data; boundary=${boundary}`,
        },
        body: bodyBytes,
    });

    if (!uploadResponse.ok) {
        const errorText = await uploadResponse.text();
        console.error("Upload failed with status:", uploadResponse.status);
        console.error("Error response:", errorText);
        console.error("Auth token used:", authEventBase64);
        throw new Error(`Failed to upload image (${uploadResponse.status}): ${errorText}`);
    }

    const result = await uploadResponse.json();
    if (!result.nip94_event?.tags) {
        throw new Error("Invalid response from server");
    }

    // Find the URL tag in the response
    const urlTag = result.nip94_event.tags.find((tag: string[]) => tag[0] === "url");
    if (!urlTag || !urlTag[1]) {
        throw new Error("No URL found in response");
    }

    // If the URL is already absolute, use it as is
    if (urlTag[1].startsWith("http://") || urlTag[1].startsWith("https://")) {
        return urlTag[1];
    }

    // Combine the base download URL with the relative path
    const cleanBaseUrl = baseDownloadUrl.replace(/\/$/, "");
    const cleanPath = urlTag[1].replace(/^\//, "");
    console.log(`${cleanBaseUrl}/${cleanPath}`);
    return `${cleanBaseUrl}/${cleanPath}`;
}

async function handleImageUpload(event: Event, field: "picture" | "banner") {
    const input = event.target as HTMLInputElement;
    if (input.files?.[0]) {
        const file = input.files[0];
        try {
            // Set loading state
            if (field === "banner") {
                bannerLoading = true;
            } else {
                pictureLoading = true;
            }

            // Upload the file and get the URL
            const url = await uploadImageToNip96(file);
            console.log("URL of uploaded image", url);
            // Update metadata with the actual URL from nostr.build
            if (field === "banner") {
                banner = url;
            } else {
                picture = url;
            }
        } catch (error) {
            console.error("Error uploading image:", error);
            toastState.add("Error", `Failed to upload ${field}: ${error}`, "error");

            // Reset the field if upload failed
            metadata[field] = field === "banner" ? banner : picture;
        } finally {
            // Clear the file input
            input.value = "";
            if (field === "banner") {
                bannerLoading = false;
            } else {
                pictureLoading = false;
            }
            document.getElementById("profile-form")?.dispatchEvent(new Event("submit"));
        }
    }
}

async function handleSubmit(event: Event) {
    event.preventDefault();

    try {
        // No need to upload images here anymore since they're uploaded immediately on selection
        let newMetadata = {
            display_name: displayName,
            name: name,
            about: about,
            website: website,
            nip05: nip05,
            lud16: lud16,
            banner: banner,
            picture: picture,
        };

        // Publish metadata update
        await invoke("publish_metadata", { metadata: newMetadata });
        toastState.add("Success", "Profile updated successfully", "success");
    } catch (error) {
        console.error("Error updating profile:", error);
        toastState.add("Error", `Failed to update profile: ${error}`, "error");
    }
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
        <h1>Your Profile</h1>
    {/snippet}
</HeaderToolbar>

<Header title="Your Profile" />

<main class="px-4 flex flex-col pb-32 gap-6">
    <!-- Banner Image -->
    <div class="relative w-full h-48 bg-gray-100 dark:bg-gray-800 rounded-lg overflow-hidden">
        {#if banner}
            <img src={banner} alt="Profile banner" class="w-full h-full object-cover" />
        {/if}
        <label class="absolute bottom-4 right-4 bg-white dark:bg-gray-700 px-4 py-2 rounded-lg cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors flex flex-row gap-2 items-center">
            <input type="file" accept="image/*" class="hidden" onchange={(e) => handleImageUpload(e, 'banner')} />
            Change Banner
            {#if bannerLoading}
                <div class="animate-spin rounded-full h-4 w-4 border-2 border-primary border-t-transparent"></div>
            {/if}
        </label>
    </div>

    <!-- Avatar Image -->
    <div class="relative w-32 h-32 mx-auto -mt-16">
        <div class="w-full h-full rounded-full bg-gray-100 dark:bg-gray-800 overflow-hidden border-4 border-white dark:border-gray-900">
            {#if pictureLoading}
                <div class="w-full h-full flex items-center justify-center">
                    <div class="animate-spin rounded-full h-8 w-8 border-4 border-primary border-t-transparent"></div>
                </div>
            {:else if picture}
                <img src={picture} alt="profile" class="w-full h-full object-cover" />
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
    <form id="profile-form" class="space-y-6 max-w-2xl mx-auto w-full" onsubmit={handleSubmit}>
        <div class="space-y-4">
            <!-- Display Name -->
            <div>
                <label for="display_name" class="block text-sm font-medium mb-1">Display Name</label>
                <input
                    type="text"
                    id="display_name"
                    bind:value={displayName}
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
                    bind:value={name}
                    class="w-full px-4 py-2 rounded-lg border dark:border-gray-700 bg-white dark:bg-gray-800 focus:ring-2 focus:ring-blue-500 dark:focus:ring-blue-400"
                    placeholder="Your username"
                />
            </div>

            <!-- About -->
            <div>
                <label for="about" class="block text-sm font-medium mb-1">About</label>
                <textarea
                    id="about"
                    bind:value={about}
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
                    bind:value={website}
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
                    bind:value={nip05}
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
                    bind:value={lud16}
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
