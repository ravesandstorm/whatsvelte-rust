<script lang="ts">
  // Quick reaction row (6 common emojis) with the full EmojiPicker below, shown
  // as a top-most overlay layer with click-outside + Escape to close.
  import { COMMON_REACTIONS } from "../../lib/emoji";
  import EmojiPicker from "./EmojiPicker.svelte";

  let {
    onpick,
    onclose,
    current,
  }: { onpick: (emoji: string) => void; onclose?: () => void; current?: string } = $props();

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      onclose?.();
    }
  }
</script>

<svelte:window onkeydown={onKey} />
<div class="reaction-overlay" onclick={() => onclose?.()} role="presentation">
  <div class="reaction-picker" onclick={(e) => e.stopPropagation()} role="presentation">
    <div class="common">
      {#each COMMON_REACTIONS as e (e)}
        <button class="quick" class:on={current === e} onclick={() => onpick(e)}>{e}</button>
      {/each}
    </div>
    <EmojiPicker {onpick} {onclose} />
  </div>
</div>

<style>
  .reaction-overlay {
    position: fixed;
    inset: 0;
    z-index: 1000;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .reaction-picker {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .common {
    display: flex;
    gap: 2px;
    padding: 4px;
    background: var(--wa-panel-2);
    border: 1px solid var(--wa-border);
    border-radius: 10px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  }
  .quick {
    border: none;
    background: transparent;
    font-size: 22px;
    padding: 4px 6px;
    border-radius: 8px;
    flex: 1;
  }
  .quick:hover {
    background: var(--wa-hover);
  }
  .quick.on {
    background: var(--wa-hover);
  }
</style>
