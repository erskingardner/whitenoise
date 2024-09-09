<script lang="ts">
    import { identities, switchIdentity, createIdentity, login, logout } from "../../../../stores/identities";
    import Avatar from "../../../../components/Avatar.svelte";
    import Name from "../../../../components/Name.svelte";
    import { SignOut, PlusCircle, SignIn } from "phosphor-svelte";

    let nsecOrHex = "";

    async function handleLogin() {
        await login(nsecOrHex);
        nsecOrHex = ""; // Clear the input field
    }
</script>

<h1 class="text-xl font-semibold mb-6">Profile</h1>
<div class="flex flex-col gap-12 items-start">
    <div class="flex flex-col gap-6 w-full">
        {#each $identities as id}
            <div class="rounded-lg bg-gray-800 flex flex-row items-center justify-between p-4">
                <div class="flex flex-row gap-6 items-center">
                    <button onclick={() => switchIdentity(id.pubkey)}>
                        <Avatar pubkey={id.pubkey} pxSize={40} />
                    </button>
                    <Name pubkey={id.pubkey} />
                </div>
                <button onclick={() => logout(id.pubkey)} title="Logout from this account" class="flex flex-row gap-2 items-center px-3 py-2 rounded-lg bg-gray-700 hover:bg-gray-600 ring-1 ring-gray-500">
                    <SignOut size="2rem" weight="thin" />
                    Sign out
                </button>
            </div>
        {/each}
    </div>
    <div class="flex flex-col gap-6 items-start w-full">
        <form class="flex flex-col gap-2 w-full items-start">
            <label for="nsec" class="mb-2 flex flex-col gap-2 text-lg items-start font-medium w-full">
                Login in with your nsec
                <input
                    type="password"
                    id="nsec"
                    bind:value={nsecOrHex}
                    placeholder="nsec1..."
                    autocapitalize="off"
                    autocorrect="off"
                    class="bg-transparent ring-1 ring-gray-700 rounded-md px-3 py-3 w-full"
                />
            </label>
            <button
                type="submit"
                onclick={handleLogin}
                class="px-3 py-2 flex flex-row shrink items-center justify-start gap-2 font-semibold bg-blue-700 hover:bg-blue-600 rounded-md ring-1 ring-blue-500"
                >
                <SignIn size="2rem" weight="thin" />
                Log In
            </button>
        </form>
    </div>
    <button onclick={createIdentity} class="px-3 py-2 text-center flex flex-row items-center gap-2 rounded-md bg-gray-700 hover:bg-gray-600 ring-1 ring-gray-500">
        <PlusCircle size="2rem" weight="thin" />
        Create New Nostr Identity
    </button>
</div>
