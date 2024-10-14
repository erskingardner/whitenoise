import { mount } from "svelte";
import App from "../App.svelte";

import Framework7Svelte from "framework7-svelte";
import Framework7 from "./framework7-custom";

import "../css/framework7-custom.less";
import "../css/app.css";

// Init F7 Svelte Plugin
Framework7.use(Framework7Svelte);

// Mount Svelte App
const app = mount(App, { target: document.body });

export default app;
