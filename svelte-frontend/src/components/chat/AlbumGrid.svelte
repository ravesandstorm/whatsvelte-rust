<script lang="ts">
  import type { UiMessage } from "../../lib/stores/messages.svelte";
  import { galleryOf, openLightbox, ui } from "../../lib/stores/ui.svelte";
  import AlbumTile from "./AlbumTile.svelte";

  // WhatsApp-style album preview: up to four square tiles. With more than four
  // items the fourth tile shows "+N" (the items it stands for) and opens the
  // full gallery; the others open the viewer at that item.
  let { items }: { items: UiMessage[] } = $props();

  const shown = $derived(items.slice(0, 4));
  const overflow = $derived(items.length > 4 ? items.length - 3 : 0);

  function open(i: number) {
    const m = items[i];
    if (!m?.media) return;
    const set = galleryOf(items);
    const idx = set.findIndex((g) => g.media === m.media);
    openLightbox(m.media, m.thumbnail, set, Math.max(0, idx));
  }
</script>

<div class="grid" class:two={shown.length === 2} class:three={shown.length === 3}>
  {#each shown as item, i (item.id)}
    {#if i === 3 && overflow > 0}
      <AlbumTile {item} more={overflow} onopen={() => (ui.albumItems = items)} />
    {:else}
      <AlbumTile {item} onopen={() => open(i)} />
    {/if}
  {/each}
</div>

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 3px;
    width: 300px;
    max-width: 100%;
  }
  /* Three items: two on top, one full-width below. */
  .grid.three > :global(:last-child) {
    grid-column: span 2;
    aspect-ratio: 2;
  }
</style>
