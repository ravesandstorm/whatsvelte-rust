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
  let saving = $state(false);

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
      const path = await api.downloadMedia(media.descriptor, media.mimetype, media.fileName);
      await openPath(path);
    } catch (e) {
      console.error("open document failed", e);
      error = true;
    } finally {
      loading = false;
    }
  }

  // Save a copy into the OS Downloads folder (reuses the existing command).
  async function downloadDoc() {
    if (saving) return;
    saving = true;
    error = false;
    try {
      await api.saveMediaToDownloads(media.descriptor, media.mimetype, media.fileName);
    } catch (e) {
      console.error("download document failed", e);
      error = true;
    } finally {
      saving = false;
    }
  }

  // Open the full-screen viewer (zoom/pan for images, playback for video).
  // Pass the thumbnail too so the viewer shows it instantly while the
  // full-resolution media downloads.
  function openLightbox() {
    ui.lightboxThumbnail = thumbnail;
    ui.lightboxMedia = media;
  }

  // Audio starts playing only right after the user asked for it (the load
  // button), not whenever the element re-mounts with a cached src, and only one
  // clip plays at a time.
  let playOnLoad = $state(false);
  function onAudioPlay(e: Event) {
    const el = e.currentTarget as HTMLAudioElement;
    document.querySelectorAll("audio").forEach((a) => {
      if (a !== el && !a.paused) a.pause();
    });
    playOnLoad = false;
  }
</script>

{#if media.kind === "image"}
  <button class="image" onclick={openLightbox} aria-label="Open image">
    {#if src}
      <img src={src} alt="" />
    {:else}
      <div class="image-loading">
        {#if thumbUrl}
          <img class="placeholder" src={thumbUrl} alt="" />
        {/if}
        {#if loading}
          <div class="spinner-overlay"><div class="media-spinner"></div></div>
        {:else if !thumbUrl}
          <div class="ph">🖼</div>
        {/if}
      </div>
    {/if}
  </button>
{:else if media.kind === "sticker"}
  <div class="sticker">
    {#if src}
      <img src={src} alt="sticker" />
    {:else}
      <div class="sticker-loading">
        <div class="media-spinner"></div>
      </div>
    {/if}
  </div>
{:else if media.kind === "video"}
  <button class="poster" onclick={openLightbox} aria-label="Play video">
    {#if thumbUrl}<img src={thumbUrl} alt="" />{/if}
    {#if loading}
      <div class="spinner-overlay"><div class="media-spinner"></div></div>
    {:else}
      <span class="play">▶</span>
    {/if}
  </button>
{:else if media.kind === "audio"}
  {#if src}
    <audio src={src} controls autoplay={playOnLoad} onplay={onAudioPlay}></audio>
  {:else}
    <button
      class="audio"
      onclick={() => {
        playOnLoad = true;
        void load();
      }}
    >
      {#if loading}
        <div class="media-spinner sm"></div>
      {:else}
        ▶
      {/if}
      Voice / audio {formatDuration(media.durationSecs)}
    </button>
  {/if}
{:else if media.kind === "document"}
  <div class="doc">
    <button class="doc-open" onclick={openDoc} disabled={loading}>
      {#if loading}
        <div class="media-spinner sm"></div>
      {:else}
        📄
      {/if}
      <span class="doc-name">{media.fileName ?? "Document"}</span>
    </button>
    <button class="doc-dl" onclick={downloadDoc} disabled={saving} title="Download" aria-label="Download">
      {saving ? "…" : "⬇"}
    </button>
  </div>
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
  /* Keep inline images modestly sized; the lightbox shows full resolution. */
  .image img {
    max-width: 300px;
    max-height: 360px;
  }
  .sticker img {
    max-width: 140px;
    background: transparent;
  }
  .image-loading {
    position: relative;
    display: inline-block;
    min-width: 140px;
    min-height: 100px;
    border-radius: 6px;
    overflow: hidden;
  }
  .sticker-loading {
    width: 120px;
    height: 120px;
    display: grid;
    place-content: center;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 8px;
  }
  .spinner-overlay {
    position: absolute;
    inset: 0;
    margin: auto;
    display: grid;
    place-content: center;
    background: rgba(0, 0, 0, 0.35);
    border-radius: 6px;
    z-index: 2;
  }
  .media-spinner {
    width: 26px;
    height: 26px;
    border: 3px solid rgba(255, 255, 255, 0.25);
    border-top-color: var(--wa-unread, #00a884);
    border-radius: 50%;
    animation: media-spin 0.8s linear infinite;
  }
  .media-spinner.sm {
    width: 16px;
    height: 16px;
    border-width: 2px;
  }
  @keyframes media-spin {
    to {
      transform: rotate(360deg);
    }
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
    width: 280px;
    min-height: 160px;
  }
  .poster img {
    width: 100%;
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
  .doc-open {
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
  .doc {
    display: flex;
    align-items: stretch;
    gap: 4px;
    width: 100%;
  }
  .doc-open {
    flex: 1;
    min-width: 0;
  }
  .doc-dl {
    flex-shrink: 0;
    border: none;
    background: var(--wa-panel);
    color: var(--wa-text);
    border-radius: 6px;
    padding: 8px 12px;
    font-size: 15px;
  }
  .doc-dl:hover {
    background: var(--wa-hover);
  }
  .doc-dl:disabled {
    opacity: 0.5;
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
