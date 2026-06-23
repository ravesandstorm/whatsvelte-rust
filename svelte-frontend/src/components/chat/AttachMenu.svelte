<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { readFile } from "@tauri-apps/plugin-fs";
  import { basename } from "@tauri-apps/api/path";
  import { mimeForPath, type PendingAttachment } from "../../lib/send-media";

  let {
    onpick,
    onrecord,
  }: {
    onpick: (a: PendingAttachment) => void;
    onrecord: (mode: "voice" | "video" | "photo") => void;
  } = $props();

  let menuOpen = $state(false);

  function record(mode: "voice" | "video" | "photo") {
    menuOpen = false;
    onrecord(mode);
  }

  async function pickPhoto() {
    menuOpen = false;
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg", "gif", "webp", "bmp"] }],
    });
    if (typeof selected !== "string") return;
    const name = await basename(selected);
    const bytes = await readFile(selected);
    const blob = new Blob([bytes], { type: mimeForPath(name) ?? "image/jpeg" });
    onpick({
      kind: "image",
      blob,
      fileName: name,
      previewUrl: URL.createObjectURL(blob),
    });
  }

  async function pickDocument() {
    menuOpen = false;
    const selected = await open({ multiple: false, directory: false });
    if (typeof selected !== "string") return;
    const name = await basename(selected);
    onpick({
      kind: "document",
      path: selected,
      fileName: name,
      mimetype: mimeForPath(name) ?? undefined,
    });
  }
</script>

<svelte:window onclick={() => (menuOpen = false)} />

<div class="attach" role="presentation" onclick={(e) => e.stopPropagation()}>
  {#if menuOpen}
    <div class="menu">
      <button class="item" onclick={pickPhoto}><span class="ico">🖼️</span>Photos</button>
      <button class="item" onclick={pickDocument}><span class="ico">📄</span>Document</button>
      <button class="item" onclick={() => record("photo")}><span class="ico">📷</span>Camera</button>
      <button class="item" onclick={() => record("video")}><span class="ico">🎬</span>Record video</button
      >
      <button class="item" onclick={() => record("voice")}><span class="ico">🎤</span>Voice message</button
      >
    </div>
  {/if}
  <button
    class="icon"
    aria-label="Attach"
    class:active={menuOpen}
    onclick={() => (menuOpen = !menuOpen)}>📎</button
  >
</div>

<style>
  .attach {
    position: relative;
    flex-shrink: 0;
  }
  .icon {
    width: 40px;
    height: 40px;
    border: none;
    border-radius: 50%;
    background: transparent;
    font-size: 20px;
    color: var(--wa-text-muted);
  }
  .icon:hover,
  .icon.active {
    background: var(--wa-panel-2);
    color: var(--wa-text);
  }
  .menu {
    position: absolute;
    bottom: calc(100% + 8px);
    left: 0;
    z-index: 1000;
    min-width: 170px;
    background: var(--wa-panel-2);
    border: 1px solid var(--wa-border);
    border-radius: 8px;
    padding: 4px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  }
  .item {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    border: none;
    background: transparent;
    color: var(--wa-text);
    padding: 9px 12px;
    border-radius: 6px;
    text-align: left;
    font-size: 14px;
  }
  .item:hover {
    background: var(--wa-hover);
  }
  .ico {
    font-size: 18px;
  }
</style>
