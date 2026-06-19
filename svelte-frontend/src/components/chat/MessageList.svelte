<script lang="ts">
  import { tick } from "svelte";
  import type { UiMessage } from "../../lib/stores/messages.svelte";
  import { isGroup } from "../../lib/util/jid";
  import MessageBubble from "./MessageBubble.svelte";

  let { messages, chatJid }: { messages: UiMessage[]; chatJid: string } = $props();

  // Render only a window of the most recent messages. A full chat can hold
  // thousands of history messages (each with a base64 thumbnail) — mounting
  // them all is what spikes RAM when a chat opens. We start at the latest and
  // reveal older ones as the user scrolls up.
  const PAGE = 40;
  let limit = $state(PAGE);
  let el: HTMLDivElement | undefined = $state();
  // Whether the viewport is parked at the bottom; only then do we auto-follow
  // new messages, so scrolling up to read history doesn't yank you back down.
  let pinned = $state(true);

  const visible = $derived(messages.slice(Math.max(0, messages.length - limit)));
  const hasOlder = $derived(messages.length > limit);

  // Auto-scroll to the newest message on first render and whenever a message
  // arrives while parked at the bottom.
  $effect(() => {
    visible.length; // track
    if (pinned && el) el.scrollTop = el.scrollHeight;
  });

  async function onScroll() {
    const node = el;
    if (!node) return;
    pinned = node.scrollHeight - node.scrollTop - node.clientHeight < 80;

    // Near the top with more history available → reveal an older page and keep
    // the current messages visually in place (no jump).
    if (node.scrollTop < 60 && hasOlder) {
      const prevHeight = node.scrollHeight;
      const prevTop = node.scrollTop;
      limit += PAGE;
      await tick();
      node.scrollTop = node.scrollHeight - prevHeight + prevTop;
    }
  }
</script>

<div class="list" bind:this={el} onscroll={onScroll}>
  {#if messages.length === 0}
    <div class="loading">Loading messages…</div>
  {:else}
    {#if hasOlder}
      <div class="older">Showing the latest messages — scroll up for more</div>
    {/if}
    {#each visible as m (m.id)}
      <MessageBubble message={m} group={isGroup(chatJid)} />
    {/each}
  {/if}
</div>

<style>
  .list {
    flex: 1;
    /* Pair with flex:1 so the list scrolls internally instead of overflowing
       the conversation (which would push the composer off-screen). */
    min-height: 0;
    overflow-y: auto;
    padding: 14px 8%;
    /* Transparent so the conversation wallpaper (set on the parent) shows
       through; the parent defaults to var(--wa-chat-bg) when no wallpaper. */
    background: transparent;
    display: flex;
    flex-direction: column;
  }
  .loading {
    margin: auto;
    color: var(--wa-text-muted);
    font-size: 14px;
  }
  .older {
    align-self: center;
    margin: 4px 0 10px;
    padding: 4px 12px;
    border-radius: 12px;
    background: var(--wa-panel);
    color: var(--wa-text-muted);
    font-size: 12px;
  }
</style>
