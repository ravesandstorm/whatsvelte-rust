<script lang="ts">
  import { api } from "../../lib/ipc";
  import { mediaSrc } from "../../lib/media";
  import { ui } from "../../lib/stores/ui.svelte";

  // The viewer is driven entirely by ui.lightboxMedia; rendering is gated on it
  // in MainLayout, but we read it reactively here too so a change re-resolves.
  const media = $derived(ui.lightboxMedia);
  const isVideo = $derived(media?.kind === "video");
  // Instant low-res placeholder while the full media downloads.
  const thumbUrl = $derived(
    ui.lightboxThumbnail ? `data:image/jpeg;base64,${ui.lightboxThumbnail}` : null,
  );

  let src = $state<string | null>(null);
  let loadError = $state(false);
  // True once the full-resolution <img>/<video> has actually decoded — until then
  // we keep the blurred thumbnail + spinner up.
  let fullLoaded = $state(false);
  // Monotonic token so a slow resolve for a previous item can't clobber the
  // current one (more robust than comparing object identity).
  let loadGen = 0;
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
    const gen = ++loadGen;
    src = null;
    loadError = false;
    fullLoaded = false;
    saved = false;
    scale = 1;
    tx = 0;
    ty = 0;
    dragging = false;
    if (!m) return;
    void (async () => {
      try {
        const url = await mediaSrc(m);
        if (gen === loadGen) src = url;
      } catch (e) {
        console.error("lightbox media load failed", e);
        if (gen === loadGen) loadError = true;
      }
    })();
  });

  function close() {
    ui.lightboxMedia = null;
    ui.lightboxThumbnail = null;
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

    <!-- Full-resolution media. Mounted as soon as the asset URL resolves; its
         load/error events drive the overlay below. -->
    {#if src}
      {#if isVideo}
        <!-- svelte-ignore a11y_media_has_caption -->
        <video
          src={src}
          poster={thumbUrl ?? undefined}
          controls
          autoplay
          onloadeddata={() => (fullLoaded = true)}
          onerror={() => (loadError = true)}
        ></video>
      {:else}
        <img
          class="zoom"
          src={src}
          alt=""
          draggable="false"
          style:transform={`translate(${tx}px, ${ty}px) scale(${scale})`}
          onload={() => (fullLoaded = true)}
          onerror={() => (loadError = true)}
        />
      {/if}
    {/if}

    <!-- Overlay shown until the full media has actually decoded: blurred
         thumbnail as an instant preview, a spinner while loading, or an error. -->
    {#if !fullLoaded}
      <div class="loading">
        {#if thumbUrl}<img class="zoom placeholder" src={thumbUrl} alt="" draggable="false" />{/if}
        {#if loadError}
          <div class="msg chip">Couldn't load {isVideo ? "video" : "image"}</div>
        {:else}
          <div class="spinner"></div>
        {/if}
      </div>
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
  /* Overlay (blurred thumbnail + spinner/error) sitting over the full media
     until it decodes. Children stack in one centered grid cell. */
  .loading {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    pointer-events: none;
  }
  .loading > * {
    grid-area: 1 / 1;
  }
  /* Low-res thumbnail shown (blurred) until the full media loads. The slight
     scale hides the blur bleeding past the edges. */
  .placeholder {
    filter: blur(14px);
    transform: scale(1.06);
  }
  .spinner {
    width: 42px;
    height: 42px;
    border: 3px solid rgba(255, 255, 255, 0.25);
    border-top-color: #fff;
    border-radius: 50%;
    animation: spin 0.9s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
  .msg {
    color: var(--wa-text-muted, #aaa);
    font-size: 14px;
  }
  .chip {
    padding: 6px 14px;
    border-radius: 16px;
    background: rgba(0, 0, 0, 0.6);
    color: #fff;
  }
</style>
