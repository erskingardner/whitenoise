import Chats from "../pages/Chats.svelte";
import Calls from "../pages/Calls.svelte";
import Profile from "../pages/Profile.svelte";
import Privacy from "../pages/Privacy.svelte";
import Developer from "../pages/Developer.svelte";
import Settings from "../pages/Settings.svelte";

const routes = [
    {
        path: "/chats/",
        component: Chats,
    },
    {
        path: "/chats/:id/",
        asyncComponent: () => import("../pages/Messages.svelte"),
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
        path: "/settings/profile/",
        component: Profile,
    },
    {
        path: "/settings/privacy/",
        component: Privacy,
    },
    {
        path: "/settings/developer/",
        component: Developer,
    },
];

export default routes;
