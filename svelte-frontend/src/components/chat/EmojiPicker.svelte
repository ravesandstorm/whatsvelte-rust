<script lang="ts">
  import { EMOJI_CATEGORIES, getRecentEmojis, pushRecentEmoji } from "../../lib/emoji";

  let { onpick, onclose }: { onpick: (emoji: string) => void; onclose?: () => void } =
    $props();

  const initialRecents = getRecentEmojis();
  let recents = $state(initialRecents);
  let activeKey = $state(initialRecents.length ? "recent" : EMOJI_CATEGORIES[0].key);

  const tabs = $derived([
    ...(recents.length ? [{ key: "recent", label: "Recent", icon: "🕘" }] : []),
    ...EMOJI_CATEGORIES.map((c) => ({ key: c.key, label: c.label, icon: c.icon })),
  ]);

  function emojisFor(key: string): string[] {
    if (key === "recent") return recents;
    return EMOJI_CATEGORIES.find((c) => c.key === key)?.emojis ?? [];
  }

  function choose(emoji: string) {
    pushRecentEmoji(emoji);
    recents = getRecentEmojis();
    onpick(emoji);
  }
</script>

<div class="picker">
  <div class="tabs">
    {#each tabs as t (t.key)}
      <button
        class="tab"
        class:active={activeKey === t.key}
        title={t.label}
        onclick={() => (activeKey = t.key)}>{t.icon}</button
      >
    {/each}
    {#if onclose}
      <button class="tab close" title="Close" onclick={onclose}>✕</button>
    {/if}
  </div>
  <div class="grid">
    {#each emojisFor(activeKey) as e (e)}
      <button class="emoji" onclick={() => choose(e)}>{e}</button>
    {:else}
      <p class="empty">No emoji yet</p>
    {/each}
  </div>
</div>

<style>
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
  .tab.close {
    margin-left: auto;
    font-size: 14px;
    color: var(--wa-text-muted);
  }
  .grid {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    display: grid;
    grid-template-columns: repeat(8, 1fr);
    gap: 2px;
    padding: 6px;
  }
  .emoji {
    border: none;
    background: transparent;
    font-size: 22px;
    aspect-ratio: 1;
    border-radius: 6px;
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
