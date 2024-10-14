import Chats from "../pages/Chats.svelte";
import Calls from "../pages/Calls.svelte";
import Settings from "../pages/Settings.svelte";
import CreateGroup from "../pages/CreateGroup.svelte";

// Order matters here
const routes = [
    {
        path: "/chats/",
        component: Chats,
    },
    {
        path: "/calls/",
        component: Calls,
    },
    {
        path: "/settings/",
        component: Settings,
    },
    {
        path: "/groups/new/",
        component: CreateGroup,
    },
    {
        path: "/chats/:pubkey/",
        asyncComponent: () => import("../pages/LegacyChat.svelte"),
    },
    {
        path: "/chats/:pubkey/info",
        popup: {
            asyncComponent: () => import("../pages/LegacyChatInfo.svelte"),
        },
    },
    {
        path: "/groups/:mls_group_id/",
        asyncComponent: () => import("../pages/MlsGroup.svelte"),
    },
    {
        path: "/groups/:mls_group_id/group_info",
        popup: {
            asyncComponent: () => import("../pages/MlsGroupInfo.svelte"),
        },
    },
    {
        path: "/contacts/",
        popup: {
            asyncComponent: () => import("../pages/Contacts.svelte"),
        },
    },
];

export default routes;
