<script lang="ts">
  import { api } from "../../lib/ipc";
  import { session } from "../../lib/stores/session.svelte";
  import { ui } from "../../lib/stores/ui.svelte";
  import {
    settings,
    setEnterToSend,
    setSendReadReceipts,
    setWallpaper,
  } from "../../lib/stores/settings.svelte";
  import { jidUser } from "../../lib/util/jid";
  import { zoomIn, zoomOut, zoomReset } from "../../lib/zoom";
  import { themeState, setTheme } from "../../lib/stores/theme.svelte";

  // A few preset wallpaper colors/gradients plus "none" and a custom image.
  const PRESETS: { id: string; label: string; value: string | null }[] = [
    { id: "default", label: "Default", value: null },
    { id: "teal", label: "Teal", value: "#0b141a" },
    { id: "slate", label: "Slate", value: "#1f2c33" },
    {
      id: "dusk",
      label: "Dusk",
      value: "linear-gradient(160deg, #0b141a 0%, #14323a 100%)",
    },
  ];

  function onWallpaperFile(e: Event) {
    const file = (e.target as HTMLInputElement).files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => setWallpaper(String(reader.result));
    reader.readAsDataURL(file);
  }

  async function logout() {
    try {
      await api.logout();
    } catch (err) {
      console.error("logout failed", err);
    }
    ui.settingsOpen = false;
  }
</script>

<div class="overlay" onclick={() => (ui.settingsOpen = false)} role="presentation"></div>
<aside class="panel">
  <header>
    <button class="back" aria-label="Close" onclick={() => (ui.settingsOpen = false)}>←</button>
    <h2>Settings</h2>
  </header>

  <div class="body">
    <section>
      <h3>Account</h3>
      <p class="muted">{session.jid ? jidUser(session.jid) : "Not linked"}</p>
      <button class="danger" onclick={logout}>Log out</button>
    </section>

    <section>
      <h3>Appearance</h3>
      <label class="row">
        <span>Theme</span>
        <span class="seg">
          <button class:active={themeState.theme === "light"} onclick={() => setTheme("light")}
            >Light</button
          >
          <button class:active={themeState.theme === "dark"} onclick={() => setTheme("dark")}
            >Dark</button
          >
        </span>
      </label>
      <label class="row">
        <span>Zoom</span>
        <span class="zoom">
          <button onclick={zoomOut} aria-label="Zoom out">−</button>
          <button onclick={zoomReset}>Reset</button>
          <button onclick={zoomIn} aria-label="Zoom in">+</button>
        </span>
      </label>
      <div class="wallpaper">
        <span>Wallpaper</span>
        <div class="presets">
          {#each PRESETS as p (p.id)}
            <button
              class="swatch"
              class:active={settings.wallpaper === p.value}
              style:background={p.value ?? "var(--wa-chat-bg)"}
              title={p.label}
              onclick={() => setWallpaper(p.value)}
            ></button>
          {/each}
          <label class="swatch upload" title="Custom image">
            🖼
            <input type="file" accept="image/*" onchange={onWallpaperFile} hidden />
          </label>
        </div>
      </div>
    </section>

    <section>
      <h3>Chats</h3>
      <label class="row">
        <span>Enter sends message</span>
        <input
          type="checkbox"
          checked={settings.enterToSend}
          onchange={(e) => setEnterToSend(e.currentTarget.checked)}
        />
      </label>
    </section>

    <section>
      <h3>Privacy</h3>
      <label class="row">
        <span>Send read receipts</span>
        <input
          type="checkbox"
          checked={settings.sendReadReceipts}
          onchange={(e) => setSendReadReceipts(e.currentTarget.checked)}
        />
      </label>
      <p class="muted small">
        When off, your blue ticks aren't sent. (Whole-chat read state may still
        sync from your phone.)
      </p>
    </section>
  </div>
</aside>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    z-index: 30;
  }
  .panel {
    position: fixed;
    top: 0;
    left: 0;
    width: 380px;
    max-width: 90vw;
    height: 100%;
    background: var(--wa-bg);
    border-right: 1px solid var(--wa-border);
    z-index: 31;
    display: flex;
    flex-direction: column;
  }
  header {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 20px 16px;
    background: var(--wa-panel);
  }
  header h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 500;
  }
  .back {
    border: none;
    background: transparent;
    color: var(--wa-text);
    font-size: 20px;
  }
  .body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 8px 0;
  }
  section {
    padding: 16px;
    border-bottom: 1px solid var(--wa-border);
  }
  h3 {
    margin: 0 0 12px;
    font-size: 13px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--wa-green);
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 6px 0;
  }
  .muted {
    color: var(--wa-text-muted);
  }
  .small {
    font-size: 12px;
    margin: 8px 0 0;
  }
  .danger {
    margin-top: 10px;
    border: 1px solid #c0392b;
    background: transparent;
    color: #e06457;
    padding: 8px 14px;
    border-radius: 6px;
  }
  .zoom {
    display: flex;
    gap: 6px;
  }
  .zoom button {
    border: 1px solid var(--wa-border);
    background: var(--wa-panel-2);
    color: var(--wa-text);
    border-radius: 6px;
    padding: 4px 10px;
  }
  .seg {
    display: inline-flex;
    border: 1px solid var(--wa-border);
    border-radius: 8px;
    overflow: hidden;
  }
  .seg button {
    border: none;
    background: var(--wa-panel-2);
    color: var(--wa-text-muted);
    padding: 5px 14px;
    font-size: 13px;
  }
  .seg button.active {
    background: var(--wa-green);
    color: #04221c;
    font-weight: 600;
  }
  .wallpaper {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding-top: 6px;
  }
  .presets {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
  }
  .swatch {
    width: 44px;
    height: 44px;
    border-radius: 8px;
    border: 2px solid var(--wa-border);
    cursor: pointer;
    display: grid;
    place-content: center;
    font-size: 18px;
    color: var(--wa-text-muted);
  }
  .swatch.active {
    border-color: var(--wa-green);
  }
  .upload {
    background: var(--wa-panel-2);
  }
</style>
