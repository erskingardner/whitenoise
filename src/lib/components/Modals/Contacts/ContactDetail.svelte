<script lang="ts">
    import Avatar from "../../Avatar.svelte";
    import Name from "../../Name.svelte";
    import type { EnrichedContact } from "$lib/types/nostr";
    import { nameFromMetadata } from "$lib/utils/nostr";
    import { accounts } from "$lib/stores/accounts";
    import { invoke } from "@tauri-apps/api/core";
    import { getToastState } from "$lib/stores/toast-state.svelte";
    import type { CloseModal } from "$lib/types/modal";

    let toastState = getToastState();

    let { pubkey, contact, closeModal } = $props<{
        pubkey: string;
        contact: EnrichedContact;
        closeModal: CloseModal;
    }>();

    async function startSecureChat() {
        await invoke("create_group", {
            creatorPubkey: $accounts.activeAccount,
            memberPubkeys: [pubkey],
            adminPubkeys: [$accounts.activeAccount, pubkey],
            groupName: "Secure DM",
            description: "",
        })
            .then((group) => {
                console.log("Group created", group);
                toastState.add("Group created", "Group created successfully", "success");
                setTimeout(() => {
                    closeModal();
                }, 1000);
            })
            .catch((e) => {
                toastState.add("Error creating group", e.toString(), "error");
                console.error("Error creating group", e);
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
            {contact.metadata.about}
        </p>
    </div>

    <div class="flex flex-col gap-10 mt-10 items-center w-full md:w-2/3 lg:w-1/2 mx-auto">
        {#if contact.nip104}
            <p>
                White Noise uses MLS (messaging layer security). Your messages are end-to-end encrypted and can only be
                read by you and the other participant.
            </p>
            <p class="mt-4">
                Ready to invite {nameFromMetadata(contact.metadata, pubkey)} to start a secure chat?
            </p>
            <button class="button-primary" onclick={startSecureChat}> Start secure chat </button>
        {:else}
            <p>
                {nameFromMetadata(contact.metadata, pubkey)} is not yet set up to use secure MLS messaging.
            </p>

            <button class="button-primary" onclick={inviteToWhiteNoise}>
                Invite {nameFromMetadata(contact.metadata, pubkey)} to White Noise
            </button>
        {/if}
    </div>
</div>
