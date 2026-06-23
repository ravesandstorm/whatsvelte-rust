<script lang="ts">
  import { openPath } from "@tauri-apps/plugin-opener";
  import type { MediaDto } from "../../lib/types";
  import { api } from "../../lib/ipc";
  import { formatDuration, mediaSrc } from "../../lib/media";
  import { ui } from "../../lib/stores/ui.svelte";

  let { media, thumbnail }: { media: MediaDto; thumbnail: string | null } = $props();

  let src = $state<string | null>(null);
  let loading = $state(false);
  let error = $state(false);

  const thumbUrl = $derived(thumbnail ? `data:image/jpeg;base64,${thumbnail}` : null);
  // Images and stickers load eagerly; heavier media waits for a click.
  const auto = $derived(media.kind === "image" || media.kind === "sticker");

  async function load() {
    if (src || loading) return;
    loading = true;
    error = false;
    try {
      src = await mediaSrc(media);
    } catch (e) {
      console.error("media load failed", e);
      error = true;
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    if (auto) void load();
  });

  // Documents open in the OS default app (no in-app renderer): download to the
  // local cache, then hand the real path to the opener plugin.
  async function openDoc() {
    loading = true;
    error = false;
    try {
      const path = await api.downloadMedia(media.descriptor, media.mimetype);
      await openPath(path);
    } catch (e) {
      console.error("open document failed", e);
      error = true;
    } finally {
      loading = false;
    }
  }

  // Open the full-screen viewer (zoom/pan for images, playback for video).
  function openLightbox() {
    ui.lightboxMedia = media;
  }
</script>

{#if media.kind === "image"}
  <button class="image" onclick={openLightbox} aria-label="Open image">
    {#if src}
      <img src={src} alt="" />
    {:else if thumbUrl}
      <img class="placeholder" src={thumbUrl} alt="" />
    {:else}
      <div class="ph">🖼</div>
    {/if}
  </button>
{:else if media.kind === "sticker"}
  <div class="sticker">
    {#if src}
      <img src={src} alt="sticker" />
    {:else}
      <div class="ph">🟦</div>
    {/if}
  </div>
{:else if media.kind === "video"}
  <button class="poster" onclick={openLightbox} aria-label="Play video">
    {#if thumbUrl}<img src={thumbUrl} alt="" />{/if}
    <span class="play">▶</span>
  </button>
{:else if media.kind === "audio"}
  {#if src}
    <audio src={src} controls autoplay></audio>
  {:else}
    <button class="audio" onclick={load}>
      {loading ? "…" : "▶"} Voice / audio {formatDuration(media.durationSecs)}
    </button>
  {/if}
{:else if media.kind === "document"}
  <button class="doc" onclick={openDoc}>
    📄 <span class="doc-name">{media.fileName ?? "Document"}</span>
  </button>
{/if}

{#if error}<div class="err">Couldn't load media</div>{/if}

<style>
  .image {
    border: none;
    padding: 0;
    background: transparent;
    display: block;
    max-width: 100%;
    cursor: zoom-in;
  }
  .image img,
  .sticker img {
    max-width: 100%;
    border-radius: 6px;
    display: block;
  }
  .sticker img {
    max-width: 140px;
    background: transparent;
  }
  .placeholder {
    filter: blur(6px);
  }
  .ph {
    display: grid;
    place-content: center;
    width: 160px;
    height: 120px;
    background: var(--wa-panel);
    border-radius: 6px;
    font-size: 28px;
  }
  audio {
    width: 240px;
    display: block;
  }
  .poster {
    position: relative;
    border: none;
    padding: 0;
    background: var(--wa-panel);
    border-radius: 6px;
    overflow: hidden;
    display: block;
  }
  .poster img {
    max-width: 100%;
    display: block;
  }
  .play {
    position: absolute;
    inset: 0;
    margin: auto;
    width: 44px;
    height: 44px;
    border-radius: 50%;
    background: rgba(0, 0, 0, 0.55);
    color: #fff;
    display: grid;
    place-content: center;
    font-size: 18px;
  }
  .audio,
  .doc {
    border: none;
    background: var(--wa-panel);
    color: var(--wa-text);
    border-radius: 6px;
    padding: 8px 12px;
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    text-align: left;
  }
  .doc-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .err {
    color: #e06457;
    font-size: 12px;
    margin-top: 2px;
  }
</style>
