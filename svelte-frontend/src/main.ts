import { mount } from "svelte";
import App from "./App.svelte";
import "./app.css";
import { initZoom } from "./lib/zoom";

initZoom();

const app = mount(App, { target: document.getElementById("app")! });

export default app;
