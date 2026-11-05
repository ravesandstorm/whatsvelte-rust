<script lang="ts">
  import type { UiMessage } from "../../lib/stores/messages.svelte";
  import { mediaSrc } from "../../lib/media";

  // One square tile of an album grid / gallery: the inline thumbnail at once,
  // swapped for the full image once it's cached. Videos keep their poster.
  let { item, more = 0, onopen }: { item: UiMessage; more?: number; onopen: () => void } =
    $props();

  let src = $state<string | null>(null);
  const thumb = $derived(item.thumbnail ? `data:image/jpeg;base64,${item.thumbnail}` : null);
  const isVideo = $derived(item.kind === "video");

  $effect(() => {
    const media = item.media;
    src = null;
    if (!media || isVideo) return;
    let live = true;
    mediaSrc(media)
      .then((url) => {
        if (live) src = url;
      })
      .catch(() => {
        /* keep the thumbnail */
      });
    return () => {
      live = false;
    };
  });
</script>

<button
  class="tile"
  onclick={(e) => {
    e.stopPropagation();
    onopen();
  }}
  aria-label={more > 0 ? `Show ${more} more` : isVideo ? "Play video" : "Open photo"}
>
  {#if src ?? thumb}
    <img src={src ?? thumb} alt="" class:blur={!src && !isVideo} decoding="async" draggable="false" />
  {:else}
    <span class="ph">{isVideo ? "🎬" : "🖼"}</span>
  {/if}
  {#if isVideo && more === 0}<span class="play">▶</span>{/if}
  {#if more > 0}<span class="more">+{more}</span>{/if}
</button>

<style>
  .tile {
    position: relative;
    border: none;
    padding: 0;
    margin: 0;
    background: var(--wa-panel-2);
    border-radius: 4px;
    overflow: hidden;
    cursor: pointer;
    aspect-ratio: 1;
    width: 100%;
    display: grid;
    place-items: center;
  }
  .tile > * {
    grid-area: 1 / 1;
  }
  img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
  img.blur {
    filter: blur(6px);
    transform: scale(1.08);
  }
  .ph {
    font-size: 28px;
    opacity: 0.6;
  }
  .play {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    background: rgba(0, 0, 0, 0.5);
    color: #fff;
    display: grid;
    place-items: center;
    font-size: 16px;
  }
  .more {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    background: rgba(0, 0, 0, 0.55);
    color: #fff;
    font-size: 28px;
    font-weight: 500;
  }
</style>
