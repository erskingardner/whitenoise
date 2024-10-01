import Chats from "../pages/Chats.svelte";
import Calls from "../pages/Calls.svelte";
import Settings from "../pages/Settings.svelte";
import CreateGroup from "../pages/CreateGroup.svelte";

const routes = [
    {
        path: "/chats/",
        component: Chats,
    },
    {
        path: "/chats/:pubkey/",
        asyncComponent: () => import("../pages/Messages.svelte"),
    },
    {
        path: "/contacts/",
        popup: {
            asyncComponent: () => import("../pages/Contacts.svelte"),
        },
    },
    {
        path: "/groups/new/",
        component: CreateGroup,
    },
    {
        path: "/calls/",
        component: Calls,
    },
    {
        path: "/settings/",
        component: Settings,
    },
];

export default routes;
