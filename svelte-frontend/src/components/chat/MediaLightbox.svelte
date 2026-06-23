<script lang="ts">
  import { api } from "../../lib/ipc";
  import { mediaSrc } from "../../lib/media";
  import { ui } from "../../lib/stores/ui.svelte";

  // The viewer is driven entirely by ui.lightboxMedia; rendering is gated on it
  // in MainLayout, but we read it reactively here too so a change re-resolves.
  const media = $derived(ui.lightboxMedia);
  const isVideo = $derived(media?.kind === "video");

  let src = $state<string | null>(null);
  let loadError = $state(false);
  let saving = $state(false);
  let saved = $state(false);

  // Zoom / pan transform (images only).
  let scale = $state(1);
  let tx = $state(0);
  let ty = $state(0);
  let dragging = $state(false);
  let startX = 0;
  let startY = 0;
  let startTx = 0;
  let startTy = 0;

  const MIN = 1;
  const MAX = 8;

  // Resolve the asset URL whenever the open item changes, resetting view state.
  $effect(() => {
    const m = ui.lightboxMedia;
    src = null;
    loadError = false;
    saved = false;
    scale = 1;
    tx = 0;
    ty = 0;
    dragging = false;
    if (!m) return;
    void (async () => {
      try {
        const url = await mediaSrc(m);
        // Guard against a stale resolve after the user moved to another item.
        if (ui.lightboxMedia === m) src = url;
      } catch (e) {
        console.error("lightbox media load failed", e);
        if (ui.lightboxMedia === m) loadError = true;
      }
    })();
  });

  function close() {
    ui.lightboxMedia = null;
  }

  function onBackdropClick(e: MouseEvent) {
    // Close only when the click lands on the empty backdrop, not on the image,
    // video, or toolbar.
    if (e.target === e.currentTarget) close();
  }

  function onWheel(e: WheelEvent) {
    if (isVideo) return;
    e.preventDefault();
    const next = Math.min(MAX, Math.max(MIN, scale - e.deltaY * 0.0015 * scale));
    if (next === MIN) {
      tx = 0;
      ty = 0;
    }
    scale = next;
  }

  function onPointerDown(e: PointerEvent) {
    if (isVideo || scale === MIN) return;
    dragging = true;
    startX = e.clientX;
    startY = e.clientY;
    startTx = tx;
    startTy = ty;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }

  function onPointerMove(e: PointerEvent) {
    if (!dragging) return;
    tx = startTx + (e.clientX - startX);
    ty = startTy + (e.clientY - startY);
  }

  function onPointerUp() {
    dragging = false;
  }

  async function save() {
    if (!media || saving) return;
    saving = true;
    try {
      await api.saveMediaToDownloads(media.descriptor, media.mimetype, media.fileName);
      saved = true;
    } catch (e) {
      console.error("save to downloads failed", e);
    } finally {
      saving = false;
    }
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") close();
  }
</script>

<svelte:window onkeydown={onKey} />

{#if media}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="overlay"
    class:grab={!isVideo && scale > MIN}
    class:grabbing={dragging}
    onclick={onBackdropClick}
    onwheel={onWheel}
    onpointerdown={onPointerDown}
    onpointermove={onPointerMove}
    onpointerup={onPointerUp}
    role="presentation"
  >
    <div class="toolbar">
      <button class="tbtn" onclick={save} disabled={saving} title="Save to Downloads">
        {saved ? "✓ Saved" : saving ? "Saving…" : "⬇ Download"}
      </button>
      <button class="tbtn close" onclick={close} aria-label="Close" title="Close (Esc)">✕</button>
    </div>

    {#if isVideo}
      {#if src}
        <!-- svelte-ignore a11y_media_has_caption -->
        <video src={src} controls autoplay></video>
      {:else if loadError}
        <div class="msg">Couldn't load video</div>
      {:else}
        <div class="msg">Loading…</div>
      {/if}
    {:else if src}
      <img
        class="zoom"
        src={src}
        alt=""
        draggable="false"
        style:transform={`translate(${tx}px, ${ty}px) scale(${scale})`}
      />
    {:else if loadError}
      <div class="msg">Couldn't load image</div>
    {:else}
      <div class="msg">Loading…</div>
    {/if}
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 50;
    display: grid;
    place-items: center;
    background: rgba(0, 0, 0, 0.9);
    overflow: hidden;
    user-select: none;
  }
  .overlay.grab {
    cursor: grab;
  }
  .overlay.grabbing {
    cursor: grabbing;
  }
  .toolbar {
    position: absolute;
    top: 14px;
    right: 16px;
    display: flex;
    gap: 8px;
    z-index: 1;
  }
  .tbtn {
    border: none;
    background: rgba(0, 0, 0, 0.55);
    color: #fff;
    border-radius: 18px;
    padding: 8px 14px;
    font-size: 13px;
    line-height: 1;
    cursor: pointer;
    backdrop-filter: blur(4px);
  }
  .tbtn:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.2);
  }
  .tbtn:disabled {
    opacity: 0.6;
    cursor: default;
  }
  .tbtn.close {
    padding: 8px 12px;
    font-size: 15px;
  }
  .zoom {
    max-width: 92vw;
    max-height: 92vh;
    object-fit: contain;
    /* No transition while dragging/zooming would make panning feel laggy. */
    will-change: transform;
    -webkit-user-drag: none;
  }
  video {
    max-width: 92vw;
    max-height: 92vh;
    outline: none;
  }
  .msg {
    color: var(--wa-text-muted, #aaa);
    font-size: 14px;
  }
</style>
