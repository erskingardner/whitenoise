<script lang="ts">
    import { ChatCircle, Phone, Gear } from "phosphor-svelte";

    let { activeTab = $bindable("chats") }: { activeTab: string } = $props();
</script>

<nav
    class="fixed bottom-0 left-0 right-0 border-t border-gray-700 py-4 min-w-96 overflow-hidden md:hidden bg-gray-950/80 backdrop-blur-sm z-10"
>
    <div class="max-w-screen-xl mx-auto px-4 flex justify-around items-center">
        <a href="/chats">
            <span class="tabbar-link {activeTab === 'chats' ? 'active' : ''}">
                <ChatCircle size={28} weight={activeTab === "chats" ? "fill" : "light"} />
                Chats
            </span>
        </a>
        <a href="/calls">
            <span class="tabbar-link {activeTab === 'calls' ? 'active' : ''}">
                <Phone size={28} weight={activeTab === "calls" ? "fill" : "light"} />
                Calls
            </span>
        </a>
        <a href="/settings">
            <span class="tabbar-link {activeTab === 'settings' ? 'active' : ''}">
                <Gear size={28} weight={activeTab === "settings" ? "fill" : "light"} />
                Settings
            </span>
        </a>
    </div>
</nav>

<style lang="postcss">
    .tabbar-link {
        @apply flex flex-col items-center font-medium text-lg gap-2;
    }
    .tabbar-link.active {
        @apply text-blue-500;
    }
</style>
