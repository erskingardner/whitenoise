<script lang="ts">
    import Avatar from "./Avatar.svelte";
    import Name from "./Name.svelte";
    import type { EnrichedContact } from "$lib/types/nostr";
    import { nameFromMetadata } from "$lib/utils/nostr";
    import { accounts } from "$lib/stores/accounts";
    import { invoke } from "@tauri-apps/api/core";

    let { pubkey, contact } = $props<{
        pubkey: string;
        contact: EnrichedContact;
    }>();

    async function startSecureChat() {
        await invoke("create_group", {
            creatorPubkey: $accounts.activeAccount,
            memberPubkeys: [pubkey],
            adminPubkeys: [$accounts.activeAccount, pubkey],
            groupName: "Secure DM",
            description: "",
        });
    }

    async function inviteToWhiteNoise() {
        // await invoke("invite_to_white_noise", { pubkey });
    }
</script>

<div>
    <div class="flex flex-col items-center justify-start gap-2">
        <Avatar {pubkey} picture={contact.metadata.picture} pxSize={80} />
        <div class="text-4xl font-bold mt-6">
            <Name {pubkey} metadata={contact.metadata} unstyled={true} />
        </div>
        <p class="text-gray-500">
            {contact.metadata.about || "No bio set"}
        </p>
    </div>

    <div class="flex flex-col gap-4 mt-10 justify-center items-center">
        {#if contact.nip104}
            <p>
                White Noise uses MLS (messaging layer security). Your messages are end-to-end
                encrypted and can only be read by you and the other participant.
            </p>
            <p class="mt-4">
                Ready to invite {nameFromMetadata(contact.metadata, pubkey)} to start a secure chat?
            </p>
            <button class="w-full py-2 px-4 bg-blue-600 rounded-md mt-4">
                Start secure chat
            </button>
        {:else}
            <p>
                {nameFromMetadata(contact.metadata, pubkey)} is not yet set up to use secure MLS messaging.
            </p>

            <button class="py-2 px-4 bg-blue-600 rounded-md mt-4" onclick={inviteToWhiteNoise}>
                Invite {nameFromMetadata(contact.metadata, pubkey)} to White Noise
            </button>
        {/if}
    </div>
</div>
