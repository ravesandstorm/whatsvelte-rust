<script lang="ts">
  // WhatsApp-style picker panel anchored above the composer's emoji button.
  // Bottom tab row switches between Emoji and GIF sections; Stickers is reserved
  // for a future section. The existing EmojiPicker is reused unchanged (bare mode).
  import EmojiPicker from "./EmojiPicker.svelte";
  import GifPicker from "./GifPicker.svelte";
  import type { Gif } from "../../lib/klipy";

  let {
    onemoji,
    ongif,
    onclose,
  }: {
    onemoji: (emoji: string) => void;
    ongif: (gif: Gif) => void;
    onclose: () => void;
  } = $props();

  type Tab = "emoji" | "gif" | "sticker";
  let tab = $state<Tab>("emoji");
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && onclose()} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="backdrop" onclick={onclose} role="presentation"></div>

<div class="panel">
  <div class="body">
    {#if tab === "emoji"}
      <EmojiPicker onpick={onemoji} />
    {:else if tab === "gif"}
      <GifPicker onpick={ongif} />
    {:else}
      <div class="soon">Stickers coming soon</div>
    {/if}
  </div>
  <div class="tabs" role="tablist">
    <button role="tab" aria-selected={tab === "emoji"} class:active={tab === "emoji"} onclick={() => (tab = "emoji")}>
      😊 Emoji
    </button>
    <button role="tab" aria-selected={tab === "gif"} class:active={tab === "gif"} onclick={() => (tab = "gif")}>
      GIF
    </button>
    <button role="tab" aria-selected={tab === "sticker"} class:active={tab === "sticker"} onclick={() => (tab = "sticker")}>
      Stickers
    </button>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 1000;
  }
  .panel {
    position: absolute;
    bottom: calc(100% + 8px);
    left: 0;
    z-index: 1001;
    display: flex;
    flex-direction: column;
    background: var(--wa-panel-2);
    border: 1px solid var(--wa-border);
    border-radius: 10px;
    overflow: hidden;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  }
  /* The Emoji/GIF children carry their own border/radius/shadow; neutralize the
     panel's so they don't double up. */
  .panel :global(.picker),
  .panel :global(.gif-picker) {
    border: none;
    border-radius: 0;
    box-shadow: none;
  }
  .body {
    display: flex;
  }
  .soon {
    width: 320px;
    height: 300px;
    display: grid;
    place-content: center;
    color: var(--wa-text-muted);
    font-size: 13px;
  }
  .tabs {
    display: flex;
    border-top: 1px solid var(--wa-border);
  }
  .tabs button {
    flex: 1;
    border: none;
    background: transparent;
    color: var(--wa-text-muted);
    padding: 9px 6px;
    font-size: 13px;
    cursor: pointer;
    border-top: 2px solid transparent;
  }
  .tabs button:hover {
    background: var(--wa-hover);
  }
  .tabs button.active {
    color: var(--wa-text);
    border-top-color: var(--wa-green, #00a884);
  }
</style>
