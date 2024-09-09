<script lang="ts">
    import { login, createIdentity } from "../../../stores/identities";

    let nsecOrHex: string = "";

    async function handleLogin() {
        await login(nsecOrHex);
        nsecOrHex = "";
    }
</script>

<div class="flex flex-col items-center justify-center w-screen h-screen bg-gray-800">
    <div class="bg-gray-800 w-full md:w-1/2 h-2/3 flex flex-col items-center justify-center gap-6 py-12 px-6">
        <h1 class="text-5xl font-extrabold text-center">White Noise</h1>
        <h2 class="text-3xl font-medium text-center">Secure. Distributed. Uncensorable.</h2>
        <form class="w-full md:w-4/5 flex flex-col gap-2 mt-12">
            <label for="nsec" class="mb-2 flex flex-col gap-2 text-lg items-start font-medium">
                Login in with your nsec
                <input
                    type="text"
                    id="nsec"
                    bind:value={nsecOrHex}
                    placeholder="nsec1..."
                    autocapitalize="off"
                    autocorrect="off"
                    class="bg-transparent ring-1 ring-gray-700 rounded-md px-3 py-2 w-full"
                />
            </label>
            <button
                type="submit"
                onclick={handleLogin}
                class="p-3 font-semibold bg-blue-700 hover:bg-blue-600 rounded-md ring-1 ring-blue-500"
                >Log In</button
            >
        </form>

        <h3 class="font-semibold text-gray-400">OR</h3>
        <button
            class="p-3 w-full md:w-4/5 font-semibold bg-indigo-700 hover:bg-indigo-600 rounded-md ring-1 ring-indigo-500"
            onclick={createIdentity}>Create a new Nostr identity</button
        >
    </div>
    <div class="flex flex-row gap-1 items-end mt-20">
        Powered by
        <img src="/nostr.webp" alt="nostr" class="w-20" />
    </div>
</div>
