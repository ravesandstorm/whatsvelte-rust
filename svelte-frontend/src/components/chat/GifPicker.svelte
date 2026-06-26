<script lang="ts">
  import { onMount } from "svelte";
  import { klipyTrending, klipySearch, klipyConfigured, type Gif } from "../../lib/klipy";

  let { onpick }: { onpick: (gif: Gif) => void } = $props();

  let gifs = $state<Gif[]>([]);
  let query = $state("");
  let loading = $state(false);
  let error = $state<string | null>(null);
  let page = $state(1);
  let hasNext = $state(false);
  let searchEl = $state<HTMLInputElement>();
  let debounce: ReturnType<typeof setTimeout> | null = null;

  async function load(reset: boolean) {
    if (loading) return;
    loading = true;
    error = null;
    const p = reset ? 1 : page + 1;
    const q = query.trim();
    try {
      const res = q ? await klipySearch(q, p) : await klipyTrending(p);
      gifs = reset ? res.gifs : [...gifs, ...res.gifs];
      hasNext = res.hasNext;
      page = p;
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to load GIFs";
      if (reset) gifs = [];
    } finally {
      loading = false;
    }
  }

  function onInput() {
    if (debounce) clearTimeout(debounce);
    debounce = setTimeout(() => void load(true), 350);
  }

  // Infinite scroll: pull the next page as the user nears the bottom.
  function onScroll(e: Event) {
    const el = e.currentTarget as HTMLDivElement;
    if (hasNext && !loading && el.scrollTop + el.clientHeight >= el.scrollHeight - 140) {
      void load(false);
    }
  }

  onMount(() => {
    searchEl?.focus();
    if (klipyConfigured) void load(true);
  });
</script>

<div class="gif-picker">
  <div class="search">
    <input
      bind:this={searchEl}
      bind:value={query}
      oninput={onInput}
      type="text"
      placeholder="Search GIFs"
      aria-label="Search GIFs"
    />
  </div>

  {#if !klipyConfigured}
    <div class="state">
      <p>GIF search isn't configured.</p>
      <p class="hint">Set <code>VITE_KLIPY_API_KEY</code> and rebuild.</p>
    </div>
  {:else}
    <div class="grid" onscroll={onScroll}>
      {#each gifs as g (g.id)}
        <button class="cell" onclick={() => onpick(g)} aria-label="Send GIF">
          <img src={g.previewUrl} alt="" loading="lazy" />
        </button>
      {/each}
      {#if loading}
        <p class="state inline">Loading…</p>
      {:else if error}
        <p class="state inline">{error}</p>
      {:else if gifs.length === 0}
        <p class="state inline">No GIFs found</p>
      {/if}
    </div>
  {/if}
</div>

<style>
  .gif-picker {
    display: flex;
    flex-direction: column;
    width: 320px;
    height: 300px;
    background: var(--wa-panel-2);
    border: 1px solid var(--wa-border);
    border-radius: 10px;
    overflow: hidden;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  }
  .search {
    padding: 6px;
    border-bottom: 1px solid var(--wa-border);
  }
  .search input {
    width: 100%;
    box-sizing: border-box;
    border: none;
    background: var(--wa-panel);
    color: var(--wa-text);
    border-radius: 6px;
    padding: 6px 10px;
    font-size: 13px;
    outline: none;
  }
  .search input::placeholder {
    color: var(--wa-text-muted);
  }
  .grid {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    grid-auto-rows: 96px;
    gap: 4px;
    padding: 6px;
  }
  .cell {
    border: none;
    padding: 0;
    background: var(--wa-panel);
    border-radius: 6px;
    overflow: hidden;
    cursor: pointer;
  }
  .cell img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
  .state {
    flex: 1;
    display: grid;
    place-content: center;
    text-align: center;
    color: var(--wa-text-muted);
    font-size: 13px;
    padding: 16px;
  }
  .state.inline {
    grid-column: 1 / -1;
    align-self: start;
  }
  .state .hint {
    margin-top: 6px;
    font-size: 12px;
  }
  code {
    background: var(--wa-panel);
    padding: 1px 5px;
    border-radius: 4px;
  }
</style>
