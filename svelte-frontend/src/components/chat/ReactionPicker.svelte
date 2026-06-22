<script lang="ts">
  // Quick reaction row (6 common emojis) with the full EmojiPicker below.
  import { COMMON_REACTIONS } from "../../lib/emoji";
  import EmojiPicker from "./EmojiPicker.svelte";

  let {
    onpick,
    onclose,
    current,
  }: { onpick: (emoji: string) => void; onclose?: () => void; current?: string } = $props();
</script>

<div class="reaction-picker">
  <div class="common">
    {#each COMMON_REACTIONS as e (e)}
      <button class="quick" class:on={current === e} onclick={() => onpick(e)}>{e}</button>
    {/each}
  </div>
  <EmojiPicker {onpick} {onclose} />
</div>

<style>
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
