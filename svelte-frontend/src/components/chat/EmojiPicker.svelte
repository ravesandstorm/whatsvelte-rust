<script lang="ts">
  import {
    EMOJI_CATEGORIES,
    getRecentEmojis,
    pushRecentEmoji,
    searchEmojiShortcodes,
  } from "../../lib/emoji";

  // `overlay`: render as a fixed, top-most layer with a click-outside backdrop and
  // Escape-to-close (used by the composer). When false (default) the picker is
  // bare so a parent like ReactionPicker can own the overlay/layout.
  let {
    onpick,
    onclose,
    overlay = false,
  }: { onpick: (emoji: string) => void; onclose?: () => void; overlay?: boolean } = $props();

  // Recents are single emoji glyphs; defensively drop any malformed (e.g.
  // whitespace-bearing) entries so one bad value can't blow out the grid layout.
  const cleanRecents = (list: string[]) => list.filter((e) => e && !/\s/.test(e));

  const initialRecents = cleanRecents(getRecentEmojis());
  let recents = $state(initialRecents);
  let activeKey = $state(initialRecents.length ? "recent" : EMOJI_CATEGORIES[0].key);
  let query = $state("");
  let searchEl = $state<HTMLInputElement | undefined>();

  const tabs = $derived([
    ...(recents.length ? [{ key: "recent", label: "Recent", icon: "🕘" }] : []),
    ...EMOJI_CATEGORIES.map((c) => ({ key: c.key, label: c.label, icon: c.icon })),
  ]);

  // Typing a query switches the grid to name-based search (same index the
  // composer's `:` shortcode search uses); empty query shows the tabbed view.
  const trimmed = $derived(query.trim());
  const searching = $derived(trimmed.length > 0);
  const results = $derived(searching ? searchEmojiShortcodes(trimmed, 64) : []);

  function emojisFor(key: string): string[] {
    if (key === "recent") return recents;
    return EMOJI_CATEGORIES.find((c) => c.key === key)?.emojis ?? [];
  }

  function choose(emoji: string) {
    pushRecentEmoji(emoji);
    recents = cleanRecents(getRecentEmojis());
    onpick(emoji);
  }

  function onKey(e: KeyboardEvent) {
    // Only the overlay instance owns Escape; the bare instance is closed by its
    // parent (ReactionPicker) so we don't double-handle.
    if (!overlay) return;
    if (e.key === "Escape") {
      e.preventDefault();
      // Escape clears an active search first, then closes the picker.
      if (query) query = "";
      else onclose?.();
    }
  }

  // Autofocus the search box when shown as the composer overlay (not in the
  // small reaction popover, where stealing focus would be disruptive).
  $effect(() => {
    if (overlay) searchEl?.focus();
  });
</script>

<svelte:window onkeydown={onKey} />

{#snippet body()}
  <div class="picker">
    <div class="search">
      <input
        bind:this={searchEl}
        bind:value={query}
        type="text"
        placeholder="Search emoji"
        aria-label="Search emoji"
      />
      {#if onclose}
        <button class="close" title="Close" aria-label="Close" onclick={onclose}>✕</button>
      {/if}
    </div>

    {#if searching}
      <div class="grid">
        {#each results as r (r.code)}
          <button class="emoji" title={`:${r.code}:`} onclick={() => choose(r.emoji)}>{r.emoji}</button>
        {:else}
          <p class="empty">No emoji found</p>
        {/each}
      </div>
    {:else}
      <div class="tabs">
        {#each tabs as t (t.key)}
          <button
            class="tab"
            class:active={activeKey === t.key}
            title={t.label}
            onclick={() => (activeKey = t.key)}>{t.icon}</button
          >
        {/each}
      </div>
      <div class="grid">
        {#each emojisFor(activeKey) as e (e)}
          <button class="emoji" onclick={() => choose(e)}>{e}</button>
        {:else}
          <p class="empty">No emoji yet</p>
        {/each}
      </div>
    {/if}
  </div>
{/snippet}

{#if overlay}
  <div class="emoji-overlay" onclick={() => onclose?.()} role="presentation">
    <div class="anchor" onclick={(e) => e.stopPropagation()} role="presentation">
      {@render body()}
    </div>
  </div>
{:else}
  {@render body()}
{/if}

<style>
  .emoji-overlay {
    position: fixed;
    inset: 0;
    z-index: 1000;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .picker {
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
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px;
    border-bottom: 1px solid var(--wa-border);
  }
  .search input {
    flex: 1;
    min-width: 0;
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
  .search .close {
    flex: 0 0 auto;
    border: none;
    background: transparent;
    color: var(--wa-text-muted);
    font-size: 14px;
    padding: 4px 6px;
    border-radius: 6px;
  }
  .tabs {
    display: flex;
    gap: 2px;
    padding: 6px;
    border-bottom: 1px solid var(--wa-border);
    overflow-x: auto;
  }
  .tab {
    flex: 0 0 auto;
    border: none;
    background: transparent;
    font-size: 18px;
    padding: 4px 6px;
    border-radius: 6px;
    opacity: 0.7;
  }
  .tab.active {
    background: var(--wa-hover);
    opacity: 1;
  }
  .grid {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    display: grid;
    grid-template-columns: repeat(8, 1fr);
    grid-auto-rows: min-content;
    gap: 2px;
    padding: 6px;
  }
  .emoji {
    border: none;
    background: transparent;
    font-size: 22px;
    aspect-ratio: 1;
    border-radius: 6px;
    /* Keep cells from being widened by an over-wide value, which would otherwise
       break the 8-column grid into one overflowing horizontal row. */
    min-width: 0;
    overflow: hidden;
  }
  .emoji:hover {
    background: var(--wa-hover);
  }
  .empty {
    grid-column: 1 / -1;
    text-align: center;
    color: var(--wa-text-muted);
    font-size: 13px;
  }
</style>
