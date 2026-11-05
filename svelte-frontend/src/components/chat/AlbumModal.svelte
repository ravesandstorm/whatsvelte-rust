<script lang="ts">
  import { galleryOf, openLightbox, ui } from "../../lib/stores/ui.svelte";
  import AlbumTile from "./AlbumTile.svelte";

  // Every item of an album, as a scrollable grid. Picking one opens the viewer
  // on it, with prev/next through the album.
  const items = $derived(ui.albumItems ?? []);

  function close() {
    ui.albumItems = null;
  }

  function open(i: number) {
    const m = items[i];
    if (!m?.media) return;
    const set = galleryOf(items);
    const idx = set.findIndex((g) => g.media === m.media);
    openLightbox(m.media, m.thumbnail, set, Math.max(0, idx));
  }

  function onKey(e: KeyboardEvent) {
    // The viewer, when open on top, handles Escape itself.
    if (e.key === "Escape" && !ui.lightboxMedia) close();
  }
</script>

<svelte:window onkeydown={onKey} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="backdrop" onclick={close} role="presentation">
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div
    class="panel"
    role="dialog"
    aria-modal="true"
    aria-label="Album"
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
  >
    <header>
      <span>{items.length} {items.some((m) => m.kind === "video") ? "items" : "photos"}</span>
      <button class="close" onclick={close} aria-label="Close">✕</button>
    </header>
    <div class="grid">
      {#each items as item, i (item.id)}
        <AlbumTile {item} onopen={() => open(i)} />
      {/each}
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 40;
    background: rgba(0, 0, 0, 0.7);
    display: grid;
    place-items: center;
  }
  .panel {
    width: min(720px, calc(100vw - 32px));
    max-height: calc(100vh - 64px);
    background: var(--wa-panel);
    border-radius: 10px;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    color: var(--wa-text);
    font-size: 15px;
    border-bottom: 1px solid var(--wa-border);
  }
  .close {
    border: none;
    background: transparent;
    color: var(--wa-text-muted);
    font-size: 16px;
    cursor: pointer;
  }
  .grid {
    overflow-y: auto;
    padding: 10px;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 6px;
  }
</style>
