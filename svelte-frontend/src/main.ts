import { mount } from "svelte";
import App from "./App.svelte";
import "./app.css";
import { initZoom } from "./lib/zoom";
import { mockEnabled } from "./lib/mock";

initZoom();

async function boot() {
  // Mock backend (dev only): see lib/mock.ts. Installed before mount so the
  // event bridge talks to it from its very first call.
  if (import.meta.env.DEV && mockEnabled()) {
    const { installMock } = await import("./lib/mock");
    await installMock();
  }
  return mount(App, { target: document.getElementById("app")! });
}

void boot();
