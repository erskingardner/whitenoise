<script lang="ts">
import { accounts, activeAccount, setActiveAccount } from "$lib/stores/accounts";
import { PlusCircle } from "phosphor-svelte";
import Avatar from "./Avatar.svelte";
import Modal from "./Modals/Modal.svelte";
import NewAccountModal from "./Modals/NewAccountModal.svelte";
import Name from "./Name.svelte";

let showModal: boolean = $state(false);
let activePubkey: string = $derived($activeAccount?.pubkey as string);
let otherAccounts = $derived(
    $accounts.filter((account) => account.pubkey !== $activeAccount?.pubkey)
);

function openModal() {
    showModal = true;
}

function closeModal() {
    showModal = false;
}
</script>

<div class="flex flex-row justify-between items-center px-4 pb-6">
    <h1 class="text-4xl font-extrabold flex flex-row items-center gap-2 truncate">
        <Avatar pubkey={activePubkey} pxSize={48} />
        <span class="truncate">
            <Name pubkey={activePubkey} unstyled={true} />
        </span>
    </h1>
    <div class="flex flex-row items-center gap-2 ml-16">
        {#each otherAccounts as account (account.pubkey)}
            <button
                onclick={() => setActiveAccount(account.pubkey)}
            >
                <Avatar
                    pubkey={account.pubkey}
                    pxSize={26}
                    showRing={activePubkey === account.pubkey}
                />
            </button>
        {/each}
        <button onclick={openModal} class="p-2 -mr-2 text-gray-300 hover:text-white">
            <PlusCircle size={30} />
        </button>
    </div>
</div>

{#if showModal}
    <Modal initialComponent={NewAccountModal} modalProps={{ closeModal }} bind:showModal />
{/if}
